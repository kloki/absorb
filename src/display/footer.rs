use ratatui::{
    style::{Color, Style},
    widgets::{LineGauge, Paragraph},
};

pub fn progress(current: usize, total: usize) -> LineGauge<'static> {
    let ratio = if total == 0 {
        0.0
    } else {
        (current as f64 / total as f64).min(1.0)
    };

    LineGauge::default()
        .filled_style(Style::default().fg(Color::Cyan))
        .unfilled_style(Style::default().fg(Color::DarkGray))
        .ratio(ratio)
}

pub fn controls() -> Paragraph<'static> {
    Paragraph::new("(r)estart | (q)uit | (h)elp").style(Style::default().fg(Color::White))
}

pub fn status(current: usize, total: usize, wpm: u32, playing: bool) -> Paragraph<'static> {
    let state = if playing { "" } else { "[PAUSED]" };
    let pos = current.min(total);
    let text = format!("{} {} WPM | {}/{}", state, wpm, pos, total);
    Paragraph::new(text)
        .right_aligned()
        .style(Style::default().fg(Color::White))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::tests::render_to_string;

    #[test]
    fn progress_renders_without_panic() {
        render_to_string(progress(0, 0), 20, 1);
        render_to_string(progress(5, 10), 20, 1);
        render_to_string(progress(10, 10), 20, 1);
    }

    #[test]
    fn status_playing() {
        let rendered = render_to_string(status(5, 10, 300, true), 40, 1);
        assert!(rendered.contains("300 WPM"));
        assert!(rendered.contains("5/10"));
    }

    #[test]
    fn status_paused() {
        let rendered = render_to_string(status(5, 10, 300, false), 40, 1);
        assert!(rendered.contains("PAUSED"));
    }

    #[test]
    fn status_clamps_current_to_total() {
        let rendered = render_to_string(status(15, 10, 600, false), 40, 1);
        assert!(rendered.contains("10/10"));
        assert!(!rendered.contains("15/10"));
    }
}
