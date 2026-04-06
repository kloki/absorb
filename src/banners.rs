use ratatui::{
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
};

const HEADER: &str = r"  ┓      ┓
┏┓┣┓┏┏┓┏┓┣┓
┗┻┗┛┛┗┛┛ ┗┛";

pub fn header() -> Paragraph<'static> {
    Paragraph::new(Text::raw(HEADER))
        .centered()
        .style(Style::default().fg(Color::Cyan))
}
