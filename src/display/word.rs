use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use tui_big_text::{BigText, PixelSize};

pub fn orp_index(word: &str) -> usize {
    let len = word.chars().count();
    if len <= 1 { 0 } else { (len - 1) / 4 + 1 }
}

pub fn word<'a>(width: u16, w: &str, highlight: Color) -> Paragraph<'a> {
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

pub fn big_word<'a>(w: &str, highlight: Color) -> BigText<'a> {
    let orp = orp_index(w);
    let chars: Vec<char> = w.chars().collect();
    let before: String = chars[..orp].iter().collect();
    let focus: String = chars[orp..orp + 1].iter().collect();
    let after: String = chars[orp + 1..].iter().collect();

    let line = Line::from(vec![
        Span::styled(before, Style::default().fg(Color::White)),
        Span::styled(
            focus,
            Style::default().fg(highlight).add_modifier(Modifier::BOLD),
        ),
        Span::styled(after, Style::default().fg(Color::White)),
    ]);

    BigText::builder()
        .pixel_size(PixelSize::HalfHeight)
        .lines(vec![line])
        .build()
}

pub fn end() -> Paragraph<'static> {
    Paragraph::new("— END —")
        .centered()
        .style(Style::default().fg(Color::White))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::tests::render_to_string;

    #[test]
    fn orp_index_single_char() {
        assert_eq!(orp_index("a"), 0);
    }

    #[test]
    fn orp_index_short_words() {
        assert_eq!(orp_index("hi"), 1);
        assert_eq!(orp_index("the"), 1);
        assert_eq!(orp_index("word"), 1);
    }

    #[test]
    fn orp_index_medium_words() {
        assert_eq!(orp_index("hello"), 2);
        assert_eq!(orp_index("absorb"), 2);
        assert_eq!(orp_index("reading"), 2);
        assert_eq!(orp_index("keyboard"), 2);
    }

    #[test]
    fn orp_index_long_words() {
        assert_eq!(orp_index("beautiful"), 3);
        assert_eq!(orp_index("programmer"), 3);
        assert_eq!(orp_index("outstanding"), 3);
        assert_eq!(orp_index("transmission"), 3);
    }

    #[test]
    fn orp_index_very_long_words() {
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
}
