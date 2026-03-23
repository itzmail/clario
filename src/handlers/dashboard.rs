use crate::app::{App, AppMode};
use crate::core::file_scanner::FileScanner;
use crate::utils::platform;
use crossterm::event::{KeyCode, KeyEvent};
use std::sync::mpsc;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.selected_menu = (app.selected_menu + 1) % 3;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.selected_menu = (app.selected_menu + 2) % 3;
        }
        // Konfirmasi Pilihan Menu
        KeyCode::Enter => {
            match app.selected_menu {
                0 => {
                    // Pindah UI ke File Manager
                    app.mode = AppMode::FileManager;

                    // Jalankan scan menggunakan background thread agar TUI tidak nge-freeze (loading screen tampil)!
                    if app.scanned_files.is_empty() && !app.is_scanning {
                        app.is_scanning = true;
                        let (tx, rx) = mpsc::channel();
                        app.scan_rx = Some(rx);
                        let targets = platform::get_scan_targets();
                        let threshold_days = app.config.safety_threshold_days;

                        // Spawn tokio background task thread
                        tokio::task::spawn_blocking(move || {
                            FileScanner::scan_targets(&targets, threshold_days, tx);
                        });
                    }
                }
                1 => {
                    // Pindah UI ke App Uninstaller
                    app.mode = AppMode::AppUninstaller;

                    // Kickoff scan hanya jika belum ada data apps yang di-cache
                    if app.apps.is_empty() && !app.is_scanning {
                        app.is_scanning = true;
                        app.scan_progress_text = String::new();
                        let (tx, rx) = mpsc::channel();
                        app.scan_rx = Some(rx);

                        tokio::task::spawn_blocking(move || {
                            let scanned_apps =
                                crate::core::app_scanner::AppScanner::scan_applications(tx.clone());
                            let _ = tx.send(crate::core::events::ScanEvent::FinishedApps(
                                scanned_apps,
                            ));
                        });
                    }
                }
                2 => app.mode = AppMode::Settings,
                _ => {}
            }
        }
        _ => {}
    }
}
