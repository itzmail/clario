use crate::handlers;
use crate::models::config::AppConfig;
use crate::models::file_info::FileInfo;
use crate::ui::{
    app_uninstaller::draw_app_uninstaller, components::draw_exit_modal, dashboard::draw_dashboard,
    file_manager::draw_file_manager, settings::draw_settings,
};
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

pub struct App {
    pub should_quit: bool,
    pub mode: AppMode,
    pub selected_menu: usize,
    pub scanned_files: Vec<FileInfo>,
    pub is_scanning: bool,
    pub scan_progress_text: String,
    pub scan_rx: Option<mpsc::Receiver<crate::core::events::ScanEvent>>,
    pub file_table_state: TableState,
    pub show_delete_confirm: bool,
    pub delete_confirm_selected: u8, // 0 = Confirm, 1 = Cancel
    pub is_deleting: bool,           // True jika sedang dlm proses hapus data di background
    pub delete_rx: Option<mpsc::Receiver<Option<String>>>,
    pub show_archive_confirm: bool,
    pub archive_confirm_selected: u8, // 0 = Confirm, 1 = Cancel
    pub is_archiving: bool,
    pub archive_rx: Option<mpsc::Receiver<Option<String>>>,
    pub sys: sysinfo::System,
    pub last_sys_refresh: Instant,
    pub config: AppConfig,
    pub settings_selected_index: usize,
    pub is_dir_picker_open: bool,
    pub dir_picker_path: std::path::PathBuf,
    pub dir_picker_items: Vec<FileInfo>,
    pub dir_picker_selected: usize,
    pub show_exit_confirm: bool,
    pub exit_confirm_selected: u8, // 0 = Yes, 1 = Wait

    // Fitur App Uninstaller
    pub apps: Vec<crate::models::app_info::AppInfo>,
    pub selected_app_index: usize,
    pub app_table_state: TableState,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AppMode {
    Dashboard,
    FileManager,
    Settings,
    AppUninstaller,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            mode: AppMode::Dashboard,
            selected_menu: 0,
            scanned_files: Vec::new(),
            is_scanning: false,
            scan_progress_text: String::new(),
            scan_rx: None,
            file_table_state: TableState::default(),
            show_delete_confirm: false,
            delete_confirm_selected: 1,
            is_deleting: false,
            delete_rx: None,
            show_archive_confirm: false,
            archive_confirm_selected: 1,
            is_archiving: false,
            archive_rx: None,
            sys: sysinfo::System::new_all(),
            last_sys_refresh: Instant::now(),
            config: AppConfig::load(),
            settings_selected_index: 0,
            is_dir_picker_open: false,
            dir_picker_path: dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/")),
            dir_picker_items: Vec::new(),
            dir_picker_selected: 0,
            show_exit_confirm: false,
            exit_confirm_selected: 1, // Default focus on "Wait, not yet"
            apps: Vec::new(),
            selected_app_index: 0,
            app_table_state: TableState::default(),
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        while !self.should_quit {
            // Menerima stream scanning events dari background thread secara instan (non-blocking)
            if let Some(rx) = &mut self.scan_rx {
                while let Ok(event) = rx.try_recv() {
                    match event {
                        crate::core::events::ScanEvent::Progress(text) => {
                            self.scan_progress_text = text;
                        }
                        crate::core::events::ScanEvent::Finished(files) => {
                            self.scanned_files = files;
                            self.is_scanning = false;
                            self.scan_rx = None;
                            self.file_table_state.select(Some(0));
                            break;
                        }
                        crate::core::events::ScanEvent::FinishedApps(apps) => {
                            self.apps = apps;
                            self.is_scanning = false;
                            self.scan_rx = None;
                            self.app_table_state.select(Some(0));
                            break;
                        }
                    }
                }
            }

            // Menerima signal stream dari thread deletion di background
            if let Some(ref mut rx) = self.delete_rx {
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        Some(text) => {
                            self.scan_progress_text = text;
                        }
                        None => {
                            // Finish Deleting
                            self.is_deleting = false;
                            self.show_delete_confirm = false;
                            self.delete_rx = None;

                            // Bersihkan data sesuai mode saat ini
                            if self.mode == AppMode::AppUninstaller {
                                // Hapus AppInfo yang sudah di-uninstall dari list
                                self.apps.retain(|a| !a.is_selected);
                                // Reset index agar tidak out-of-bounds
                                if self.selected_app_index >= self.apps.len() {
                                    self.selected_app_index = self.apps.len().saturating_sub(1);
                                }
                                self.app_table_state.select(Some(self.selected_app_index));
                            } else {
                                crate::core::file_ops::FileOps::retain_unselected(
                                    &mut self.scanned_files,
                                );
                            }
                            break;
                        }
                    }
                }
            }

