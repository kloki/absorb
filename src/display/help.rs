use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

pub fn help_popup() -> Paragraph<'static> {
    let key_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::White);

    let lines: Vec<Line<'static>> = vec![
        Line::from(""),
        binding("SPACE", "Play / Pause", key_style, desc_style),
        binding("\u{2190}", "Back one word", key_style, desc_style),
        binding("\u{2192}", "Forward one word", key_style, desc_style),
        binding(
            "\u{2191}",
            "Increase speed (+25 WPM)",
            key_style,
            desc_style,
        ),
        binding(
            "\u{2193}",
            "Decrease speed (\u{2212}25 WPM)",
            key_style,
            desc_style,
        ),
        binding("v", "Toggle split-view", key_style, desc_style),
        binding("b", "Toggle big text", key_style, desc_style),
        binding("c", "Cycle highlight color", key_style, desc_style),
        binding("r", "Restart", key_style, desc_style),
        binding("q / Esc", "Quit", key_style, desc_style),
        binding("h", "Show this help", key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("Split-view only:", Style::default())),
        binding("Scroll", "Scroll text pane", key_style, desc_style),
        binding("Click", "Jump to word", key_style, desc_style),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default())),
    ];

    Paragraph::new(lines).block(
        Block::bordered()
            .title(" Keybindings ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .padding(Padding::horizontal(2))
            .style(Style::default().fg(Color::White)),
    )
}

fn binding<'a>(key: &'a str, desc: &'a str, key_style: Style, desc_style: Style) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<12}", key), key_style),
        Span::styled(desc, desc_style),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_rect_centers_correctly() {
        let area = Rect::new(0, 0, 100, 40);
        let r = centered_rect(60, 18, area);
        assert_eq!(r.x, 20);
        assert_eq!(r.y, 11);
        assert_eq!(r.width, 60);
        assert_eq!(r.height, 18);
    }

    #[test]
    fn centered_rect_clamps_to_area() {
        let area = Rect::new(0, 0, 30, 10);
        let r = centered_rect(60, 18, area);
        assert_eq!(r.width, 30);
        assert_eq!(r.height, 10);
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
    }

    #[test]
    fn help_popup_renders_without_panic() {
        use crate::display::tests::render_to_string;
        let rendered = render_to_string(help_popup(), 60, 20);
        assert!(rendered.contains("Keybindings"));
        assert!(rendered.contains("SPACE"));
        assert!(rendered.contains("Press any key to close"));
    }
}
