use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn draw_settings(f: &mut Frame, app: &App) {
    let size = f.area();
    let theme = &app.config.theme;

    // Apply global background based on Theme (TokyoNight / Catppuccin)
    let block = Block::default()
        .title(" [s] Settings ")
        .style(Style::default().bg(theme.bg()).fg(theme.text()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary()));

    let inner_area = block.inner(size);
    f.render_widget(block, size);

    // Split layout vertically (body and footer)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content (Left/Right panes)
            Constraint::Length(3), // Footer
        ])
        .split(inner_area);

    // Split main content horizontally (Left Menu 35% vs Right Context 65%)
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_chunks[0]);

    // --- LEFT PANE (Settings Menu) ---
    // Add title
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Padding/Title
            Constraint::Min(0),
        ])
        .split(panes[0]);

    let left_title = Paragraph::new(Span::styled(
        "Clario System Configuration",
        Style::default().add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center);
    f.render_widget(left_title, left_chunks[0]);

    // Construct Setting Rows (Left Panel)
    let settings_list = vec![
        ("Color Theme", format!("< {} >", app.config.theme.name()), 0),
        (
            "Archive Directory",
            app.config.archive_dir.to_string_lossy().to_string(),
            1,
        ),
        (
            "Safety Threshold",
            format!("< {} Days >", app.config.safety_threshold_days),
            2,
        ),
    ];

    let mut lines = Vec::new();
    // highlight rule: if normal mode, highlight according to settings_selected_index.
    // if picker mode, gray out everything on the left except index 1 which stays primary colored.
    for (label, value, index) in settings_list {
        let is_selected = app.settings_selected_index == index;

        let (label_style, value_style) = if is_selected && !app.is_dir_picker_open {
            (
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.primary())
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.primary())
                    .add_modifier(Modifier::BOLD),
            )
        } else if is_selected && app.is_dir_picker_open {
            // Let it stand out just a bit, but not fully active
            (
                Style::default().fg(theme.primary()),
                Style::default().fg(theme.primary()),
            )
        } else {
            (
                Style::default().fg(theme.text()),
                Style::default().fg(theme.primary()),
            )
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {:<20} ", label),
            label_style,
        )]));
        // Put values on the line below with extra indentation
        lines.push(Line::from(vec![Span::styled(
            format!("    {} ", value),
            value_style,
        )]));
        lines.push(Line::from("")); // Spacing
    }

    let left_block_widget = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.muted_text()));

    let settings_paragraph = Paragraph::new(lines).block(left_block_widget);
    f.render_widget(settings_paragraph, left_chunks[1]);

    // --- RIGHT PANE (Context / Target) ---

    if app.is_dir_picker_open {
        // Mode 1: Directory Picker
        let right_title = Paragraph::new(Line::from(vec![
            Span::styled(" Browse: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                app.dir_picker_path.to_string_lossy().to_string(),
                Style::default().fg(theme.accent()),
            ),
        ]))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.muted_text())),
        );

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(panes[1]);

        f.render_widget(right_title, right_chunks[0]);

        let mut rows = Vec::new();
        for (i, dir) in app.dir_picker_items.iter().enumerate() {
            let is_selected = i == app.dir_picker_selected;
            let row_style = if is_selected {
                Style::default()
                    .bg(theme.unselected_bg())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if is_selected { ">> " } else { "   " };
            let icon = if dir.name == ".." {
                "⤴️  "
            } else {
                "📁 "
            };

            rows.push(
                Row::new(vec![Cell::from(Span::styled(
                    format!("{}{}{}", prefix, icon, dir.name),
                    Style::default().fg(if is_selected {
                        theme.primary()
                    } else {
                        theme.text()
                    }),
                ))])
                .style(row_style),
            );
        }

        let table = Table::new(rows, [Constraint::Percentage(100)])
            .block(Block::default().padding(ratatui::widgets::Padding::new(2, 2, 1, 1)));

        f.render_widget(table, right_chunks[1]);
    } else {
        // Mode 2: Information Context Panel
        let (title, content) = match app.settings_selected_index {
            0 => (
                "🎨 Color Theme",
                "Select a color theme to personalize your Clario experience.\n\n\
                Themes gracefully update the entire terminal UI, changing accents, backgrounds, and warning colors.\n\
                Options include 'TokyoNight' and 'Catppuccin' variants."
            ),
            1 => (
                "📦 Archive Directory",
                "Choose where Clario saves compressed `.zip` backup files.\n\n\
                When you choose to 'Archive' files in the File Manager instead of deleting them, they are moved safely into this folder.\n\n\
                👉 Press [Enter] to open the Interactive Directory Picker."
            ),
            2 => (
                "🛡️ Safety Threshold",
                "The Safety Threshold defines the minimum age of cache files.\n\n\
                Folders modified within the last N days will be flagged cautiously (Yellow) to prevent slowing down active applications.\n\
                Adjust this to 0 for maximum storage clearing."
            ),
            _ => ("", ""),
        };

        let info_paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                title,
                Style::default()
                    .fg(theme.primary())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(content),
        ])
        .alignment(Alignment::Left)
        .block(Block::default().padding(ratatui::widgets::Padding::new(4, 4, 2, 2)))
        .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(info_paragraph, panes[1]);
    }

    // --- FOOTER HELP ---
    let help_spans = if app.is_dir_picker_open {
        vec![
            Span::styled(
                " [Esc] ",
                Style::default()
                    .fg(theme.warning())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Cancel  ", Style::default().fg(theme.muted_text())),
            Span::styled(
                " [↑/↓] ",
                Style::default()
                    .fg(theme.primary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Navigate  ", Style::default().fg(theme.muted_text())),
            Span::styled(
                " [Enter/→/←] ",
                Style::default()
                    .fg(theme.primary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Go In/Out  ", Style::default().fg(theme.muted_text())),
            Span::styled(
                " [Space] ",
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Confirm Selection", Style::default().fg(theme.muted_text())),
        ]
    } else {
        vec![
            Span::styled(
                " [Esc] ",
                Style::default()
                    .fg(theme.warning())
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
                " [←/→] ",
                Style::default()
                    .fg(theme.safe())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Change Value  ", Style::default().fg(theme.muted_text())),
            Span::styled(
                " [Enter] ",
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Select (For Directory)",
                Style::default().fg(theme.muted_text()),
            ),
        ]
    };

    let help_line = Paragraph::new(Line::from(help_spans))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme.muted_text())),
        );
    f.render_widget(help_line, main_chunks[1]);
}
