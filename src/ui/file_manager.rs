use crate::models::file_info::FileInfo;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

// Kita update signature-nya agar menerima list file dari state App
pub fn draw_file_manager<B: Backend>(
    f: &mut Frame,
    files: &[FileInfo],
    is_scanning: bool,
    table_state: &mut ratatui::widgets::TableState,
    show_delete_confirm: bool,
) {
    let size = f.area();

    // Jika sedang dalam proses scan, tampilkan layar loading
    if is_scanning {
        let loading = Paragraph::new("⏳ Scanning your Mac for junk files... Please wait.")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(ratatui::layout::Alignment::Center);

        // Render di tengah layar
        f.render_widget(loading, size);
        return;
    }

    // Membagi layar menjadi 2 bagian: MAIN TABLE dan FOOTER
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0),    // Tabel mengisi sisa paling besar
            Constraint::Length(3), // Header Shortcut (Mirip Dashboard)
        ])
        .split(size);

    // Jika sudah selesai scan, buat tabelnya
    let block = Block::default()
        .title(" 📁 File Manager - Clean Junk Files ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    // Convert data 'FileInfo' kita ke dalam format baris (Row) Ratatui
    let mut rows = Vec::new();

    // Total size counter untuk footer nanti
    let mut total_size_bytes = 0;

    // Fungsi helper mutlak (closure mutasi) untuk meratakan struktur tree ke vektor linear 'rows' khusus untuk dirender Ratatui
    fn build_rows<'a>(
        files: &'a [FileInfo],
        depth: usize,
        rows: &mut Vec<Row<'a>>,
        total_size: &mut u64,
    ) {
        let indent = "  ".repeat(depth);

        for file in files {
            if depth == 0 {
                *total_size += file.size_bytes; // Hitung total ukuran target utama
            }

            let size_mb = (file.size_bytes as f64) / 1_048_576.0;
            let size_str = format!("{:.2} MB", size_mb);

            let cat_str = format!("{:?}", file.category);
            let (mut icon, _cat_color) = match cat_str.as_str() {
                "Cache" => ("📦", Color::Yellow),
                "Log" => ("📜", Color::Blue),
                _ => ("📄", Color::DarkGray),
            };

            // Custom icon untuk Directory Action
            if file.is_dir {
                if file.is_expanded {
                    icon = "📂▼";
                } else {
                    icon = "📁▶";
                }
            } else if depth > 0 {
                icon = "╰─ ";
            }

            let (safety_str, safety_color) = match file.safety {
                crate::models::file_info::SafetyLevel::SafeToDelete => ("✅ Safe", Color::Green),
                crate::models::file_info::SafetyLevel::ProceedWithCaution => {
                    ("⚠️ Caution", Color::Yellow)
                }
                crate::models::file_info::SafetyLevel::SystemCritical => ("🛑 Danger", Color::Red),
            };

            // Checkbox UI
            let check_icon = if file.is_selected { "[x]" } else { "[ ]" };

            let row = Row::new(vec![
                Cell::from(format!("{}{} {} {}", indent, check_icon, icon, file.name)), // Nama & Indentasi hirarkis
                Cell::from(cat_str),
                Cell::from(size_str),
                Cell::from(safety_str).style(Style::default().fg(safety_color)),
                Cell::from(file.last_modified.format("%Y-%m-%d").to_string()),
            ])
            .style(Style::default().fg(Color::White))
            .bottom_margin(0);

            rows.push(row);

            // Jika folder tersebut "expanded", render anak-nya juga sehingga indexnya bersambung natural
            if file.is_expanded && file.is_dir {
                build_rows(&file.children, depth + 1, rows, total_size);
            }
        }
    }

    // Panggil helper rekursif!
    build_rows(files, 0, &mut rows, &mut total_size_bytes);

    // Header Kolom
    let header = Row::new(vec![
        "File Name",
        "Category",
        "Size",
        "Safety",
        "Date Modified",
    ])
    .style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    // Bikin objek Tabel
    // Constraint percentage itu bagaikan pembagian lebar flex (misal kolom nama dapet 50% layar)
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(45),
            Constraint::Percentage(13),
            Constraint::Percentage(12),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(block) // Tempel block (border) ke tabel
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    ) // Kursor table (fix linter!)
    .highlight_symbol(">> "); // Indikator garis panah seleksi saat ini

    // Render as stateful widget agar dia merespon dan me-remember scroll posisinya! pada Layout ke-0
    f.render_stateful_widget(table, main_layout[0], table_state);

    // Render FOOTER TABS -> pada Layout ke-1
    let help_line = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Esc] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Back to Dashboard  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " [↑/↓] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Scroll  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " [→] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Expand/Collapse  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " [Space] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Select  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " [x/Del] ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Delete", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(help_line, main_layout[1]);

    // MENGGAMBAR MODAL KONFIRMASI (Overlay di atas Tabel)
    if show_delete_confirm {
        // Buat kotak di tengah layar (lebar 60 char, tinggi 12 char agar muat tombol)
        let modal_area = centered_rect(60, 12, size);

        let confirm_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "⚠️ WARNING: DELETE FILES",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Are you sure you want to permanently"),
            Line::from("delete the selected files?"),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  [Y/Enter] Confirm Delete  ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("    "),
                Span::styled(
                    "  [N/Esc] Cancel  ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let modal_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::Red))
            .title(" Confirm Deletion ");

        let paragraph = Paragraph::new(confirm_text)
            .block(modal_block)
            .alignment(Alignment::Center);

        // CLEAR area di belakang modal agar Text Tabel tidak bertumpuk / tembus
        f.render_widget(ratatui::widgets::Clear, modal_area);
        f.render_widget(paragraph, modal_area);
    }
}

/// Helper untuk membuat Rectangle box di tengah layar (menggunakan absolute Length, bukan persentase)
fn centered_rect(width: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(r.height.saturating_sub(height) / 2),
            Constraint::Length(height), // FIXED HEIGHT untuk Modal (misal 12)
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(r.width.saturating_sub(width) / 2),
            Constraint::Length(width), // FIXED WIDTH for Modal (misal 60)
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}
