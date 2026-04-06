use std::{
    fs,
    io::{self, BufWriter, IsTerminal, Read},
    path::PathBuf,
    process,
};

use clap::Parser;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        self,
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

mod app;
mod banners;
mod display;

#[derive(Parser)]
#[command(
    name = "absorb",
    about = "Quickly absorb text using RSVP speed reading"
)]
struct Cli {
    /// File to read (reads from stdin if not provided)
    file: Option<PathBuf>,

    /// Words per minute
    #[arg(short, long, default_value_t = 600)]
    wpm: u32,
}

fn read_input(file: Option<PathBuf>) -> Option<String> {
    if let Some(path) = file {
        Some(fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        }))
    } else if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {}", e);
            process::exit(1);
        });
        Some(buf)
    } else {
        None
    }
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let text = match read_input(cli.file) {
        Some(t) => t,
        None => {
            eprintln!("No input provided. Pass a file path or pipe text via stdin.");
            process::exit(1);
        }
    };

    let words: Vec<String> = text.split_whitespace().map(String::from).collect();
    if words.is_empty() {
        eprintln!("No words found in input.");
        process::exit(1);
    }

    let mut output = io::stdout().lock();
    enable_raw_mode()?;
    crossterm::execute!(output, EnterAlternateScreen, EnableMouseCapture)?;
    let mut term = Terminal::new(CrosstermBackend::new(BufWriter::new(output)))?;

    let mut app = app::App::new(words, cli.wpm);
    let result = app.run(&mut term);

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    result
}
