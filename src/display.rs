use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{LineGauge, Paragraph},
};

fn orp_index(word: &str) -> usize {
    let len = word.chars().count();
    if len <= 1 { 0 } else { (len - 1) / 4 + 1 }
}

pub fn draw(frame: &mut Frame, words: &[String], current: usize, wpm: u32, playing: bool) {
    let area = frame.area();

    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    frame.render_widget(crate::banners::header(), layout[0]);

    if current < words.len() {
        render_word(frame, layout[2], &words[current]);
    } else {
        let end = Paragraph::new("— END —")
            .centered()
            .style(Style::default().fg(Color::White));
        frame.render_widget(end, layout[2]);
    }

    render_progress(frame, layout[4], current, words.len());

    let footer = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(layout[5]);
    render_footer(frame, footer[0]);
    render_status(frame, footer[1], current, words.len(), wpm, playing);
}

fn render_word(frame: &mut Frame, area: ratatui::layout::Rect, word: &str) {
    let orp = orp_index(word);
    let center = area.width as usize / 2;
    let padding = center.saturating_sub(orp);

    let chars: Vec<char> = word.chars().collect();
    let before: String = chars[..orp].iter().collect();
    let focus: String = chars[orp..orp + 1].iter().collect();
    let after: String = chars[orp + 1..].iter().collect();

    let line = Line::from(vec![
        Span::raw(" ".repeat(padding)),
        Span::styled(before, Style::default().fg(Color::White)),
        Span::styled(
            focus,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled(after, Style::default().fg(Color::White)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn render_progress(frame: &mut Frame, area: ratatui::layout::Rect, current: usize, total: usize) {
    let ratio = if total == 0 {
        0.0
    } else {
        (current as f64 / total as f64).min(1.0)
    };

    let gauge = LineGauge::default()
        .filled_style(Style::default().fg(Color::Cyan))
        .unfilled_style(Style::default().fg(Color::DarkGray))
        .ratio(ratio);

    frame.render_widget(gauge, area);
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect) {
    let controls = Paragraph::new("SPACE play/pause | \u{2190}\u{2192} navigate | \u{2191}\u{2193} speed | r restart | q quit")
        .style(Style::default().fg(Color::White));
    frame.render_widget(controls, area);
}

pub fn render_status(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    current: usize,
    total: usize,
    wpm: u32,
    playing: bool,
) {
    let state = if playing { "PLAYING" } else { "PAUSED" };
    let pos = current.min(total);
    let text = format!("{} WPM | {}/{} | {}", wpm, pos, total, state);
    let status = Paragraph::new(text)
        .right_aligned()
        .style(Style::default().fg(Color::White));
    frame.render_widget(status, area);
}
