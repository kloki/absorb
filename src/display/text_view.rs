use std::ops::Range;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

#[derive(Default, Clone)]
pub struct WordMap {
    entries: Vec<(usize, Range<u16>, usize)>, // (line, col_range, word_index)
}

impl WordMap {
    pub fn hit_test(&self, line: usize, col: u16) -> Option<usize> {
        self.entries
            .iter()
            .find(|(l, range, _)| *l == line && range.contains(&col))
            .map(|(_, _, idx)| *idx)
    }
}

pub fn text_view(
    text: &str,
    current: usize,
    height: u16,
    highlight: Color,
    scroll_offset: Option<usize>,
) -> (Paragraph<'static>, WordMap, usize) {
    let mut word_index = 0;
    let mut current_line_index = 0;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut word_map = WordMap::default();

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

            let start_col = (col - token.len()) as u16;
            let end_col = col as u16;
            word_map
                .entries
                .push((line_num, start_col..end_col, word_index));

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

        if col < text_line.len() {
            spans.push(Span::raw(text_line[col..].to_string()));
        }

        lines.push(Line::from(spans));
    }

    let scroll =
        scroll_offset.unwrap_or_else(|| current_line_index.saturating_sub(height as usize / 2));

    let paragraph = Paragraph::new(lines).scroll((scroll as u16, 0));
    (paragraph, word_map, scroll)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_map_hit_test_finds_word() {
        let mut map = WordMap::default();
        map.entries.push((0, 0..5, 0));
        map.entries.push((0, 6..11, 1));
        map.entries.push((1, 0..3, 2));

        assert_eq!(map.hit_test(0, 0), Some(0));
        assert_eq!(map.hit_test(0, 4), Some(0));
        assert_eq!(map.hit_test(0, 6), Some(1));
        assert_eq!(map.hit_test(1, 1), Some(2));
    }

    #[test]
    fn word_map_hit_test_misses_whitespace() {
        let mut map = WordMap::default();
        map.entries.push((0, 0..5, 0));
        map.entries.push((0, 6..11, 1));

        assert_eq!(map.hit_test(0, 5), None);
        assert_eq!(map.hit_test(1, 0), None);
    }

    #[test]
    fn text_view_builds_word_map() {
        let text = "hello world\nfoo bar";
        let (_, map, _) = text_view(text, 0, 20, Color::Red, None);

        assert_eq!(map.hit_test(0, 0), Some(0));
        assert_eq!(map.hit_test(0, 4), Some(0));
        assert_eq!(map.hit_test(0, 6), Some(1));
        assert_eq!(map.hit_test(1, 0), Some(2));
        assert_eq!(map.hit_test(1, 4), Some(3));
        assert_eq!(map.hit_test(0, 5), None);
    }

    #[test]
    fn text_view_scroll_follows_current() {
        let lines: Vec<String> = (0..50).map(|i| format!("word{}", i)).collect();
        let text = lines.join("\n");
        let (_, _, scroll) = text_view(&text, 30, 10, Color::Red, None);
        assert_eq!(scroll, 25);
    }

    #[test]
    fn text_view_manual_scroll_overrides() {
        let text = "hello world\nfoo bar";
        let (_, _, scroll) = text_view(text, 0, 20, Color::Red, Some(5));
        assert_eq!(scroll, 5);
    }
}
