use crate::app::App;
use sysinfo::System;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

/// Renders the main dashboard of the application.
///
/// This function is responsible for building the UI layout and populating it with
/// widgets. It uses `ratatui`'s `Layout` to divide the screen into logical sections
/// (Header, System Info, Menu, and Footer), similar to flexbox in web development.
///
/// # Arguments
///
/// * `f` - The mutable `Frame` where widgets are rendered. It represents the current terminal screen.
/// * `app` - Reference to the App state containing menu selection, system info, and theme.
pub fn draw_dashboard(f: &mut Frame, app: &App) {
    let selected_menu = app.selected_menu;
    let sys = &app.sys;
    let theme = &app.config.theme;
    let size = f.area();

    // ==========================================
    // 1. MAIN LAYOUT DEFINITION
    // ==========================================
    // We split the vertical space into 4 main sections:
    // - Header: Fixed 3 lines height
    // - System Info: Fixed 6 lines height
    // - Main Content: Takes up all remaining space (Constraint::Min(0))
    // - Footer: Fixed 3 lines height
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    // ==========================================
    // SECTION 1: HEADER
    // ==========================================
    // The Block struct acts as a container. We give it rounded borders and apply
    // the primary theme color.
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.primary()));

    // Paragraph is used to display text. We use `Line` and `Span` to apply
    // different styles (bold, colors) to different parts of the same text line.
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " 🧹 Clario ",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "v1.0 - Terminal System Cleaner",
            Style::default().fg(theme.muted_text()),
        ),
    ])])
    .block(header_block)
    .alignment(Alignment::Center);

    // Render the header widget into the first chunk of our main layout
    f.render_widget(header, main_layout[0]);

    // ==========================================
    // SECTION 2: SYSTEM INFORMATION
    // ==========================================
    let sys_info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.primary()))
        .title(" System Overview ");

    // Fetch dynamic OS information
    let os_name = System::long_os_version().unwrap_or_else(|| "Unknown OS".to_string());

    // Calculate RAM usage (Bytes to Gigabytes conversion)
    // `f64` is used for floating-point arithmetic to get precise decimal values.
    let total_mem_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let used_mem_gb = sys.used_memory() as f64 / 1_073_741_824.0;

    let mem_percentage = if total_mem_gb > 0.0 {
        (used_mem_gb / total_mem_gb) * 100.0
    } else {
        0.0
    };

    // Determine color based on memory usage threshold.
    // In Rust, `if` is an expression, meaning it can return a value which we
    // immediately bind to a variable. This is a very common Rust pattern.
    let mem_health_color = if mem_percentage > 85.0 {
        theme.danger()
    } else if mem_percentage > 70.0 {
        theme.warning()
    } else {
        theme.safe()
    };

    // Note: Parsing specific physical drive usage efficiently is complex with
    // multiple virtual volumes (e.g., APFS on macOS). We focus on RAM here as
    // a primary visual indicator of system overhead.
    let sys_info = Paragraph::new(vec![
        Line::from(vec![
            "💻 Host: ".bold(),
            os_name.fg(theme.primary()),
            "   |   🧠 RAM Usage: ".bold(),
            format!("{:.1}GB", used_mem_gb).fg(mem_health_color),
            "/".dark_gray(),
            format!("{:.1}GB ", total_mem_gb).fg(theme.secondary()),
            format!("({:.0}%)", mem_percentage).fg(mem_health_color),
        ]),
        Line::from(""), // Empty line for spacing
        Line::from(vec![
            "📈 Tip: ".bold().fg(theme.warning()),
            "Use the 'File Manager' to hunt down and delete large unused caches".fg(theme.text()),
        ]),
    ])
    .block(sys_info_block)
    .wrap(Wrap { trim: true });

    f.render_widget(sys_info, main_layout[1]);

    // ==========================================
    // SECTION 3: MAIN MENU (Split into 2 Columns)
    // ==========================================
    // We subdivide the third main layout chunk horizontally (like flex-direction: row)
    // Left side: Interactive Menu (60% width)
    // Right side: Statistics (40% width)
    let menu_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[2]);

    // --- Left Column: Action Menu ---
    let action_menu_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" 📋 Main Menu ")
        .border_style(Style::default().fg(theme.accent()));

    // Dynamic styling: Highlight the currently selected menu item.
    let m1_style = if selected_menu == 0 {
        Style::default()
            .fg(theme.bg())
            .bg(theme.primary())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.primary())
            .add_modifier(Modifier::BOLD)
    };

    let m2_style = if selected_menu == 1 {
        Style::default()
            .fg(theme.bg())
            .bg(theme.secondary())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.secondary())
            .add_modifier(Modifier::BOLD)
    };

    let m3_style = if selected_menu == 2 {
        Style::default()
            .fg(theme.bg())
            .bg(theme.accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD)
    };

    let m4_style = if selected_menu == 3 {
        Style::default()
            .fg(theme.bg())
            .bg(theme.danger())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.danger())
            .add_modifier(Modifier::BOLD)
    };

    // Prefix indicator for the active item
    let m1_prefix = if selected_menu == 0 { "▶ " } else { "  " };
    let m2_prefix = if selected_menu == 1 { "▶ " } else { "  " };
    let m3_prefix = if selected_menu == 2 { "▶ " } else { "  " };
    let m4_prefix = if selected_menu == 3 { "▶ " } else { "  " };

    let action_menu_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::raw(m1_prefix),
            Span::styled("🧹 [f] File Manager / Clean Files", m1_style),
        ]),
        Line::from(format!("      Smart cleanup - 2.1GB ready to delete").fg(theme.muted_text())),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::raw(m2_prefix),
            Span::styled("📦 [u] Deep App Uninstaller", m2_style),
        ]),
        Line::from(
            format!("      Eradicate applications & hidden library junk").fg(theme.muted_text()),
        ),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::raw(m3_prefix),
            Span::styled("⚙️  [s] Settings", m3_style),
        ]),
        Line::from(
            format!("      Configure cleanup rules and safety options").fg(theme.muted_text()),
        ),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::raw(m4_prefix),
            Span::styled("  [p] Process Monitor", m4_style),
        ]),
        Line::from("      Scan running processes & flag suspicious activity".fg(theme.muted_text())),
    ];

    let action_menu = Paragraph::new(action_menu_text).block(action_menu_block);
    f.render_widget(action_menu, menu_layout[0]);

    // --- Right Column: Quick Stats ---
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" 📊 Quick Stats ")
        .border_style(Style::default().fg(theme.secondary()));

    // Build real last_clean display from persistent stats
    let last_clean_text = match &app.config.stats.last_clean_date {
        Some(date) => {
            let elapsed = chrono::Local::now().signed_duration_since(*date);
            if elapsed.num_days() > 0 {
                format!("{} days ago", elapsed.num_days())
            } else if elapsed.num_hours() > 0 {
                format!("{} hours ago", elapsed.num_hours())
            } else {
                "Just now".to_string()
            }
        }
        None => "Never".to_string(),
    };

    let files_deleted_text = format!("{} files", app.config.stats.total_files_deleted);

    let bytes = app.config.stats.total_bytes_freed;
    let space_freed_text = if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    };

    let stats_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Last Clean   : ", Style::default().fg(theme.muted_text())),
            Span::raw(last_clean_text).fg(theme.text()),
        ]),
        Line::from(vec![
            Span::styled("  Files Deleted: ", Style::default().fg(theme.muted_text())),
            Span::raw(files_deleted_text).fg(theme.text()),
        ]),
        Line::from(vec![
            Span::styled("  Space Freed  : ", Style::default().fg(theme.muted_text())),
            Span::styled(
                space_freed_text,
                Style::default()
                    .fg(theme.safe())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Score        : ", Style::default().fg(theme.muted_text())),
            Span::styled(
                "Keep cleaning! ⚡",
                Style::default()
                    .fg(theme.warning())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let stats_menu = Paragraph::new(stats_text).block(stats_block);
    f.render_widget(stats_menu, menu_layout[1]);

    // ==========================================
    // SECTION 4: FOOTER (Help & Navigation)
    // ==========================================
    let help_line = Paragraph::new(Line::from(vec![
        " [q] ".fg(theme.accent()).bold(),
        "Quit  ".fg(theme.muted_text()),
        " [f] ".fg(theme.primary()).bold(),
        "File Manager  ".fg(theme.muted_text()),
        " [u] ".fg(theme.secondary()).bold(),
        "Uninstaller  ".fg(theme.muted_text()),
        " [s] ".fg(theme.accent()).bold(),
        "Settings  ".fg(theme.muted_text()),
        " [p] ".fg(theme.danger()).bold(),
        "Processes  ".fg(theme.muted_text()),
        " [↑↓] ".fg(theme.safe()).bold(),
        "Navigate".fg(theme.muted_text()),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.muted_text())),
    );
    f.render_widget(help_line, main_layout[3]);
}
