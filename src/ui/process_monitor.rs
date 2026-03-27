use crate::app::App;
use crate::core::process_scanner::{format_memory, format_uptime};
use crate::models::process_info::SuspicionSeverity;
use crate::ui::components::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

pub fn draw_process_monitor(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let theme = &app.config.theme;

    // Main vertical layout: header (3), content (min), status bar (1)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content (left/right panels)
            Constraint::Length(1), // Status bar / footer
        ])
        .split(size);

    // ==========================================
    // HEADER
    // ==========================================
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Process Monitor ",
            Style::default()
                .fg(theme.danger())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("| {} processes", app.processes.len()),
            Style::default().fg(theme.muted_text()),
        ),
    ]))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary())),
    );
    f.render_widget(header, main_chunks[0]);

    // ==========================================
    // CONTENT: Left 65% + Right 35%
    // ==========================================
    let content_area = main_chunks[1];
    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(content_area);

    // ==========================================
    // LEFT PANEL: Process table
    // ==========================================
    let left_block = Block::default()
        .title(" Processes ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.primary()));

    let mut rows = Vec::new();
    for process in &app.processes {
        let row_color = match process.severity() {
            SuspicionSeverity::Clean => theme.text(),
            SuspicionSeverity::Warning => theme.warning(),
            SuspicionSeverity::Danger => theme.danger(),
        };

        let check_prefix = if process.is_selected { "[x] " } else { "[ ] " };
        let name_cell = format!("{}{}", check_prefix, process.name);

        rows.push(
            Row::new(vec![
                Cell::from(name_cell).style(Style::default().fg(row_color)),
                Cell::from(process.pid.to_string()).style(Style::default().fg(row_color)),
                Cell::from(format!("{:.1}%", process.cpu_usage))
                    .style(Style::default().fg(row_color)),
                Cell::from(format_memory(process.memory_bytes))
                    .style(Style::default().fg(row_color)),
            ])
        );
    }

    let process_table = Table::new(
        rows,
        [
            Constraint::Percentage(40), // Name
            Constraint::Percentage(15), // PID
            Constraint::Percentage(20), // CPU%
            Constraint::Percentage(25), // RAM
        ],
    )
    .header(
        Row::new(vec!["Name", "PID", "CPU%", "RAM"])
            .style(
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.primary())
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(left_block)
    .row_highlight_style(
        Style::default()
            .bg(theme.primary())
            .fg(theme.bg()),
    )
    .highlight_symbol(">> ");

    f.render_stateful_widget(process_table, panel_chunks[0], &mut app.process_table_state);

    // ==========================================
    // RIGHT PANEL: Process detail
    // ==========================================
    let right_block = Block::default()
        .title(" Process Detail ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.secondary()));

    if app.processes.is_empty() {
        f.render_widget(
            Paragraph::new("No processes loaded. Press 'r' to refresh.")
                .style(Style::default().fg(theme.muted_text()))
                .block(right_block),
            panel_chunks[1],
        );
    } else {
        let proc = &app.processes[app.selected_process_index];

        let exe_str = match &proc.exe_path {
            Some(path) => path.to_string_lossy().to_string(),
            None => "Unknown (SIP protected)".to_string(),
        };
        let uid_str = match proc.user_id {
            Some(uid) => uid.to_string(),
            None => "Unknown".to_string(),
        };
        let parent_str = match proc.parent_pid {
            Some(ppid) => ppid.to_string(),
            None => "None".to_string(),
        };

        let mut detail_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Name:       ", Style::default().fg(theme.muted_text())),
                Span::styled(proc.name.clone(), Style::default().fg(theme.text())),
            ]),
            Line::from(vec![
                Span::styled("PID:        ", Style::default().fg(theme.muted_text())),
                Span::styled(proc.pid.to_string(), Style::default().fg(theme.text())),
            ]),
            Line::from(vec![
                Span::styled("Executable: ", Style::default().fg(theme.muted_text())),
                Span::styled(exe_str, Style::default().fg(theme.text())),
            ]),
            Line::from(vec![
                Span::styled("CPU Usage:  ", Style::default().fg(theme.muted_text())),
                Span::styled(
                    format!("{:.1}%", proc.cpu_usage),
                    Style::default().fg(theme.text()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Memory:     ", Style::default().fg(theme.muted_text())),
                Span::styled(
                    format_memory(proc.memory_bytes),
                    Style::default().fg(theme.text()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Owner UID:  ", Style::default().fg(theme.muted_text())),
                Span::styled(uid_str, Style::default().fg(theme.text())),
            ]),
            Line::from(vec![
                Span::styled("Parent PID: ", Style::default().fg(theme.muted_text())),
                Span::styled(parent_str, Style::default().fg(theme.text())),
            ]),
            Line::from(vec![
                Span::styled("Uptime:     ", Style::default().fg(theme.muted_text())),
                Span::styled(
                    format_uptime(proc.run_time_secs),
                    Style::default().fg(theme.text()),
                ),
            ]),
            Line::from(""),
        ];

        if !proc.suspicion_flags.is_empty() {
            detail_lines.push(Line::from(Span::styled(
                "--- Why Suspicious ---",
                Style::default()
                    .fg(theme.warning())
                    .add_modifier(Modifier::BOLD),
            )));
            for flag in &proc.suspicion_flags {
                detail_lines.push(Line::from(Span::styled(
                    format!("  - {}", flag.display_reason()),
                    Style::default().fg(theme.danger()),
                )));
            }
        }

        f.render_widget(
            Paragraph::new(detail_lines)
                .block(right_block)
                .wrap(ratatui::widgets::Wrap { trim: false }),
            panel_chunks[1],
        );
    }

    // ==========================================
    // STATUS BAR / FOOTER
    // ==========================================
    let status_msg = app
        .kill_status_message
        .as_deref()
        .unwrap_or("")
        .to_string();

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(status_msg, Style::default().fg(theme.danger())),
        Span::styled(
            "  [Space] Select  [x] Kill  [r] Refresh  [Esc/d] Dashboard",
            Style::default().fg(theme.muted_text()),
        ),
    ]))
    .alignment(Alignment::Left);
    f.render_widget(footer, main_chunks[2]);

    // ==========================================
    // KILL CONFIRM MODAL (rendered on top)
    // ==========================================
    if app.show_kill_confirm {
        let modal_area = centered_rect(50, 9, size);
        f.render_widget(Clear, modal_area);

        let selected_count = app.processes.iter().filter(|p| p.is_selected).count();

        let modal_block = Block::default()
            .title(" Confirm Kill ")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(theme.danger()).bg(theme.bg()))
            .style(Style::default().bg(theme.bg()));

        let inner = modal_block.inner(modal_area);
        f.render_widget(modal_block, modal_area);

        let modal_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Confirmation text
                Constraint::Length(3), // Buttons
                Constraint::Min(0),
            ])
            .split(inner);

        let confirm_text = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Kill {} selected process(es)?", selected_count),
                Style::default()
                    .fg(theme.text())
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .alignment(Alignment::Center);
        f.render_widget(confirm_text, modal_chunks[0]);

        // 3 buttons: Cancel | Graceful Kill | Force Kill
        let btn_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(modal_chunks[1]);

        // Cancel button (index 0)
        let cancel_style = if app.kill_confirm_selected == 0 {
            Style::default()
                .fg(theme.bg())
                .bg(theme.muted_text())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted_text())
        };
        let cancel_border = if app.kill_confirm_selected == 0 {
            theme.text()
        } else {
            theme.muted_text()
        };

        // Graceful Kill button (index 1)
        let graceful_style = if app.kill_confirm_selected == 1 {
            Style::default()
                .fg(theme.bg())
                .bg(theme.warning())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.warning())
        };
        let graceful_border = if app.kill_confirm_selected == 1 {
            theme.warning()
        } else {
            theme.muted_text()
        };

        // Force Kill button (index 2)
        let force_style = if app.kill_confirm_selected == 2 {
            Style::default()
                .fg(theme.bg())
                .bg(theme.danger())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.danger())
        };
        let force_border = if app.kill_confirm_selected == 2 {
            theme.danger()
        } else {
            theme.muted_text()
        };

        let btn_cancel = Paragraph::new(" Cancel ")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(cancel_border)),
            )
            .style(cancel_style);

        let btn_graceful = Paragraph::new(" Graceful Kill ")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(graceful_border)),
            )
            .style(graceful_style);

        let btn_force = Paragraph::new(" Force Kill ")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(force_border)),
            )
            .style(force_style);

        f.render_widget(btn_cancel, btn_layout[0]);
        f.render_widget(btn_graceful, btn_layout[1]);
        f.render_widget(btn_force, btn_layout[2]);
    }
}
