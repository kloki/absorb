use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use futures::StreamExt;
use ratatui::{Terminal, backend::Backend, layout::Rect, style::Color};

use crate::display::{self, DrawResult, ViewState, WordMap};

enum Action {
    Continue,
    Quit,
}

pub struct App {
    words: Vec<String>,
    text: String,
    current: usize,
    playing: bool,
    target_wpm: u32,
    last_advance: Instant,
    split_view: bool,
    highlight: Color,
    scroll_offset: Option<usize>,
    last_scroll: usize,
    word_map: WordMap,
    text_pane: Option<Rect>,
    show_help: bool,
    was_playing: Option<bool>,
    words_since_resume: usize,
    big_text: bool,
    easein_words: usize,
    sentence_pause: f64,
}

impl App {
    pub fn new(
        words: Vec<String>,
        text: String,
        wpm: u32,
        highlight: Color,
        big_text: bool,
        easein_words: usize,
        sentence_pause: f64,
    ) -> Self {
        Self {
            words,
            text,
            current: 0,
            playing: true,
            target_wpm: wpm,
            last_advance: Instant::now(),
            split_view: false,
            highlight,
            scroll_offset: None,
            last_scroll: 0,
            word_map: WordMap::default(),
            text_pane: None,
            show_help: false,
            was_playing: None,
            words_since_resume: 0,
            big_text,
            easein_words,
            sentence_pause,
        }
    }

    fn easein_wpm(&self, words: usize) -> u32 {
        if self.easein_words == 0 || words >= self.easein_words {
            return self.target_wpm;
        }
        let start = self.target_wpm / 3;
        let progress = words as f64 / self.easein_words as f64;
        let wpm = start as f64 + (self.target_wpm - start) as f64 * progress;
        (wpm as u32).max(50)
    }

    fn effective_wpm(&self) -> u32 {
        self.easein_wpm(self.current)
            .min(self.easein_wpm(self.words_since_resume))
    }

    fn tick_duration(&self) -> Duration {
        if self.playing {
            let base = Duration::from_millis(60_000 / self.effective_wpm() as u64);
            let tick = if self.sentence_pause > 0.0
                && self.current < self.words.len()
                && self.words[self.current].ends_with('.')
            {
                base.mul_f64(self.sentence_pause)
            } else {
                base
            };
            tick.saturating_sub(self.last_advance.elapsed())
        } else {
            // Effectively infinite: when paused, only input from select! wakes the loop
            Duration::from_secs(86400)
        }
    }

