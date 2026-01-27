use ratatui::{backend::Backend, widgets, Frame};

pub fn draw_settings<B: Backend>(frame: &mut Frame) {
    let area = frame.area();
    let block = widgets::Block::default()
        .title("[Settings]")
        .borders(widgets::Borders::ALL);
    frame.render_widget(block, area);
}
