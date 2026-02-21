use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Margin},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_dashboard<B: Backend>(f: &mut Frame) {
    let area = f.area();
    let container = Block::default().borders(Borders::ALL);
    f.render_widget(container, area);
    let padded_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    let inner_area = padded_area;
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Length(4), // System Info
            ]
            .as_ref(),
        )
        .split(inner_area);
    // Header
    let header = Paragraph::new("🧹 Clario v1.0");
    f.render_widget(header, chunks[0]);
    // System Info
    let system_info = Paragraph::new(Text::from(vec![
        Line::from(Span::raw(
            "💻 System: MacBook Pro M2 🟢  Storage: 156.3GB/256GB (61%)",
        )),
        Line::from(Span::raw(
            "📊 Last Clean: 2 days ago  📁 Files Deleted: 142",
        )),
        Line::from(Span::raw(
            "🗑️ Space Freed: 8.1GB  ⚡ Performance Score: 85/100",
        )),
    ]));
    f.render_widget(system_info, chunks[1]);
}
