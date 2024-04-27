use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use crate::terminal::app::App;


/// Renders the user interface widgets.
pub fn render(_app: &mut App, frame: &mut Frame) {

    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counting",
        ))
        .block(
            Block::bordered()
                .title("Template")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .centered(),
        frame.size(),
    )
}