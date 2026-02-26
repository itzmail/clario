use crate::models::{
    config::AppTheme,
    file_info::{FileCategory, FileInfo},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

// Kita update signature-nya agar menerima list file dari state App
pub fn draw_file_manager(
    f: &mut Frame,
    files: &[FileInfo],
    is_scanning: bool,
    scan_progress_text: &str,
    table_state: &mut ratatui::widgets::TableState,
    show_delete_confirm: bool,
    delete_confirm_selected: u8,
    is_deleting: bool,
    show_archive_confirm: bool,
    archive_confirm_selected: u8,
    is_archiving: bool,
    theme: &AppTheme,
) {
    let size = f.area();

    // Jika sedang dalam proses scan, tampilkan layar loading
    if is_scanning {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(5),
                Constraint::Percentage(40),
            ])
            .split(size);

        let loading_text = format!(
            "⏳ Scanning your Mac for junk files... Please wait.\n\nScanning: {}",
            scan_progress_text
        );

        let loading = Paragraph::new(loading_text)
            .style(
                Style::default()
                    .fg(theme.warning())
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(ratatui::layout::Alignment::Center);

        // Render di tengah layar
        f.render_widget(loading, layout[1]);
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
        .border_style(Style::default().fg(theme.primary()))
        .style(Style::default().bg(theme.bg()).fg(theme.text()))
        .border_type(BorderType::Rounded);

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
        theme: &AppTheme,
    ) {
        let indent = "  ".repeat(depth);

        for file in files {
            if depth == 0 {
                *total_size += file.size_bytes; // Hitung total ukuran target utama
            }

            let size_mb = (file.size_bytes as f64) / 1_048_576.0;
            let size_str = format!("{:.2} MB", size_mb);

            let cat_str = match file.category {
                FileCategory::Cache => "Cache",
                FileCategory::Log => "Log",
                FileCategory::Document => "Document",
                FileCategory::Application => "App",
                FileCategory::Archive => "Archive",
                FileCategory::Other => "Other",
            };

            let (mut icon_prefix, icon_color) = if file.is_dir {
                if file.is_expanded {
                    ("📁 ▼", theme.secondary())
                } else {
                    ("📁 ▶", theme.primary())
                }
            } else {
                ("📄  ", theme.text())
            };

            // Custom icon untuk Directory Action
            if !file.is_dir && depth > 0 {
                icon_prefix = "╰─ ";
            }

            let (safety_str, safety_color) = match file.safety {
                crate::models::file_info::SafetyLevel::SafeToDelete => ("✅ Safe", theme.safe()),
                crate::models::file_info::SafetyLevel::ProceedWithCaution => {
                    ("⚠️ Caution", theme.warning())
                }
                crate::models::file_info::SafetyLevel::SystemCritical => {
                    ("🛑 Danger", theme.danger())
                }
            };

            // Checkbox UI
            let check_icon = if file.is_selected { "[x]" } else { "[ ]" };
            let check_color = if file.is_selected {
                theme.danger()
            } else {
                theme.muted_text()
            };

            let row = Row::new(vec![
                Cell::from(Span::styled(
                    format!("{}{} {}", indent, check_icon, icon_prefix),
                    Style::default().fg(check_color),
                )),
                Cell::from(Span::styled(
                    file.name.clone(),
                    Style::default().fg(if file.is_selected {
                        theme.danger()
                    } else {
                        icon_color
                    }),
                )),
                Cell::from(Span::styled(
                    cat_str,
                    Style::default().fg(theme.muted_text()),
                )),
                Cell::from(Span::styled(
                    size_str,
                    Style::default().fg(if file.size_bytes > 500_000_000 {
                        theme.warning()
                    } else {
                        theme.text()
                    }),
                )),
                Cell::from(Span::styled(safety_str, Style::default().fg(safety_color))),
                Cell::from(Span::styled(
                    file.last_modified.format("%Y-%m-%d").to_string(),
                    Style::default().fg(theme.muted_text()),
                )),
            ])
            .style(Style::default().fg(theme.text()))
            .bottom_margin(0);

            rows.push(row);

            // Jika folder tersebut "expanded", render anak-nya juga sehingga indexnya bersambung natural
            if file.is_expanded && file.is_dir {
                build_rows(&file.children, depth + 1, rows, total_size, theme);
            }
        }
    }

    // Panggil helper rekursif!
    build_rows(files, 0, &mut rows, &mut total_size_bytes, theme);

    // Header Kolom
    let header = Row::new(vec![
        " ",
        "File Name",
        "Category",
        "Size",
        "Safety",
        "Date Modified",
    ])
    .style(
        Style::default()
            .fg(theme.bg())
            .bg(theme.primary())
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    // Bikin objek Tabel
    // Constraint percentage itu bagaikan pembagian lebar flex (misal kolom nama dapet 50% layar)
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(8),
            Constraint::Percentage(37),
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
            .bg(theme.unselected_bg())
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
        Span::styled("Scroll  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [→/←] ",
            Style::default()
                .fg(theme.safe())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Exp/Col  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [Space] ",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Select  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [x/Del] ",
            Style::default()
                .fg(theme.danger())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Delete  ", Style::default().fg(theme.muted_text())),
        Span::styled(
            " [a] ",
            Style::default()
                .fg(theme.archive())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Archive", Style::default().fg(theme.muted_text())),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.muted_text())),
    );
    f.render_widget(help_line, main_layout[1]);

    // MENGGAMBAR MODAL KONFIRMASI (Overlay di atas Tabel)
    if show_delete_confirm {
        // Buat kotak di tengah layar (lebar 60 char, tinggi 12 char agar muat tombol)
        let modal_area = centered_rect(60, 12, size);

        // CLEAR area di belakang modal agar Text Tabel tidak bertumpuk / tembus
        f.render_widget(ratatui::widgets::Clear, modal_area);

        // Render main modal outer block
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(theme.danger()))
            .title(" Confirm Deletion ");
        f.render_widget(modal_block.clone(), modal_area);

        // Ambil area di dalam border modal utama
        let inner_area = modal_block.inner(modal_area);

        // Memotong inner_area secara horizontal: Area Teks dan Area Tombol Border
        let modal_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Tinggi teks
                Constraint::Length(3), // Tinggi Tombol bertulang (border)
                Constraint::Min(0),
            ])
            .split(inner_area);

        let confirm_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "⚠️ WARNING: DELETE FILES",
                Style::default()
                    .fg(theme.danger())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Are you sure you want to permanently"),
            Line::from("delete the selected files?"),
        ];

        f.render_widget(
            Paragraph::new(confirm_text).alignment(Alignment::Center),
            modal_chunks[0],
        );

        // Layout horizontal untuk menengahkan dua frame tombol kotak
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),      // Margin di kiri
                Constraint::Percentage(50), // Grid 1 untuk Tombol Confirm
                Constraint::Percentage(50), // Grid 2 untuk Tombol Cancel
                Constraint::Length(2),      // Margin di kanan
            ])
            .split(modal_chunks[1]);

        let (confirm_border, confirm_style) = if delete_confirm_selected == 0 {
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

        let (cancel_border, cancel_style) = if delete_confirm_selected == 1 {
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
        // Render tombol ke grid yang sudah kita pecah tadi
        f.render_widget(btn_confirm, button_chunks[1]);
        f.render_widget(btn_cancel, button_chunks[2]);
    }

    // MENGGAMBAR MODAL KONFIRMASI ARCHIVE
    if show_archive_confirm {
        let modal_area = centered_rect(60, 12, size);
        f.render_widget(ratatui::widgets::Clear, modal_area);

        let modal_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(theme.archive()))
            .title(" Confirm Archiving ");
        f.render_widget(modal_block.clone(), modal_area);

        let inner_area = modal_block.inner(modal_area);
        let modal_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner_area);

        let confirm_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "📦 ARCHIVE SELECTED FILES",
                Style::default()
                    .fg(theme.archive())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Files will be bundled into a .zip in ~/Clario_Archives/"),
            Line::from("The original files will be removed. Proceed?"),
        ];

        f.render_widget(
            Paragraph::new(confirm_text).alignment(Alignment::Center),
            modal_chunks[0],
        );

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
                Constraint::Length(2),
            ])
            .split(modal_chunks[1]);

        let (confirm_border, confirm_style) = if archive_confirm_selected == 0 {
            (
                theme.archive(),
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.archive())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (theme.muted_text(), Style::default().fg(theme.muted_text()))
        };

        let (cancel_border, cancel_style) = if archive_confirm_selected == 1 {
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

        let btn_confirm = Paragraph::new(" [Y/Enter] Archive ")
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

        f.render_widget(btn_confirm, button_chunks[1]);
        f.render_widget(btn_cancel, button_chunks[2]);
    }

    // Modal Indikator Loading Deletion / Archiving
    if is_deleting || is_archiving {
        let modal_area = centered_rect(50, 6, size);
        f.render_widget(ratatui::widgets::Clear, modal_area); // Clear background under modal

        let color = if is_deleting {
            theme.danger()
        } else {
            theme.archive()
        };
        let msg = if is_deleting {
            "🗑️  Deletions are permanent. Please wait..."
        } else {
            "📦 Archiving files into ~/Clario_Archives... Please wait."
        };

        // Modal Styling Block
        let modal_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(color))
            .title(if is_deleting {
                "🗑️  DELETING "
            } else {
                "📦 ARCHIVING "
            })
            .style(Style::default().bg(theme.bg())); // Warna Background menutupi tabel agar jelas

        // Text Animasi ProgressBar (Menggunakan Spinner sederhana untuk UI terminal)
        let spinner = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                if is_deleting {
                    "Deleting Selected Files"
                } else {
                    "Archiving Selected Files"
                },
                Style::default()
                    .fg(theme.text())
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                scan_progress_text, // Munculkan log dinamis string file IO !
                Style::default().fg(color),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                msg,
                Style::default().fg(theme.muted_text()),
            )]),
        ];

        let paragraph = Paragraph::new(spinner)
            .alignment(Alignment::Center)
            .block(modal_block);

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