            // Menerima signal stream dari thread archive di background
            if let Some(ref mut rx) = self.archive_rx {
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        Some(text) => {
                            self.scan_progress_text = text; // Update UI ZIP path
                        }
                        None => {
                            // Finish Archiving
                            self.is_archiving = false;
                            self.show_archive_confirm = false;
                            self.archive_rx = None;
                            crate::core::file_ops::FileOps::retain_unselected(
                                &mut self.scanned_files,
                            );
                            break;
                        }
                    }
                }
            }

            // Handle Sysinfo Throttling Rate Limit (refresh disk+ram stat every 2 secs)
            if self.last_sys_refresh.elapsed() >= std::time::Duration::from_secs(2) {
                self.sys.refresh_all();
                self.last_sys_refresh = Instant::now();
            }

            // Render UI sesuai mode
            terminal.draw(|f| {
                match self.mode {
                    AppMode::Dashboard => {
                        draw_dashboard(f, self)
                    }
                    AppMode::FileManager => {
                        // Render indikator scanner progress di UI saat mode scan
                        if self.scanned_files.is_empty() && !self.is_scanning {
                            self.is_scanning = true;

                            let (tx, rx) = mpsc::channel();
                            self.scan_rx = Some(rx);

                            // Kirim job scanning raksasa (berat) ke background IO thread agar layout UI tetep gesit
                            let threshold = self.config.safety_threshold_days;
                            tokio::task::spawn_blocking(move || {
                                let targets = crate::utils::platform::get_scan_targets();
                                crate::core::file_scanner::FileScanner::scan_targets(
                                    &targets, threshold, tx,
                                );
                            });
                        }
                        draw_file_manager(f, self);
                    }
                    AppMode::Settings => draw_settings(f, self),
                    AppMode::AppUninstaller => {
                        draw_app_uninstaller(f, self);
                    }
                }

                // Draw Exit Modal ON TOP of everything if requested
                if self.show_exit_confirm {
                    draw_exit_modal(f, self.exit_confirm_selected, &self.config.theme);
                }
            })?;

            if poll(Duration::from_millis(100))? {
                if let Event::Key(key) = read()? {
                    // 1. Intercept keys if Global Exit Modal is active
                    if self.show_exit_confirm {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                self.show_exit_confirm = false;
                                self.exit_confirm_selected = 1; // reset ke "Wait"
                            }
                            KeyCode::Left
                            | KeyCode::Right
                            | KeyCode::Char('h')
                            | KeyCode::Char('l') => {
                                // Toggle antara 0 (Yes) dan 1 (No)
                                self.exit_confirm_selected = 1 - self.exit_confirm_selected;
                            }
                            KeyCode::Enter => {
                                if self.exit_confirm_selected == 0 {
                                    self.should_quit = true;
                                } else {
                                    self.show_exit_confirm = false;
                                }
                            }
                            _ => {} // Abaikan input lain selama modal terbuka
                        }
                        continue; // Skip the rest of event handling
                    }

                    // 2. Global mode switching hotkeys
                    match key.code {
                        KeyCode::Char('q') => {
                            if self.mode == AppMode::Dashboard {
                                self.should_quit = true;
                            } else {
                                self.show_exit_confirm = true;
                                self.exit_confirm_selected = 1; // Focus "Wait, not yet" default
                            }
                            continue;
                        }
                        KeyCode::Char('f') => {
                            self.mode = AppMode::FileManager;
                            continue;
                        }
                        KeyCode::Char('u') => {
                            self.mode = AppMode::AppUninstaller;
                            // Kickoff the uninstaller thread load
                            if self.apps.is_empty() && !self.is_scanning {
                                self.is_scanning = true;
                                self.scan_progress_text = String::new();
                                let (tx, rx) = mpsc::channel();
                                self.scan_rx = Some(rx);

                                tokio::task::spawn_blocking(move || {
                                    let scanned_apps =
                                        crate::core::app_scanner::AppScanner::scan_applications(
                                            tx.clone(),
                                        );
                                    let _ =
                                        tx.send(crate::core::events::ScanEvent::FinishedApps(
                                            scanned_apps,
                                        ));
                                });
                            }
                            continue;
                        }
                        KeyCode::Char('s') => {
                            self.mode = AppMode::Settings;
                            continue;
                        }
                        KeyCode::Char('d') => {
                            self.mode = AppMode::Dashboard;
                            self.selected_menu = 0;
                            continue;
                        }
                        _ => {}
                    }

                    // 3. Menyerahkan event handling ke modul handlers spesifik berdasarkan mode
                    match self.mode {
                        AppMode::Dashboard => handlers::dashboard::handle_key(self, key),
                        AppMode::FileManager => handlers::file_manager::handle_key(self, key),
                        AppMode::Settings => handlers::settings::handle_key(self, key),
                        AppMode::AppUninstaller => handlers::app_uninstaller::handle_key(self, key),
                    }
                }
            }
        }

        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }
}
