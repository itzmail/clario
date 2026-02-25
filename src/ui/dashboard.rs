use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_dashboard<B: Backend>(f: &mut Frame, selected_menu: usize) {
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
        .border_style(Style::default().fg(Color::Cyan));

    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " 🧹 Clario ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "v1.0 - Terminal System Cleaner",
            Style::default().fg(Color::DarkGray),
        ),
    ])])
    .block(header_block)
    .alignment(Alignment::Center);

    f.render_widget(header, main_layout[0]);

    // --- BAGIAN 2: SYSTEM INFO ---
    let sys_info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green))
        .title("System Overview");

    // Menggunakan macro .bold().cyan() dari stylize trait -> ini keren dan mirip builder pattern di Java!
    let sys_info = Paragraph::new(vec![
        Line::from(vec![
            "💻 System: ".bold(),
            "MacBook Pro ".cyan(),
            " 🟢 Storage: ".bold(),
            "156.3GB".red(),
            "/".dark_gray(),
            "256GB ".blue(),
            "(61%)".yellow(),
        ]),
        Line::from(""), // Spacing
        Line::from(vec![
            "📈 Current Issues: ".bold().yellow(),
            "⚠️ Browser cache > 1GB | ⚠️ 3 apps unused > 3 months".red(),
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
        .border_style(Style::default().fg(Color::Yellow));

    // Styling dinamis berdasarkan pilihan menu
    let m1_style = if selected_menu == 0 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };
    let m2_style = if selected_menu == 1 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    };
    let m3_style = if selected_menu == 2 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Yellow)
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
        Line::from("      Smart cleanup - 2.1GB ready to delete".dark_gray()),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::raw(m2_prefix),
            Span::styled("🗑️  [u] Uninstall Applications", m2_style),
        ]),
        Line::from("      15 apps, 3 unused, 524MB total size".dark_gray()),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::raw(m3_prefix),
            Span::styled("⚙️  [s] Settings", m3_style),
        ]),
        Line::from("      Configure cleanup rules and safety options".dark_gray()),
    ];

    let action_menu = Paragraph::new(action_menu_text).block(action_menu_block);
    f.render_widget(action_menu, menu_layout[0]);

    // Kotak Kanan: Stats Snapshot
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" 📊 Quick Stats ")
        .border_style(Style::default().fg(Color::LightBlue));

    let stats_text = vec![
        Line::from(""),
        Line::from("  Last Clean   : 2 days ago".gray()),
        Line::from("  Files Deleted: 142".gray()),
        Line::from("  Space Freed  : 8.1GB".green()),
        Line::from(""),
        Line::from("  Score        : 85/100 ⚡".yellow().bold()),
    ];
    let stats_menu = Paragraph::new(stats_text).block(stats_block);
    f.render_widget(stats_menu, menu_layout[1]);

    // --- BAGIAN 4: FOOTER TABS ---
    let help_line = Paragraph::new(Line::from(vec![
        " [q] ".yellow().bold(),
        "Quit  ".dark_gray(),
        " [f] ".cyan().bold(),
        "File Manager  ".dark_gray(),
        " [s] ".yellow().bold(),
        "Settings  ".dark_gray(),
        " [↑↓] ".green().bold(),
        "Navigate".dark_gray(),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(help_line, main_layout[3]);
}
