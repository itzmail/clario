use crate::models::config::AppTheme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Fungsi kecil untuk membuat `Rect` di tengah layer di atas layar asli dengan tinggi & lebar absolute u16.
pub fn centered_rect(width: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(r.height.saturating_sub(height) / 2),
            Constraint::Length(height), // FIXED HEIGHT untuk Modal
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(r.width.saturating_sub(width) / 2),
            Constraint::Length(width), // FIXED WIDTH for Modal
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw_exit_modal(f: &mut Frame, selected: u8, theme: &AppTheme) {
    let area = f.area();
    // Modal akan mengambil kotak ukuran fix 60x12 di tengah layar
    let popup_area = centered_rect(60, 12, area);

    // Render widget Clear (menjebol/mengosongkan UI di belakangnya)
    f.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title(" Quit Clario ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Thick)
        .border_style(Style::default().fg(theme.danger()).bg(theme.bg()))
        .style(Style::default().bg(theme.bg()));

    let inner_popup = popup_block.inner(popup_area);
    f.render_widget(popup_block, popup_area);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Tinggi teks
            Constraint::Length(3), // Tinggi Tombol
            Constraint::Min(0),
        ])
        .split(inner_popup);

    let confirm_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Are you sure you want to exit? 😢",
            Style::default()
                .fg(theme.text())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    f.render_widget(
        Paragraph::new(confirm_text).alignment(Alignment::Center),
        popup_chunks[0],
    );

    // Render Buttons
    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),      // Margin kiri
            Constraint::Percentage(50), // Yes button
            Constraint::Percentage(50), // No button
            Constraint::Length(2),      // Margin kanan
        ])
        .split(popup_chunks[1]);

    let (yes_border, yes_style) = if selected == 0 {
        (
            theme.danger(),
            Style::default()
                .bg(theme.danger())
                .fg(theme.bg())
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (theme.muted_text(), Style::default().fg(theme.muted_text()))
    };

    let (no_border, no_style) = if selected == 1 {
        (
            theme.safe(),
            Style::default()
                .bg(theme.safe())
                .fg(theme.bg())
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (theme.muted_text(), Style::default().fg(theme.muted_text()))
    };

    let yes_btn = Paragraph::new(" [Y/Enter] Yes ")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(yes_border)),
        )
        .style(yes_style);

    let no_btn = Paragraph::new(" [N/Esc] Wait, not yet ")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(no_border)),
        )
        .style(no_style);

    f.render_widget(yes_btn, button_layout[1]);
    f.render_widget(no_btn, button_layout[2]);
}
