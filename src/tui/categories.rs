use crate::tui::app::CategoriesScreen;
use crate::utils::size::format_size;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

const BAR_WIDTH: usize = 26;

pub fn draw(frame: &mut Frame, area: Rect, screen: &CategoriesScreen) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Analyze Disk", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("Select a location to explore", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(title, Rect { height: 1, ..area });

    let table_area = Rect { y: area.y + 2, height: area.height.saturating_sub(2), ..area };

    let max_size = screen.rows.iter().filter_map(|r| r.size_bytes).max().unwrap_or(1).max(1);

    let rows: Vec<Row> = screen
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let active = i == screen.selected;
            let (bar, pct, size_text) = match row.size_bytes {
                Some(bytes) => {
                    let pct = bytes as f64 / max_size as f64;
                    let filled = (pct * BAR_WIDTH as f64).round() as usize;
                    let bar = format!("{}{}", "█".repeat(filled), "▒".repeat(BAR_WIDTH - filled));
                    let pct_text = if bytes == 0 {
                        "0.0%".to_string()
                    } else if pct * 100.0 < 0.1 {
                        "< 0.1%".to_string()
                    } else {
                        format!("{:.1}%", pct * 100.0)
                    };
                    (bar, pct_text, format_size(bytes))
                }
                None => (String::new(), "--".to_string(), "pending...".to_string()),
            };

            let prefix = if active { "▶ " } else { "  " };
            let cells = vec![
                format!("{prefix}{:>2}.", i + 1),
                bar,
                pct,
                row.label.to_string(),
                size_text,
            ];

            let style = if active {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(cells).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(BAR_WIDTH as u16),
            Constraint::Length(8),
            Constraint::Fill(1),
            Constraint::Length(12),
        ],
    )
    .block(Block::default().borders(Borders::NONE));

    frame.render_widget(table, table_area);

    let footer = Paragraph::new(Line::from(Span::styled(
        "↑↓  |  Enter  |  Esc/Q Quit",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(footer, Rect { y: area.y + area.height.saturating_sub(1), height: 1, ..area });
}
