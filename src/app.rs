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

enum Action {
    Continue,
    Quit,
}

pub struct App {
    words: Vec<String>,
    current: usize,
    playing: bool,
    wpm: u32,
    last_advance: Instant,
}

impl App {
    pub fn new(words: Vec<String>, wpm: u32) -> Self {
        Self {
            words,
            current: 0,
            playing: false,
            wpm,
            last_advance: Instant::now(),
        }
    }

    pub fn run<B: Backend<Error = io::Error>>(&mut self, term: &mut Terminal<B>) -> io::Result<()> {
        loop {
            term.draw(|f| {
                display::draw(f, &self.words, self.current, self.wpm, self.playing);
            })?;

            let timeout = if self.playing {
                let tick = Duration::from_millis(60_000 / self.wpm as u64);
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
                && self.last_advance.elapsed() >= Duration::from_millis(60_000 / self.wpm as u64)
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
        self.wpm = (self.wpm + 25).min(1000);
    }

    fn decrease_speed(&mut self) {
        self.wpm = (self.wpm.saturating_sub(25)).max(50);
    }

    fn restart(&mut self) {
        self.current = 0;
        self.playing = false;
    }
}
