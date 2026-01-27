use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_dashboard<B: Backend>(f: &mut Frame) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.area());

    let block = Block::default().title("Clario").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);

    let text = Paragraph::new(Text::from(Span::raw("Selamat data di Clario!")));
    f.render_widget(text, chunks[0]);
}
