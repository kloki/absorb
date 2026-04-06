use std::{
    fs,
    io::{self, BufWriter, IsTerminal, Read},
    path::PathBuf,
    process,
};

use clap::{Parser, ValueEnum};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        self,
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    style::Color,
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
    #[arg(short, long, default_value_t = 600, value_parser = clap::value_parser!(u32).range(50..=2000))]
    wpm: u32,

    /// Highlight color
    #[arg(short, long, value_enum, default_value_t = HighlightColor::Red)]
    color: HighlightColor,
}

#[derive(Clone, ValueEnum)]
enum HighlightColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl From<HighlightColor> for Color {
    fn from(c: HighlightColor) -> Color {
        match c {
            HighlightColor::Black => Color::Black,
            HighlightColor::Red => Color::Red,
            HighlightColor::Green => Color::Green,
            HighlightColor::Yellow => Color::Yellow,
            HighlightColor::Blue => Color::Blue,
            HighlightColor::Magenta => Color::Magenta,
            HighlightColor::Cyan => Color::Cyan,
            HighlightColor::White => Color::White,
        }
    }
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
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

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(info);
    }));

    let mut output = io::stdout().lock();
    enable_raw_mode()?;
    crossterm::execute!(output, EnterAlternateScreen, EnableMouseCapture)?;
    let mut term = Terminal::new(CrosstermBackend::new(BufWriter::new(output)))?;

    let highlight: Color = cli.color.into();
    let mut app = app::App::new(words, text, cli.wpm, highlight);
    let result = app.run(&mut term).await;

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    result
}