    fn view_state(&self) -> ViewState<'_> {
        ViewState {
            words: &self.words,
            text: &self.text,
            current: self.current,
            wpm: self.effective_wpm(),
            playing: self.playing,
            split_view: self.split_view,
            highlight: self.highlight,
            scroll_offset: self.scroll_offset,
            show_help: self.show_help,
            big_text: self.big_text,
        }
    }

    pub async fn run<B: Backend<Error = io::Error>>(
        &mut self,
        term: &mut Terminal<B>,
    ) -> io::Result<()> {
        let mut reader = EventStream::new();

        loop {
            let state = self.view_state();
            let mut draw_result = DrawResult::default();
            term.draw(|f| {
                draw_result = display::draw(f, &state);
            })?;
            self.word_map = draw_result.word_map;
            self.text_pane = draw_result.text_pane;
            self.last_scroll = draw_result.scroll;

            let tick = tokio::time::sleep(self.tick_duration());

            tokio::select! {
                _ = tick => {
                    if self.playing {
                        self.advance();
                    }
                }
                Some(Ok(event)) = reader.next() => {
                    if let Action::Quit = self.handle_event(event) {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Action {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            _ => Action::Continue,
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Action {
        if key.kind != KeyEventKind::Press {
            return Action::Continue;
        }
        if self.show_help {
            self.show_help = false;
            if let Some(was) = self.was_playing.take() {
                self.playing = was;
                if self.playing {
                    self.last_advance = Instant::now();
                    self.words_since_resume = 0;
                }
            }
            return Action::Continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Action::Quit,
            KeyCode::Char(' ') => self.toggle_play(),
            KeyCode::Left => self.retreat(),
            KeyCode::Right => self.step_forward(),
            KeyCode::Up => self.increase_speed(),
            KeyCode::Down => self.decrease_speed(),
            KeyCode::Char('r') => self.restart(),
            KeyCode::Char('v') => self.split_view = !self.split_view,
            KeyCode::Char('b') => self.big_text = !self.big_text,
            KeyCode::Char('c') => self.cycle_color(),
            KeyCode::Char('h') => {
                self.was_playing = Some(self.playing);
                self.playing = false;
                self.show_help = true;
            }
            _ => {}
        }
        Action::Continue
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> Action {
        if !self.split_view {
            return Action::Continue;
        }
        let Some(pane) = self.text_pane else {
            return Action::Continue;
        };
        let in_pane = mouse.column >= pane.x
            && mouse.column < pane.x + pane.width
            && mouse.row >= pane.y
            && mouse.row < pane.y + pane.height;
        if !in_pane {
            return Action::Continue;
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                let offset = self.scroll_offset.unwrap_or(self.last_scroll);
                self.scroll_offset = Some(offset.saturating_sub(3));
            }
            MouseEventKind::ScrollDown => {
                let offset = self.scroll_offset.unwrap_or(self.last_scroll);
                self.scroll_offset = Some(offset + 3);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                let rel_col = mouse.column - pane.x;
                let rel_row = mouse.row - pane.y;
                let scroll = self.scroll_offset.unwrap_or(self.last_scroll);
                let abs_line = scroll + rel_row as usize;

                if let Some(idx) = self.word_map.hit_test(abs_line, rel_col) {
                    self.current = idx;
                    self.playing = false;
                    self.scroll_offset = None;
                }
            }
            _ => {}
        }
        Action::Continue
    }

    fn toggle_play(&mut self) {
        self.playing = !self.playing;
        if self.playing {
            self.last_advance = Instant::now();
            self.words_since_resume = 0;
            if self.current >= self.words.len() {
                self.current = 0;
            }
        }
    }

    fn advance(&mut self) {
        if self.current < self.words.len() {
            self.current += 1;
            self.words_since_resume += 1;
            self.last_advance = Instant::now();
            self.scroll_offset = None;
        }
        if self.current >= self.words.len() {
            self.playing = false;
        }
    }

    fn retreat(&mut self) {
        self.playing = false;
        self.current = self.current.saturating_sub(1);
    }

    fn step_forward(&mut self) {
        self.playing = false;
        if self.current < self.words.len() {
            self.current += 1;
        }
    }

    fn increase_speed(&mut self) {
        self.target_wpm = (self.target_wpm + 25).min(2000);
    }

    fn decrease_speed(&mut self) {
        self.target_wpm = (self.target_wpm.saturating_sub(25)).max(50);
    }

    fn cycle_color(&mut self) {
        const COLORS: [Color; 7] = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::White,
        ];
        let pos = COLORS
            .iter()
            .position(|&c| c == self.highlight)
            .unwrap_or(0);
        self.highlight = COLORS[(pos + 1) % COLORS.len()];
    }

    fn restart(&mut self) {
        self.current = 0;
        self.words_since_resume = 0;
        self.playing = true;
        self.last_advance = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(wpm: u32, current: usize, playing: bool) -> App {
        let now = Instant::now();
        App {
            words: vec!["a".into(); 20],
            text: "a ".repeat(20).trim().to_string(),
            current,
            playing,
            target_wpm: wpm,
            last_advance: now,
            split_view: false,
            highlight: Color::Red,
            scroll_offset: None,
            last_scroll: 0,
            word_map: WordMap::default(),
            text_pane: None,
            show_help: false,
            was_playing: None,
            words_since_resume: current,
            big_text: false,
            easein_words: 10,
            sentence_pause: 0.0,
        }
    }

    #[test]
    fn effective_wpm_at_start() {
        let app = test_app(600, 0, true);
        // Word 0: should be target/3 = 200
        assert_eq!(app.effective_wpm(), 200);
    }

    #[test]
    fn effective_wpm_midway() {
        let app = test_app(600, 5, true);
        // Word 5: start=200, progress=0.5, wpm = 200 + 400*0.5 = 400
        assert_eq!(app.effective_wpm(), 400);
    }

    #[test]
    fn effective_wpm_at_easein_boundary() {
        let app = test_app(600, 10, true);
        assert_eq!(app.effective_wpm(), 600);
    }

    #[test]
    fn effective_wpm_past_easein() {
        let app = test_app(600, 15, true);
        assert_eq!(app.effective_wpm(), 600);
    }

    #[test]
    fn effective_wpm_minimum_clamp() {
        // With very low target, start (target/3) could be below 50
        let app = test_app(90, 0, true);
        assert_eq!(app.effective_wpm(), 50);
    }

    #[test]
    fn effective_wpm_easein_after_resume() {
        let mut app = test_app(600, 15, false);
        // Simulate pause then resume
        app.toggle_play();
        // words_since_resume is 0, so effective WPM should be target/3
        assert_eq!(app.effective_wpm(), 200);
    }

    #[test]
    fn effective_wpm_ramps_after_resume() {
        let mut app = test_app(600, 15, false);
        app.toggle_play();
        app.words_since_resume = 5;
        // Midway through resume ease-in: start=200, progress=0.5, wpm=400
        assert_eq!(app.effective_wpm(), 400);
    }

    #[test]
    fn effective_wpm_full_speed_after_resume_easein() {
        let mut app = test_app(600, 15, false);
        app.toggle_play();
        app.words_since_resume = 10;
        assert_eq!(app.effective_wpm(), 600);
    }

    #[test]
    fn effective_wpm_no_ramp() {
        let mut app = test_app(600, 0, true);
        app.easein_words = 0;
        assert_eq!(app.effective_wpm(), 600);
    }

    #[test]
    fn tick_duration_when_playing() {
        let app = test_app(600, 10, true);
        // 600 WPM = 100ms per word, last_advance is now so remaining ≈ 100ms
        let d = app.tick_duration();
        assert!(d > Duration::from_millis(90));
        assert!(d <= Duration::from_millis(100));
    }

    #[test]
    fn tick_duration_sentence_pause() {
        let mut app = test_app(600, 10, true);
        app.words[10] = "end.".into();
        app.sentence_pause = 2.0;
        // 600 WPM = 100ms base, on "end." should be ~200ms
        let d = app.tick_duration();
        assert!(d > Duration::from_millis(180));
        assert!(d <= Duration::from_millis(200));
    }

    #[test]
    fn tick_duration_no_sentence_pause() {
        let mut app = test_app(600, 10, true);
        app.words[10] = "end.".into();
        app.sentence_pause = 0.0;
        // Disabled: should be normal ~100ms
        let d = app.tick_duration();
        assert!(d > Duration::from_millis(90));
        assert!(d <= Duration::from_millis(100));
    }

    #[test]
    fn tick_duration_when_paused() {
        let app = test_app(600, 5, false);
        assert_eq!(app.tick_duration(), Duration::from_secs(86400));
    }
}
