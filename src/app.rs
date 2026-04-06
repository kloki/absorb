use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    Terminal,
    backend::Backend,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};

use crate::display;

const EASEIN_WORDS: usize = 10;
const FREEZE: Duration = Duration::from_secs(1);

enum Action {
    Continue,
    Quit,
}

pub struct App {
    words: Vec<String>,
    current: usize,
    playing: bool,
    target_wpm: u32,
    last_advance: Instant,
    frozen_until: Instant,
}

impl App {
    pub fn new(words: Vec<String>, wpm: u32) -> Self {
        let now = Instant::now();
        Self {
            words,
            current: 0,
            playing: true,
            target_wpm: wpm,
            last_advance: now + FREEZE,
            frozen_until: now + FREEZE,
        }
    }

    fn is_frozen(&self) -> bool {
        Instant::now() < self.frozen_until
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

    pub fn run<B: Backend<Error = io::Error>>(&mut self, term: &mut Terminal<B>) -> io::Result<()> {
        loop {
            let wpm = self.effective_wpm();
            term.draw(|f| {
                display::draw(f, &self.words, self.current, wpm, self.playing);
            })?;

            let timeout = if self.is_frozen() {
                self.frozen_until.duration_since(Instant::now())
            } else if self.playing {
                let tick = Duration::from_millis(60_000 / wpm as u64);
                tick.saturating_sub(self.last_advance.elapsed())
            } else {
                Duration::from_secs(1)
            };

            if event::poll(timeout)? {
                if let Action::Quit = self.handle_input()? {
                    return Ok(());
                }
            }

            if self.playing
                && !self.is_frozen()
                && self.last_advance.elapsed()
                    >= Duration::from_millis(60_000 / self.effective_wpm() as u64)
            {
                self.advance();
            }
        }
    }

    fn handle_input(&mut self) -> io::Result<Action> {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(Action::Continue);
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(Action::Quit),
                KeyCode::Char(' ') => self.toggle_play(),
                KeyCode::Left | KeyCode::Char('h') => self.retreat(),
                KeyCode::Right | KeyCode::Char('l') => self.step_forward(),
                KeyCode::Up | KeyCode::Char('k') => self.increase_speed(),
                KeyCode::Down | KeyCode::Char('j') => self.decrease_speed(),
                KeyCode::Char('r') => self.restart(),
                _ => {}
            }
        }
        Ok(Action::Continue)
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
