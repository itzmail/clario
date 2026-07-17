use crate::tui::app::{BrowserScreen, Overlay};
use crate::utils::size::format_size;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

const BAR_WIDTH: usize = 26;

pub fn draw(frame: &mut Frame, area: Rect, screen: &BrowserScreen) {
    let status = if screen.status.done {
        format!(
            "{} entries  |  {}",
            screen.entries.len(),
            format_size(screen.status.bytes)
        )
    } else {
        format!(
            "Scanning: {} files, {} dirs, {}",
            screen.status.files,
            screen.status.dirs,
            format_size(screen.status.bytes)
        )
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled(status, Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(screen.root.display().to_string(), Style::default().add_modifier(Modifier::BOLD))),
    ]);
    frame.render_widget(header, Rect { height: 2, ..area });

    let table_area = Rect { y: area.y + 3, height: area.height.saturating_sub(4), ..area };

    let max_size = screen.entries.iter().map(|f| f.size_bytes).max().unwrap_or(1).max(1);

    let rows: Vec<Row> = screen
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let active = i == screen.selected;
            let checked = screen.multi_selected.contains(&i);

            let pct = entry.size_bytes as f64 / max_size as f64;
            let filled = (pct * BAR_WIDTH as f64).round() as usize;
            let bar = format!("{}{}", "█".repeat(filled), "▒".repeat(BAR_WIDTH - filled));
            let pct_text = if entry.size_bytes == 0 {
                "0.0%".to_string()
            } else if pct * 100.0 < 0.1 {
                "< 0.1%".to_string()
            } else {
                format!("{:.1}%", pct * 100.0)
            };

            let cursor = if active { "▶ " } else { "  " };
            let check = if checked { "●" } else { "○" };
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let name = format!("{icon} {}", entry.name);

            let cells = vec![
                format!("{cursor}{check} {:>2}.", i + 1),
                bar,
                pct_text,
                name,
                format_size(entry.size_bytes),
            ];

            let style = if active {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if checked {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            Row::new(cells).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(BAR_WIDTH as u16),
            Constraint::Length(8),
            Constraint::Fill(1),
            Constraint::Length(12),
        ],
    )
    .block(Block::default().borders(Borders::NONE));

    frame.render_widget(table, table_area);

    let footer = Paragraph::new(Line::from(Span::styled(
        "↑↓ | Space Select | Enter | O Open | P Preview | ⌫ Del | Esc Back | Q Quit",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(footer, Rect { y: area.y + area.height.saturating_sub(1), height: 1, ..area });

    draw_overlay(frame, area, &screen.overlay);
}

fn draw_overlay(frame: &mut Frame, area: Rect, overlay: &Overlay) {
    match overlay {
        Overlay::None => {}
        Overlay::ConfirmDelete { targets, total_bytes, blocked } => {
            let box_area = centered_rect(area, 60, 40);
            frame.render_widget(Clear, box_area);

            let mut lines = vec![
                Line::from(Span::styled(
                    format!("Delete {} item(s)?  ({})", targets.len(), format_size(*total_bytes)),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];
            for t in targets.iter().take(8) {
                lines.push(Line::from(format!("  {}", t.display())));
            }
            if targets.len() > 8 {
                lines.push(Line::from(format!("  ... and {} more", targets.len() - 8)));
            }
            if !blocked.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("{} item(s) protected, will be skipped", blocked.len()),
                    Style::default().fg(Color::Yellow),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Y confirm  |  N/Esc cancel", Style::default().fg(Color::DarkGray))));

            let block = Block::default().borders(Borders::ALL).title(" Confirm Delete ");
            frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(block), box_area);
        }
        Overlay::Preview { title, lines } => {
            let box_area = centered_rect(area, 70, 60);
            frame.render_widget(Clear, box_area);
            let text: Vec<Line> = lines.iter().map(|l| Line::from(l.clone())).collect();
            let block = Block::default().borders(Borders::ALL).title(format!(" {title} "));
            frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }).block(block), box_area);
        }
        Overlay::Message(msg) => {
            let box_area = centered_rect(area, 50, 20);
            frame.render_widget(Clear, box_area);
            let block = Block::default().borders(Borders::ALL);
            frame.render_widget(Paragraph::new(msg.as_str()).wrap(Wrap { trim: false }).block(block), box_area);
        }
    }
}

fn centered_rect(area: Rect, pct_x: u16, pct_y: u16) -> Rect {
    let w = area.width * pct_x / 100;
    let h = area.height * pct_y / 100;
    Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    }
}
