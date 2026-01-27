use ratatui::{
  backend::Backend,
  Frame,
  widgets,
};

pub fn draw_file_manager<B:Backend>(frame: &mut Frame) {
  let area = frame.area();
  let block = widgets::Block::default().title("[File Manager]").borders(widgets::Borders::ALL);
  frame.render_widget(block, area);
}
