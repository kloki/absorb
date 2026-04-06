use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{LineGauge, Paragraph},
};

pub struct ViewState<'a> {
    pub words: &'a [String],
    pub text: &'a str,
    pub current: usize,
    pub wpm: u32,
    pub playing: bool,
    pub split_view: bool,
    pub highlight: Color,
}

fn orp_index(word: &str) -> usize {
    let len = word.chars().count();
    if len <= 1 { 0 } else { (len - 1) / 4 + 1 }
}

pub fn draw(frame: &mut Frame, state: &ViewState) {
    let ViewState {
        words,
        text,
        current,
        wpm,
        playing,
        split_view,
        highlight,
    } = *state;
    let area = frame.area();

    let outer = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    if split_view {
        let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(outer[0]);
        draw_word_pane(frame, cols[0], words, current, highlight);
        let tv = text_view(text, current, cols[1].height, highlight);
        frame.render_widget(tv, cols[1]);
    } else {
        draw_word_pane(frame, outer[0], words, current, highlight);
    }

    frame.render_widget(progress(current, words.len()), outer[1]);

    let footer = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(outer[2]);
    frame.render_widget(controls(), footer[0]);
    frame.render_widget(status(current, words.len(), wpm, playing), footer[1]);
}

fn draw_word_pane(
    frame: &mut Frame,
    area: Rect,
    words: &[String],
    current: usize,
    highlight: Color,
) {
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(area);

    frame.render_widget(crate::banners::header(), layout[0]);

    if current < words.len() {
        frame.render_widget(word(area.width, &words[current], highlight), layout[2]);
    } else {
        frame.render_widget(end(), layout[2]);
    }
}

fn text_view(text: &str, current: usize, height: u16, highlight: Color) -> Paragraph<'static> {
    let mut word_index = 0;
    let mut current_line_index = 0;
    let mut lines: Vec<Line<'static>> = Vec::new();

    for (line_num, text_line) in text.lines().enumerate() {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut col = 0;

        for token in text_line.split_whitespace() {
            if let Some(pos) = text_line[col..].find(token) {
                let leading = &text_line[col..col + pos];
                if !leading.is_empty() {
                    spans.push(Span::styled(
                        leading.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                col += pos + token.len();
            }

            if word_index == current {
                current_line_index = line_num;
                spans.push(Span::styled(
                    token.to_string(),
                    Style::default().fg(highlight).add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    token.to_string(),
                    Style::default().fg(Color::White),
                ));
            }
            word_index += 1;
        }

        // Preserve any trailing whitespace
        if col < text_line.len() {
            spans.push(Span::raw(text_line[col..].to_string()));
        }

        lines.push(Line::from(spans));
    }

    let scroll = current_line_index.saturating_sub(height as usize / 2);

    Paragraph::new(lines).scroll((scroll as u16, 0))
}

fn word<'a>(width: u16, w: &str, highlight: Color) -> Paragraph<'a> {
    let orp = orp_index(w);
    let center = width as usize / 2;
    let padding = center.saturating_sub(orp);

    let chars: Vec<char> = w.chars().collect();
    let before: String = chars[..orp].iter().collect();
    let focus: String = chars[orp..orp + 1].iter().collect();
    let after: String = chars[orp + 1..].iter().collect();

    let line = Line::from(vec![
        Span::raw(" ".repeat(padding)),
        Span::styled(before, Style::default().fg(Color::White)),
        Span::styled(
            focus,
            Style::default().fg(highlight).add_modifier(Modifier::BOLD),
        ),
        Span::styled(after, Style::default().fg(Color::White)),
    ]);

    Paragraph::new(line)
}

fn end() -> Paragraph<'static> {
    Paragraph::new("— END —")
        .centered()
        .style(Style::default().fg(Color::White))
}

fn progress(current: usize, total: usize) -> LineGauge<'static> {
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

fn controls() -> Paragraph<'static> {
    Paragraph::new("SPACE play/pause | \u{2190}\u{2192} navigate | \u{2191}\u{2193} speed | v split-view | r restart | q quit")
        .style(Style::default().fg(Color::White))
}

pub fn status(current: usize, total: usize, wpm: u32, playing: bool) -> Paragraph<'static> {
    let state = if playing { "PLAYING" } else { "PAUSED" };
    let pos = current.min(total);
    let text = format!("{} WPM | {}/{} | {}", wpm, pos, total, state);
    Paragraph::new(text)
        .right_aligned()
        .style(Style::default().fg(Color::White))
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

    use super::*;

    fn render_to_string(widget: impl Widget, width: u16, height: u16) -> String {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        let mut s = String::new();
        for y in 0..height {
            for x in 0..width {
                s.push_str(buf.cell((x, y)).unwrap().symbol());
            }
            if y < height - 1 {
                s.push('\n');
            }
        }
        s
    }

    #[test]
    fn orp_index_single_char() {
        assert_eq!(orp_index("a"), 0);
    }

    #[test]
    fn orp_index_short_words() {
        // len 2-4 → index 1
        assert_eq!(orp_index("hi"), 1);
        assert_eq!(orp_index("the"), 1);
        assert_eq!(orp_index("word"), 1);
    }

    #[test]
    fn orp_index_medium_words() {
        // len 5-8 → index 2
        assert_eq!(orp_index("hello"), 2);
        assert_eq!(orp_index("absorb"), 2);
        assert_eq!(orp_index("reading"), 2);
        assert_eq!(orp_index("keyboard"), 2);
    }

    #[test]
    fn orp_index_long_words() {
        // len 9-12 → index 3
        assert_eq!(orp_index("beautiful"), 3);
        assert_eq!(orp_index("programmer"), 3);
        assert_eq!(orp_index("outstanding"), 3);
        assert_eq!(orp_index("transmission"), 3);
    }

    #[test]
    fn orp_index_very_long_words() {
        // len 13+ → index 4
        assert_eq!(orp_index("understanding"), 4);
    }

    #[test]
    fn orp_index_unicode() {
        assert_eq!(orp_index("café"), 1);
        assert_eq!(orp_index("日本語テスト"), 2);
    }

    #[test]
    fn word_orp_at_center() {
        let rendered = render_to_string(word(20, "hello", Color::Red), 20, 1);
        // ORP index for "hello" (len 5) is 2 ('l'), should be at column 10 (width/2)
        assert_eq!(rendered.chars().nth(10).unwrap(), 'l');
        assert!(rendered.trim().eq("hello"));
    }

    #[test]
    fn word_single_char_at_center() {
        let rendered = render_to_string(word(20, "I", Color::Red), 20, 1);
        assert_eq!(rendered.chars().nth(10).unwrap(), 'I');
    }

    #[test]
    fn end_shows_text() {
        let rendered = render_to_string(end(), 20, 1);
        assert!(rendered.contains("END"));
    }

    #[test]
    fn progress_renders_without_panic() {
        // Just verify these don't panic — ratio is internal state
        render_to_string(progress(0, 0), 20, 1);
        render_to_string(progress(5, 10), 20, 1);
        render_to_string(progress(10, 10), 20, 1);
    }

    #[test]
    fn status_playing() {
        let rendered = render_to_string(status(5, 10, 300, true), 40, 1);
        assert!(rendered.contains("300 WPM"));
        assert!(rendered.contains("5/10"));
        assert!(rendered.contains("PLAYING"));
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
