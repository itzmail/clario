use crate::handlers;
use crate::models::config::AppConfig;
use crate::models::file_info::FileInfo;
use crate::ui::{
    app_uninstaller::draw_app_uninstaller, components::draw_exit_modal, dashboard::draw_dashboard,
    file_manager::draw_file_manager, process_monitor::draw_process_monitor,
    settings::draw_settings, update::draw_update,
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
    pub scan_progress_text: String,      // file scan + app scan progress
    pub delete_progress_text: String,    // deletion progress
    pub archive_progress_text: String,   // archiving progress
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

    pub pending_bytes_to_free: u64, // Bytes of selected items, set before delete/archive starts

    // Fitur App Uninstaller
    pub apps: Vec<crate::models::app_info::AppInfo>,
    pub selected_app_index: usize,
    pub app_table_state: TableState,

    // Process Monitor fields
    pub processes: Vec<crate::models::process_info::ProcessInfo>,
    pub process_table_state: ratatui::widgets::TableState,
    pub selected_process_index: usize,
    pub show_kill_confirm: bool,
    pub kill_confirm_selected: u8, // 0=Cancel, 1=Graceful(SIGTERM), 2=Force(SIGKILL)
    pub kill_status_message: Option<String>,

    // Update fields
    pub update_state: crate::core::updater::UpdateState,
    pub update_releases: Vec<crate::core::updater::Release>,
    pub update_selected: usize,
    pub update_rx: Option<std::sync::mpsc::Receiver<crate::core::updater::UpdateEvent>>,
    pub update_status: String,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AppMode {
    Dashboard,
    FileManager,
    Settings,
    AppUninstaller,
    ProcessMonitor,
    Update,
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
            delete_progress_text: String::new(),
            archive_progress_text: String::new(),
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
            pending_bytes_to_free: 0,
            apps: Vec::new(),
            selected_app_index: 0,
            app_table_state: TableState::default(),
            processes: Vec::new(),
            process_table_state: ratatui::widgets::TableState::default(),
            selected_process_index: 0,
            show_kill_confirm: false,
            kill_confirm_selected: 0, // Default to Cancel (safe)
            kill_status_message: None,
            update_state: crate::core::updater::UpdateState::Idle,
            update_releases: Vec::new(),
            update_selected: 0,
            update_rx: None,
            update_status: String::new(),
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.event_loop(&mut terminal).await;

        // Always restore terminal state, even on error
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = crossterm::terminal::disable_raw_mode();

        res
    }

    async fn event_loop(
        &mut self,
        terminal: &mut ratatui::Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> anyhow::Result<()> {
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
                            self.delete_progress_text = text;
                        }
                        None => {
                            // Record stats before removing items from memory
                            self.config.stats.total_bytes_freed += self.pending_bytes_to_free;
                            self.config.stats.last_clean_date = Some(chrono::Local::now());

                            if self.mode == AppMode::AppUninstaller {
                                self.config.stats.total_files_deleted +=
                                    self.apps.iter().filter(|a| a.is_selected).count() as u64;
                            } else {
                                self.config.stats.total_files_deleted +=
                                    crate::core::file_ops::FileOps::count_selected(
                                        &self.scanned_files,
                                    ) as u64;
                            }
                            self.config.save();
                            self.pending_bytes_to_free = 0;

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
                                // Clear semua — next loop tick akan auto-trigger rescan
                                self.scanned_files.clear();
                                self.is_scanning = false;
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
                            self.archive_progress_text = text; // Update UI ZIP path
                        }
                        None => {
                            // Record stats before removing items from memory
                            self.config.stats.total_bytes_freed += self.pending_bytes_to_free;
                            self.config.stats.total_files_deleted +=
                                crate::core::file_ops::FileOps::count_selected(
                                    &self.scanned_files,
                                ) as u64;
                            self.config.stats.last_clean_date = Some(chrono::Local::now());
                            self.config.save();
                            self.pending_bytes_to_free = 0;

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

            // Poll update background task events
            if let Some(ref mut rx) = self.update_rx {
                while let Ok(event) = rx.try_recv() {
                    match event {
                        crate::core::updater::UpdateEvent::ReleasesLoaded(releases) => {
                            self.update_releases = releases;
                            self.update_state = crate::core::updater::UpdateState::Loaded;
                            self.update_status = if self.update_releases.iter().any(|r| r.is_newer_than_current()) {
                                "Update available!".to_string()
                            } else {
                                "You are on the latest version.".to_string()
                            };
                            self.update_rx = None;
                            break;
                        }
                        crate::core::updater::UpdateEvent::Progress(msg) => {
                            self.update_status = msg;
                        }
                        crate::core::updater::UpdateEvent::Done => {
                            self.update_state = crate::core::updater::UpdateState::Done;
                            self.update_status =
                                "Update installed! Restart Clario to use the new version."
                                    .to_string();
                            self.update_rx = None;
                            break;
                        }
                        crate::core::updater::UpdateEvent::Error(err) => {
                            self.update_state =
                                crate::core::updater::UpdateState::Error(err.clone());
                            self.update_status = format!("Error: {}", err);
                            self.update_rx = None;
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

            // Auto-trigger file scan when entering FileManager with no data
            if self.mode == AppMode::FileManager
                && self.scanned_files.is_empty()
                && !self.is_scanning
            {
                self.is_scanning = true;
                let (tx, rx) = mpsc::channel();
                self.scan_rx = Some(rx);
                let threshold = self.config.safety_threshold_days;
                tokio::task::spawn_blocking(move || {
                    let targets = crate::utils::platform::get_scan_targets();
                    crate::core::file_scanner::FileScanner::scan_targets(&targets, threshold, tx);
                });
            }

            // Render UI sesuai mode
            terminal.draw(|f| {
                match self.mode {
                    AppMode::Dashboard => {
                        draw_dashboard(f, self)
                    }
                    AppMode::FileManager => {
                        draw_file_manager(f, self);
                    }
                    AppMode::Settings => draw_settings(f, self),
                    AppMode::AppUninstaller => {
                        draw_app_uninstaller(f, self);
                    }
                    AppMode::ProcessMonitor => draw_process_monitor(f, self),
                    AppMode::Update => draw_update(f, self),
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
                            } else if self.mode == AppMode::ProcessMonitor {
                                // q from ProcessMonitor returns to Dashboard (same as Esc/d)
                                self.mode = AppMode::Dashboard;
                                self.selected_menu = 0;
                                self.show_kill_confirm = false;
                                self.kill_status_message = None;
                            } else {
                                self.show_exit_confirm = true;
                                self.exit_confirm_selected = 1; // Focus "Wait, not yet" default
                            }
                            continue;
                        }
                        KeyCode::Char('f') => {
                            self.mode = AppMode::FileManager;
                            self.show_kill_confirm = false;
                            self.kill_status_message = None;
                            continue;
                        }
                        KeyCode::Char('u') => {
                            self.mode = AppMode::AppUninstaller;
                            self.show_kill_confirm = false;
                            self.kill_status_message = None;
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
                            self.show_kill_confirm = false;
                            self.kill_status_message = None;
                            continue;
                        }
                        KeyCode::Char('d') => {
                            self.mode = AppMode::Dashboard;
                            self.selected_menu = 0;
                            // Reset ProcessMonitor state when leaving
                            self.show_kill_confirm = false;
                            self.kill_status_message = None;
                            continue;
                        }
                        KeyCode::Char('p') => {
                            self.mode = AppMode::ProcessMonitor;
                            if self.processes.is_empty() {
                                self.processes = crate::core::process_scanner::ProcessScanner::scan(&self.sys);
                                if !self.processes.is_empty() {
                                    self.process_table_state.select(Some(0));
                                }
                            }
                            continue;
                        }
                        KeyCode::Char('?') => {
                            self.mode = AppMode::Update;
                            self.show_kill_confirm = false;
                            self.kill_status_message = None;
                            // Auto-check on first open
                            if self.update_state == crate::core::updater::UpdateState::Idle {
                                self.update_state = crate::core::updater::UpdateState::Checking;
                                self.update_status = "Checking for updates...".to_string();
                                let (tx, rx) = std::sync::mpsc::channel();
                                self.update_rx = Some(rx);
                                tokio::spawn(async move {
                                    match crate::core::updater::fetch_releases().await {
                                        Ok(releases) => {
                                            let _ = tx.send(crate::core::updater::UpdateEvent::ReleasesLoaded(releases));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(crate::core::updater::UpdateEvent::Error(e.to_string()));
                                        }
                                    }
                                });
                            }
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
                        AppMode::ProcessMonitor => handlers::process_monitor::handle_key(self, key),
                        AppMode::Update => handlers::update::handle_key(self, key),
                    }
                }
            }
        }

        Ok(())
    }
}
