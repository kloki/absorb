use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use ratatui::{Terminal, backend::Backend};

use crate::display;

const EASEIN_WORDS: usize = 10;
const FREEZE: Duration = Duration::from_secs(1);

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
    frozen_until: Instant,
    split_view: bool,
}

impl App {
    pub fn new(words: Vec<String>, text: String, wpm: u32) -> Self {
        let now = Instant::now();
        Self {
            words,
            text,
            current: 0,
            playing: true,
            target_wpm: wpm,
            last_advance: now + FREEZE,
            frozen_until: now + FREEZE,
            split_view: false,
        }
    }

    fn effective_wpm(&self) -> u32 {
        if self.current >= EASEIN_WORDS {
            return self.target_wpm;
        }
        let start = self.target_wpm / 3;
        let progress = self.current as f64 / EASEIN_WORDS as f64;
        let wpm = start as f64 + (self.target_wpm - start) as f64 * progress;
        (wpm as u32).max(50)
    }

    fn tick_duration(&self) -> Duration {
        let now = Instant::now();
        if now < self.frozen_until {
            self.frozen_until.duration_since(now)
        } else if self.playing {
            let tick = Duration::from_millis(60_000 / self.effective_wpm() as u64);
            tick.saturating_sub(self.last_advance.elapsed())
        } else {
            // Effectively infinite: when paused, only input from select! wakes the loop
            Duration::from_secs(86400)
        }
    }

    pub async fn run<B: Backend<Error = io::Error>>(
        &mut self,
        term: &mut Terminal<B>,
    ) -> io::Result<()> {
        let mut reader = EventStream::new();

        loop {
            let wpm = self.effective_wpm();
            term.draw(|f| {
                display::draw(
                    f,
                    &self.words,
                    &self.text,
                    self.current,
                    wpm,
                    self.playing,
                    self.split_view,
                );
            })?;

            let tick = tokio::time::sleep(self.tick_duration());

            tokio::select! {
                _ = tick => {
                    if self.playing && Instant::now() >= self.frozen_until {
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
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return Action::Continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Action::Quit,
                KeyCode::Char(' ') => self.toggle_play(),
                KeyCode::Left | KeyCode::Char('h') => self.retreat(),
                KeyCode::Right | KeyCode::Char('l') => self.step_forward(),
                KeyCode::Up | KeyCode::Char('k') => self.increase_speed(),
                KeyCode::Down | KeyCode::Char('j') => self.decrease_speed(),
                KeyCode::Char('r') => self.restart(),
                KeyCode::Char('v') => self.split_view = !self.split_view,
                _ => {}
            }
        }
        Action::Continue
    }

    fn toggle_play(&mut self) {
        self.playing = !self.playing;
        if self.playing {
            self.last_advance = Instant::now();
            if self.current >= self.words.len() {
                self.current = 0;
            }
        }
    }

    fn advance(&mut self) {
        if self.current < self.words.len() {
            self.current += 1;
            self.last_advance = Instant::now();
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
        self.target_wpm = (self.target_wpm + 25).min(1000);
    }

    fn decrease_speed(&mut self) {
        self.target_wpm = (self.target_wpm.saturating_sub(25)).max(50);
    }

    fn restart(&mut self) {
        let now = Instant::now();
        self.current = 0;
        self.playing = true;
        self.last_advance = now + FREEZE;
        self.frozen_until = now + FREEZE;
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
            frozen_until: now,
            split_view: false,
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
        let app = test_app(600, EASEIN_WORDS, true);
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
    fn tick_duration_when_frozen() {
        let mut app = test_app(600, 0, true);
        app.frozen_until = Instant::now() + Duration::from_millis(500);
        let d = app.tick_duration();
        assert!(d > Duration::from_millis(400));
        assert!(d <= Duration::from_millis(500));
    }

    #[test]
    fn tick_duration_when_playing() {
        let app = test_app(600, EASEIN_WORDS, true);
        // 600 WPM = 100ms per word, last_advance is now so remaining ≈ 100ms
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
