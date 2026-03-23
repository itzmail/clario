use crate::app::App;
use crate::ui::components::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

fn format_last_used(last_accessed: Option<chrono::DateTime<chrono::Local>>) -> String {
    match last_accessed {
        None => "Unknown".to_string(),
        Some(dt) => {
            let days = (chrono::Local::now() - dt).num_days();
            if days < 1 {
                "Today".to_string()
            } else if days < 7 {
                format!("{}d ago", days)
            } else if days < 30 {
                format!("{}w ago", days / 7)
            } else if days < 365 {
                format!("{}mo ago", days / 30)
            } else {
                format!("{}y ago", days / 365)
            }
        }
    }
}

pub fn draw_app_uninstaller(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let theme = &app.config.theme;

    // Layout Utama: Header (atas), Main Content (tengah), Footer (bawah)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Split Content
            Constraint::Length(2), // Footer Keybindings
        ])
        .split(size);

    // 1. Header
    let header_text = vec![Line::from(vec![
        Span::styled(
            "📦 Deep App Uninstaller ",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" | {} Apps Found", app.apps.len()),
            Style::default().fg(theme.muted_text()),
        ),
    ])];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.accent())),
        );
    f.render_widget(header, chunks[0]);

    // Jika sedang nge-scan, tampilkan layar loading
    if app.apps.is_empty() && app.is_scanning {
        let scan_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(5),
                Constraint::Percentage(40),
            ])
            .split(chunks[1]);

        let loading_text = format!(
            "⏳ Searching Applications and analyzing Library metadata...\n\nCurrently checking: {}",
            app.scan_progress_text
        );

        let loading = Paragraph::new(loading_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.warning())),
            );

        f.render_widget(loading, scan_layout[1]);
        return;
    }

    // MAIN SPLIT: Kiri (Daftar Aplikasi) | Kanan (Detail File Library)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // 40% Kiri
            Constraint::Percentage(60), // 60% Kanan
        ])
        .split(chunks[1]);

    // ================== PANEL KIRI: DAFTAR APLIKASI ==================
    let mut app_rows = Vec::new();

    for (i, current_app) in app.apps.iter().enumerate() {
        let is_selected_row = app.selected_app_index == i;

        // Simbol Centang (Space)
        let check_symbol = if current_app.is_selected {
            Span::styled(
                "[X] ",
                Style::default()
                    .fg(theme.danger())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("[ ] ", Style::default().fg(theme.muted_text()))
        };

        let name_span = Span::styled(
            current_app.name.clone(),
            if is_selected_row {
                Style::default().fg(theme.bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text())
            },
        );

        let size_str = format!(
            "{:.2} MB",
            current_app.total_size_bytes as f64 / 1_048_576.0
        );
        let size_span = Span::styled(
            size_str,
            if is_selected_row {
                Style::default().fg(theme.bg())
            } else {
                Style::default().fg(theme.warning())
            },
        );

        let last_used_str = format_last_used(current_app.last_accessed);
        let last_used_span = Span::styled(
            last_used_str,
            if is_selected_row {
                Style::default().fg(theme.bg())
            } else {
                Style::default().fg(theme.muted_text())
            },
        );

        app_rows.push(Row::new(vec![
            Cell::from(check_symbol),
            Cell::from(name_span),
            Cell::from(size_span),
            Cell::from(last_used_span),
        ]));
    }

    let left_block = Block::default()
        .title(" Installed Applications ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary()));

    let app_table = Table::new(
        app_rows,
        [
            Constraint::Length(4),      // Ceklis
            Constraint::Percentage(50), // Nama App
            Constraint::Percentage(20), // Total Size
            Constraint::Percentage(30), // Last Used
        ],
    )
    .header(
        Row::new(vec!["Sel", "Application Name", "Total Size", "Last Used"])
            .style(
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.primary())
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(left_block)
    .row_highlight_style(Style::default().bg(theme.unselected_bg()))
    .highlight_symbol(">> ");

    f.render_stateful_widget(app_table, main_chunks[0], &mut app.app_table_state);

    // ================== PANEL KANAN: DETAIL RELASI FILE ==================
    let right_block = Block::default()
        .title(" Associated Junk & Library Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()));

    if let Some(selected) = app.apps.get(app.selected_app_index) {
        let mut detail_rows = Vec::new();

        // Rekap ukuran
        let total_mb = selected.total_size_bytes as f64 / 1_048_576.0;
        let app_mb = selected.app_size_bytes as f64 / 1_048_576.0;
        let junk_mb = (selected
            .total_size_bytes
            .saturating_sub(selected.app_size_bytes)) as f64
            / 1_048_576.0;

        detail_rows.push(Row::new(vec![
            Cell::from(Span::styled(
                "📦 Main Binary App:",
                Style::default().fg(theme.safe()),
            )),
            Cell::from(Span::styled(
                format!("{:.2} MB", app_mb),
                Style::default().fg(theme.text()),
            )),
        ]));

        detail_rows.push(Row::new(vec![
            Cell::from(Span::styled(
                "🗑️ Associated Junk:",
                Style::default().fg(theme.danger()),
            )),
            Cell::from(Span::styled(
                format!("{:.2} MB", junk_mb),
                Style::default().fg(theme.text()),
            )),
        ]));

        detail_rows.push(
            Row::new(vec![
                Cell::from(Span::styled(
                    "Total Reclaimed Space:",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    format!("{:.2} MB", total_mb),
                    Style::default()
                        .fg(theme.text())
                        .add_modifier(Modifier::BOLD),
                )),
            ])
            .bottom_margin(1),
        );

        // List File Path Relasi Library
        for file in &selected.related_files {
            let path_str = file.path.to_string_lossy().to_string();
            let size_str = format!("{:.2} MB", file.size_bytes as f64 / 1_048_576.0);

            detail_rows.push(Row::new(vec![
                Cell::from(Span::styled(
                    path_str,
                    Style::default().fg(theme.muted_text()),
                )),
                Cell::from(Span::styled(size_str, Style::default().fg(theme.warning()))),
            ]));
        }

        let detail_table = Table::new(
            detail_rows,
            [Constraint::Percentage(75), Constraint::Percentage(25)],
        )
        .block(right_block);

        f.render_widget(detail_table, main_chunks[1]);
    } else {
        // Tampilkan Kosong jika belum ada apa-apa
        f.render_widget(
            Paragraph::new("No app selected").block(right_block),
            main_chunks[1],
        );
    }

    // ================== FOOTER / KEYBINDINGS ==================
    let help_line = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc] ",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Dashboard  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [↑/↓] ",
            Style::default()
                .fg(theme.primary())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Navigate  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [Space] ",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Select App  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [Del/x] ",
            Style::default()
                .fg(theme.danger())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Deep Uninstall", Style::default().fg(theme.muted_text())),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.muted_text())),
    );

    f.render_widget(help_line, chunks[2]);

    // ================== LAYAR KONFIRMASI DELETE ==================
    if app.is_deleting {
        // Overlay progress saat penghapusan berlangsung
        use ratatui::widgets::Clear;
        let popup_area = centered_rect(62, 10, size);
        f.render_widget(Clear, popup_area);

        let deleting_block = Block::default()
            .title(" 🗑️  Uninstalling... ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(Style::default().fg(theme.danger()))
            .style(Style::default().bg(theme.bg()));

        let inner = deleting_block.inner(popup_area);
        f.render_widget(deleting_block, popup_area);

        let progress_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Removing files... please wait",
                Style::default()
                    .fg(theme.text())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                app.delete_progress_text.as_str(),
                Style::default().fg(theme.muted_text()),
            )),
        ];

        f.render_widget(
            Paragraph::new(progress_text).alignment(Alignment::Center),
            inner,
        );
        return;
    }

    if app.show_delete_confirm {
        use ratatui::widgets::Clear;
        let popup_area = centered_rect(62, 12, size);
        f.render_widget(Clear, popup_area);

        // Hitung total yang akan dihapus
        let selected_count = app.apps.iter().filter(|a| a.is_selected).count();
        let total_mb: f64 = app
            .apps
            .iter()
            .filter(|a| a.is_selected)
            .map(|a| a.total_size_bytes as f64 / 1_048_576.0)
            .sum();

        let popup_block = Block::default()
            .title(" ⚠️  Confirm Deep Uninstall ")
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(Style::default().fg(theme.danger()).bg(theme.bg()))
            .style(Style::default().bg(theme.bg()));

        let inner = popup_block.inner(popup_area);
        f.render_widget(popup_block, popup_area);

        let confirm_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Teks konfirmasi
                Constraint::Length(3), // Tombol
                Constraint::Min(0),
            ])
            .split(inner);

        let confirm_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("This will permanently delete {} app(s)", selected_count),
                Style::default()
                    .fg(theme.text())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(
                    "and ALL associated library files ({:.1} MB total).",
                    total_mb
                ),
                Style::default().fg(theme.warning()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "This action CANNOT be undone!",
                Style::default()
                    .fg(theme.danger())
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        f.render_widget(
            Paragraph::new(confirm_text).alignment(Alignment::Center),
            confirm_chunks[0],
        );

        // Tombol Yes / No
        let btn_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
                Constraint::Length(2),
            ])
            .split(confirm_chunks[1]);

        let (confirm_border, confirm_style) = if app.delete_confirm_selected == 0 {
            (
                theme.danger(),
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.danger())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (theme.muted_text(), Style::default().fg(theme.muted_text()))
        };

        let (cancel_border, cancel_style) = if app.delete_confirm_selected == 1 {
            (
                theme.text(),
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.text())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (theme.muted_text(), Style::default().fg(theme.muted_text()))
        };

        let btn_confirm = Paragraph::new(" [Y/Enter] Confirm ")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(confirm_border)),
            )
            .style(confirm_style);

        let btn_cancel = Paragraph::new(" [N/Esc] Cancel ")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(cancel_border)),
            )
            .style(cancel_style);

        f.render_widget(btn_confirm, btn_layout[1]);
        f.render_widget(btn_cancel, btn_layout[2]);
    }
}
