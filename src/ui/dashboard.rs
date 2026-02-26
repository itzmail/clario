use crate::models::config::AppTheme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};
use sysinfo::System;

pub fn draw_dashboard(
    f: &mut Frame,
    selected_menu: usize,
    sys: &sysinfo::System,
    theme: &AppTheme,
) {
    let size = f.area();

    // 1. Membagi Layar Utama (Mirip Flexbox Column)
    // - Header: 3 baris
    // - System Info & Status: 6 baris
    // - Menu Utama: Sisa layar (Min 0)
    // - Footer List: 3 baris
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

    // --- BAGIAN 1: HEADER ---
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.primary()));

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

    f.render_widget(header, main_layout[0]);

    // --- BAGIAN 2: SYSTEM INFO DYNAMIC ---
    let sys_info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.primary()))
        .title(" System Overview ");

    // Get Live Information
    let os_name = System::long_os_version().unwrap_or_else(|| "Unknown OS".to_string());

    // RAM Calculation (Convert bytes to GB)
    let total_mem_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let used_mem_gb = sys.used_memory() as f64 / 1_073_741_824.0;
    let mem_percentage = if total_mem_gb > 0.0 {
        (used_mem_gb / total_mem_gb) * 100.0
    } else {
        0.0
    };

    let mem_health_color = if mem_percentage > 85.0 {
        theme.danger()
    } else if mem_percentage > 70.0 {
        theme.warning()
    } else {
        theme.safe()
    };

    // Note: Parsing specific physical Drive Usage efficiently is complex if there are multiple virtual volumes (APFS etc).
    // For clarity, we summarize RAM here as it's the primary target of Sysinfo right now.

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
        Line::from(""), // Spacing
        Line::from(vec![
            "📈 Tip: ".bold().fg(theme.warning()),
            "Use the 'File Manager' to hunt down and delete large unused caches".fg(theme.text()),
        ]),
    ])
    .block(sys_info_block)
    .wrap(Wrap { trim: true });

    f.render_widget(sys_info, main_layout[1]);

    // --- BAGIAN 3: MENU UTAMA (Split 2 Kolom) ---
    // Di dalam Main Menu, kita split lagi jadi Kiri (Aksi) & Kanan (Statistik) -- mirip flex-direction: row
    let menu_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[2]);

    // Kotak Kiri: Action Menu
    let action_menu_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" 📋 Main Menu ")
        .border_style(Style::default().fg(theme.accent()));

    // Styling dinamis berdasarkan pilihan menu
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

    let m1_prefix = if selected_menu == 0 { "▶ " } else { "  " };
    let m2_prefix = if selected_menu == 1 { "▶ " } else { "  " };
    let m3_prefix = if selected_menu == 2 { "▶ " } else { "  " };

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
    ];

    let action_menu = Paragraph::new(action_menu_text).block(action_menu_block);
    f.render_widget(action_menu, menu_layout[0]);

    // Kotak Kanan: Stats Snapshot
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" 📊 Quick Stats ")
        .border_style(Style::default().fg(theme.secondary()));

    let stats_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Last Clean   : ", Style::default().fg(theme.muted_text())),
            Span::raw("2 days ago").fg(theme.text()),
        ]),
        Line::from(vec![
            Span::styled("  Files Deleted: ", Style::default().fg(theme.muted_text())),
            Span::raw("142").fg(theme.text()),
        ]),
        Line::from(vec![
            Span::styled("  Space Freed  : ", Style::default().fg(theme.muted_text())),
            Span::styled(
                "8.1GB",
                Style::default()
                    .fg(theme.safe())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Score        : ", Style::default().fg(theme.muted_text())),
            Span::styled(
                "85/100 ⚡",
                Style::default()
                    .fg(theme.warning())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let stats_menu = Paragraph::new(stats_text).block(stats_block);
    f.render_widget(stats_menu, menu_layout[1]);

    // --- BAGIAN 4: FOOTER TABS ---
    let help_line = Paragraph::new(Line::from(vec![
        " [q] ".fg(theme.accent()).bold(),
        "Quit  ".fg(theme.muted_text()),
        " [f] ".fg(theme.primary()).bold(),
        "File Manager  ".fg(theme.muted_text()),
        " [u] ".fg(theme.secondary()).bold(),
        "Uninstaller  ".fg(theme.muted_text()),
        " [s] ".fg(theme.accent()).bold(),
        "Settings  ".fg(theme.muted_text()),
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
