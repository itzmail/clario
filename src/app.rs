use crate::core::file_scanner::FileScanner;
use crate::models::file_info::FileInfo;
use crate::ui::{
    dashboard::draw_dashboard, file_manager::draw_file_manager, settings::draw_settings,
};
use crate::utils::platform;
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use std::sync::mpsc;
use std::time::Duration;

pub struct App {
    should_quit: bool,
    mode: AppMode,
    pub selected_menu: usize,
    pub scanned_files: Vec<FileInfo>,
    pub is_scanning: bool,
    pub scan_rx: Option<mpsc::Receiver<Vec<FileInfo>>>,
    pub file_table_state: TableState,
    pub show_delete_confirm: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AppMode {
    Dashboard,
    FileManager,
    Settings,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            mode: AppMode::Dashboard,
            selected_menu: 0,
            scanned_files: Vec::new(),
            is_scanning: false,
            scan_rx: None,
            file_table_state: TableState::default(),
            show_delete_confirm: false,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        while !self.should_quit {
            // Menerima hasil scan dari background thread secara instan (non-blocking)
            if let Some(rx) = &mut self.scan_rx {
                if let Ok(files) = rx.try_recv() {
                    self.scanned_files = files;
                    self.is_scanning = false;
                    self.scan_rx = None;
                    self.file_table_state.select(Some(0)); // Pilih baris pertama secara default
                }
            }

            // Render UI sesuai mode
            terminal.draw(|f| match self.mode {
                AppMode::Dashboard => {
                    draw_dashboard::<CrosstermBackend<std::io::Stdout>>(f, self.selected_menu)
                }
                AppMode::FileManager => draw_file_manager::<CrosstermBackend<std::io::Stdout>>(
                    f,
                    &self.scanned_files,
                    self.is_scanning,
                    &mut self.file_table_state,
                    self.show_delete_confirm,
                ),
                AppMode::Settings => draw_settings::<CrosstermBackend<std::io::Stdout>>(f),
            })?;

            if poll(Duration::from_millis(100))? {
                if let Event::Key(key) = read()? {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true, // Tekan 'q' untuk keluar
                        KeyCode::Char('f') => self.mode = AppMode::FileManager, // 'f' untuk FileManager
                        KeyCode::Char('s') => self.mode = AppMode::Settings, // 's' untuk Settings
                        KeyCode::Char('d') => {
                            self.mode = AppMode::Dashboard;
                            self.selected_menu = 0;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.mode == AppMode::Dashboard {
                                self.selected_menu = (self.selected_menu + 1) % 3;
                            } else if self.mode == AppMode::FileManager {
                                // Scroll ke bawah pada tabel
                                if let Some(selected) = self.file_table_state.selected() {
                                    self.file_table_state
                                        .select(Some(selected.saturating_add(1)));
                                } else {
                                    self.file_table_state.select(Some(0));
                                }
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.mode == AppMode::Dashboard {
                                if self.selected_menu == 0 {
                                    self.selected_menu = 2;
                                } else {
                                    self.selected_menu -= 1;
                                }
                            } else if self.mode == AppMode::FileManager {
                                // Scroll ke atas pada tabel
                                if let Some(selected) = self.file_table_state.selected() {
                                    self.file_table_state
                                        .select(Some(selected.saturating_sub(1)));
                                }
                            }
                        }
                        // Navigasi masuk/expand folder
                        KeyCode::Right | KeyCode::Char('l') => {
                            if self.mode == AppMode::FileManager && !self.show_delete_confirm {
                                if let Some(selected) = self.file_table_state.selected() {
                                    let mut count = 0;
                                    Self::toggle_recursive(
                                        &mut self.scanned_files,
                                        selected,
                                        &mut count,
                                        ToggleAction::Expand,
                                    );
                                }
                            }
                        }
                        // Konfirmasi Pilihan Menu (Enter)
                        KeyCode::Enter => {
                            if self.mode == AppMode::Dashboard {
                                match self.selected_menu {
                                    0 => {
                                        // Pindah UI ke File Manager
                                        self.mode = AppMode::FileManager;

                                        // Jalankan scan menggunakan background thread agar TUI tidak nge-freeze (loading screen tampil)!
                                        if self.scanned_files.is_empty() && !self.is_scanning {
                                            self.is_scanning = true;
                                            let (tx, rx) = mpsc::channel();
                                            self.scan_rx = Some(rx);
                                            let targets = platform::get_scan_targets();

                                            // Spawn tokio background task thread
                                            tokio::task::spawn_blocking(move || {
                                                let results = FileScanner::scan_targets(&targets);
                                                let _ = tx.send(results);
                                            });
                                        }
                                    }
                                    1 => (),
                                    2 => self.mode = AppMode::Settings,
                                    _ => {}
                                }
                            } else if self.mode == AppMode::FileManager {
                                // Enter juga berfungsi sebagai expand folder kalau di dalam file manager
                                if self.show_delete_confirm {
                                    self.execute_deletion();
                                    self.show_delete_confirm = false;
                                } else {
                                    if let Some(selected) = self.file_table_state.selected() {
                                        let mut count = 0;
                                        Self::toggle_recursive(
                                            &mut self.scanned_files,
                                            selected,
                                            &mut count,
                                            ToggleAction::Expand,
                                        );
                                    }
                                }
                            }
                        }
                        // Seleksi File & Deletion Intercept
                        KeyCode::Char(' ') => {
                            if self.mode == AppMode::FileManager && !self.show_delete_confirm {
                                if let Some(selected) = self.file_table_state.selected() {
                                    let mut count = 0;
                                    Self::toggle_recursive(
                                        &mut self.scanned_files,
                                        selected,
                                        &mut count,
                                        ToggleAction::Select,
                                    );
                                }
                            }
                        }
                        KeyCode::Char('x') | KeyCode::Delete | KeyCode::Backspace => {
                            if self.mode == AppMode::FileManager && !self.show_delete_confirm {
                                if Self::has_any_selected(&self.scanned_files) {
                                    self.show_delete_confirm = true;
                                }
                            }
                        }
                        KeyCode::Char('y') => {
                            if self.mode == AppMode::FileManager && self.show_delete_confirm {
                                self.execute_deletion();
                                self.show_delete_confirm = false;
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Esc => {
                            if self.mode == AppMode::FileManager && self.show_delete_confirm {
                                self.show_delete_confirm = false;
                            } else if key.code == KeyCode::Esc {
                                self.should_quit = true; // Fallback umum
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    /// Helper untuk mencari baris layar ke-N pada Flattened View lalu membalik state `is_expanded`-nya
    fn toggle_recursive(
        files: &mut [FileInfo],
        target_index: usize,
        current_index: &mut usize,
        action: ToggleAction,
    ) -> bool {
        for file in files.iter_mut() {
            if *current_index == target_index {
                match action {
                    ToggleAction::Expand => file.is_expanded = !file.is_expanded,
                    ToggleAction::Select => {
                        file.is_selected = !file.is_selected;
                        // Jika parent dipilih, anak-anaknya semua ikut dipilih
                        if file.is_dir {
                            Self::set_selection_all(&mut file.children, file.is_selected);
                        }
                    }
                }
                return true;
            }
            *current_index += 1;

            // Jika folder dan sedang dibuka (expanded), anak-anaknya otomatis dihitung indexnya!
            if file.is_expanded && file.is_dir {
                if Self::toggle_recursive(&mut file.children, target_index, current_index, action) {
                    return true;
                }
            }
        }
        false
    }

    fn has_any_selected(files: &[FileInfo]) -> bool {
        for f in files {
            if f.is_selected {
                return true;
            }
            if f.is_expanded && f.is_dir {
                if Self::has_any_selected(&f.children) {
                    return true;
                }
            }
        }
        false
    }

    fn set_selection_all(files: &mut [FileInfo], checked: bool) {
        for f in files.iter_mut() {
            f.is_selected = checked;
            if f.is_dir {
                Self::set_selection_all(&mut f.children, checked);
            }
        }
    }

    fn execute_deletion(&mut self) {
        // Hapus fisik via std::fs
        Self::delete_selected_recursive(&mut self.scanned_files);
        // Hapus dari list memory (UI)
        Self::retain_unselected(&mut self.scanned_files);
    }

    fn delete_selected_recursive(files: &mut [FileInfo]) {
        for f in files.iter_mut() {
            if f.is_selected {
                if f.is_dir {
                    let _ = std::fs::remove_dir_all(&f.path);
                } else {
                    let _ = std::fs::remove_file(&f.path);
                }
            } else if f.is_dir {
                Self::delete_selected_recursive(&mut f.children);
            }
        }
    }

    fn retain_unselected(files: &mut Vec<FileInfo>) {
        files.retain(|f| !f.is_selected);
        for f in files.iter_mut() {
            if f.is_dir {
                Self::retain_unselected(&mut f.children);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ToggleAction {
    Expand,
    Select,
}
