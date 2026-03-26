use crate::app::App;
use crate::core::updater::{UpdateState, CURRENT_VERSION};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_update(f: &mut Frame, app: &App) {
    let size = f.area();
    let theme = &app.config.theme;

    let outer_block = Block::default()
        .title(" [?] Update Clario ")
        .style(Style::default().bg(theme.bg()).fg(theme.text()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary()));

    let inner_area = outer_block.inner(size);
    f.render_widget(outer_block, size);

    // Layout: body (left list + right detail) | status bar | footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Body
            Constraint::Length(3), // Status bar
            Constraint::Length(3), // Footer
        ])
        .split(inner_area);

    // Body: left list (40%) and right detail (60%)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[0]);

    // --- LEFT PANE: Release List ---
    let left_block = Block::default()
        .title(" Available Versions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()));

    let left_inner = left_block.inner(body_chunks[0]);
    f.render_widget(left_block, body_chunks[0]);

    if app.update_releases.is_empty() {
        let msg = match &app.update_state {
            UpdateState::Checking => "Fetching releases from GitHub...",
            UpdateState::Error(_) => "Failed to load releases.",
            _ => "No releases found.",
        };
        let placeholder = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(theme.muted_text()),
        )))
        .alignment(Alignment::Center);
        f.render_widget(placeholder, left_inner);
    } else {
        let mut lines = Vec::new();
        lines.push(Line::from(""));
        for (i, release) in app.update_releases.iter().enumerate() {
            let is_selected = i == app.update_selected;
            let prefix = if is_selected { "▶ " } else { "  " };

            let badge = if release.is_current() {
                Span::styled(" [current] ", Style::default().fg(theme.safe()))
            } else if release.is_newer_than_current() {
                Span::styled(
                    " [new] ",
                    Style::default()
                        .fg(theme.warning())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(" [older] ", Style::default().fg(theme.muted_text()))
            };

            let row_style = if is_selected {
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.primary())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text())
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {}{}", prefix, release.tag_name), row_style),
                badge,
            ]));
        }

        let list = Paragraph::new(lines);
        f.render_widget(list, left_inner);
    }

    // --- RIGHT PANE: Selected Release Detail ---
    let right_block = Block::default()
        .title(" Release Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary()));

    let right_inner = right_block.inner(body_chunks[1]);
    f.render_widget(right_block, body_chunks[1]);

    let detail_lines = if let Some(release) = app.update_releases.get(app.update_selected) {
        let date = release
            .published_at
            .as_deref()
            .unwrap_or("Unknown")
            .get(..10)
            .unwrap_or("Unknown");

        let status_line = if release.is_current() {
            Line::from(Span::styled(
                "  ✓ This is your current version",
                Style::default().fg(theme.safe()),
            ))
        } else if release.is_newer_than_current() {
            Line::from(Span::styled(
                "  ↑ Newer than your current version",
                Style::default()
                    .fg(theme.warning())
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(
                "  ↓ Older than your current version",
                Style::default().fg(theme.muted_text()),
            ))
        };

        let notes = release.body.as_deref().unwrap_or("No release notes.");
        let note_lines: Vec<Line> = notes
            .lines()
            .map(|l| Line::from(Span::styled(format!("  {}", l), Style::default().fg(theme.text()))))
            .collect();

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Version : ", Style::default().fg(theme.muted_text())),
                Span::styled(
                    release.tag_name.clone(),
                    Style::default()
                        .fg(theme.primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Released: ", Style::default().fg(theme.muted_text())),
                Span::styled(date, Style::default().fg(theme.text())),
            ]),
            status_line,
            Line::from(""),
            Line::from(Span::styled(
                "  Release Notes:",
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];
        lines.extend(note_lines);
        lines
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Select a version from the list.",
                Style::default().fg(theme.muted_text()),
            )),
        ]
    };

    let detail = Paragraph::new(detail_lines).wrap(Wrap { trim: true });
    f.render_widget(detail, right_inner);

    // --- STATUS BAR ---
    let current_ver_label = format!(" Current: v{}  │  ", CURRENT_VERSION);
    let status_color = match &app.update_state {
        UpdateState::Done => theme.safe(),
        UpdateState::Error(_) => theme.danger(),
        UpdateState::Downloading | UpdateState::Checking => theme.warning(),
        _ => theme.muted_text(),
    };

    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(current_ver_label, Style::default().fg(theme.muted_text())),
        Span::styled(
            app.update_status.clone(),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.muted_text())),
    );
    f.render_widget(status_bar, main_chunks[1]);

    // --- FOOTER HELP ---
    let help_line = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc] ",
            Style::default()
                .fg(theme.warning())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Back  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [↑↓] ",
            Style::default()
                .fg(theme.primary())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Navigate  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [Enter] ",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Install Selected  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [r] ",
            Style::default()
                .fg(theme.safe())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Refresh", Style::default().fg(theme.muted_text())),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.muted_text())),
    );
    f.render_widget(help_line, main_chunks[2]);
}
