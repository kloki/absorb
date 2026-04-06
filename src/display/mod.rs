mod footer;
mod text_view;
mod word;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
};
pub use text_view::WordMap;

pub struct ViewState<'a> {
    pub words: &'a [String],
    pub text: &'a str,
    pub current: usize,
    pub wpm: u32,
    pub playing: bool,
    pub split_view: bool,
    pub highlight: Color,
    pub scroll_offset: Option<usize>,
}

#[derive(Default)]
pub struct DrawResult {
    pub text_pane: Option<Rect>,
    pub word_map: WordMap,
    pub scroll: usize,
}

pub fn draw(frame: &mut Frame, state: &ViewState) -> DrawResult {
    let ViewState {
        words,
        text,
        current,
        wpm,
        playing,
        split_view,
        highlight,
        scroll_offset,
    } = *state;
    let area = frame.area();

    let outer = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let mut result = DrawResult {
        text_pane: None,
        word_map: WordMap::default(),
        scroll: 0,
    };

    if split_view {
        let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(outer[0]);
        draw_word_pane(frame, cols[0], words, current, highlight);
        let (tv, word_map, scroll, _) =
            text_view::text_view(text, current, cols[1].height, highlight, scroll_offset);
        frame.render_widget(tv, cols[1]);
        result.text_pane = Some(cols[1]);
        result.word_map = word_map;
        result.scroll = scroll;
    } else {
        draw_word_pane(frame, outer[0], words, current, highlight);
    }

    frame.render_widget(footer::progress(current, words.len()), outer[1]);

    let footer_layout =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).split(outer[2]);
    frame.render_widget(footer::controls(), footer_layout[0]);
    frame.render_widget(
        footer::status(current, words.len(), wpm, playing),
        footer_layout[1],
    );

    result
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
        frame.render_widget(
            word::word(area.width, &words[current], highlight),
            layout[2],
        );
    } else {
        frame.render_widget(word::end(), layout[2]);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

    pub fn render_to_string(widget: impl Widget, width: u16, height: u16) -> String {
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
}
