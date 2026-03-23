use crate::app::{App, AppMode};
use crate::core::file_ops::{FileOps, ToggleAction};
use crossterm::event::{KeyCode, KeyEvent};
use std::sync::mpsc;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Blokir semua input ketika sedang proses hapus data BESAR di background!
    if app.is_deleting || app.is_archiving {
        return;
    }

    match key.code {
        // Expand folder
        KeyCode::Right | KeyCode::Char('l') => {
            if app.show_delete_confirm {
                app.delete_confirm_selected = 1; // Nav kearah Cancel
            } else if app.show_archive_confirm {
                app.archive_confirm_selected = 1;
            } else if let Some(selected) = app.file_table_state.selected() {
                let mut count = 0;
                FileOps::toggle_recursive(
                    &mut app.scanned_files,
                    selected,
                    &mut count,
                    ToggleAction::Expand,
                );
            }
        }
        // Collapse folder atau navigasi Kiri di dalam modal
        KeyCode::Left | KeyCode::Char('h') => {
            if app.show_delete_confirm {
                app.delete_confirm_selected = 0; // Nav kearah Confirm Delete
            } else if app.show_archive_confirm {
                app.archive_confirm_selected = 0; // Nav kearah Confirm Archive
            }
        }
        KeyCode::Enter => {
            if app.show_delete_confirm {
                if app.delete_confirm_selected == 0 {
                    app.is_deleting = true;
                    // Reset text lama ke kosong saat inisiasi
                    app.delete_progress_text = String::new();
                    let (tx, rx) = mpsc::channel::<Option<String>>();
                    app.delete_rx = Some(rx);
                    FileOps::execute_deletion(&app.scanned_files, tx);
                }
                app.show_delete_confirm = false;
            } else if app.show_archive_confirm {
                if app.archive_confirm_selected == 0 {
                    app.is_archiving = true;
                    app.archive_progress_text = String::new();
                    let (tx, rx) = mpsc::channel::<Option<String>>();
                    app.archive_rx = Some(rx);
                    FileOps::execute_archiving(&app.scanned_files, tx);
                }
                app.show_archive_confirm = false;
            } else if let Some(selected) = app.file_table_state.selected() {
                let mut count = 0;
                FileOps::toggle_recursive(
                    &mut app.scanned_files,
                    selected,
                    &mut count,
                    ToggleAction::Expand,
                );
            }
        }
        KeyCode::Char(' ') => {
            if !app.show_delete_confirm {
                if let Some(selected) = app.file_table_state.selected() {
                    let mut count = 0;
                    FileOps::toggle_recursive(
                        &mut app.scanned_files,
                        selected,
                        &mut count,
                        ToggleAction::Select,
                    );
                }
            }
        }
        KeyCode::Char('x') | KeyCode::Delete | KeyCode::Backspace => {
            if !app.show_delete_confirm && FileOps::has_any_selected(&app.scanned_files) {
                app.show_delete_confirm = true;
                app.delete_confirm_selected = 1; // Default arah ke 'Cancel'
            }
        }
        KeyCode::Char('y') => {
            if app.show_delete_confirm {
                app.is_deleting = true;
                app.delete_progress_text = String::new();
                let (tx, rx) = mpsc::channel::<Option<String>>();
                app.delete_rx = Some(rx);
                FileOps::execute_deletion(&app.scanned_files, tx);
                app.show_delete_confirm = false;
            } else if app.show_archive_confirm {
                app.is_archiving = true;
                app.archive_progress_text = String::new();
                let (tx, rx) = mpsc::channel::<Option<String>>();
                app.archive_rx = Some(rx);
                FileOps::execute_archiving(&app.scanned_files, tx);
                app.show_archive_confirm = false;
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            if app.show_delete_confirm {
                app.show_delete_confirm = false;
            } else if app.show_archive_confirm {
                app.show_archive_confirm = false;
            } else if key.code == KeyCode::Esc {
                app.mode = AppMode::Dashboard; // Kemablui ke dashboard dari File Manager
            }
        }
        KeyCode::Char('a') => {
            if !app.show_delete_confirm
                && !app.show_archive_confirm
                && FileOps::has_any_selected(&app.scanned_files)
            {
                app.show_archive_confirm = true;
                app.archive_confirm_selected = 1; // Default arah ke 'Cancel'
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.show_delete_confirm && !app.show_archive_confirm {
                let i = match app.file_table_state.selected() {
                    Some(i) => i.saturating_add(1),
                    None => 0,
                };
                app.file_table_state.select(Some(i));
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !app.show_delete_confirm && !app.show_archive_confirm {
                let i = match app.file_table_state.selected() {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                app.file_table_state.select(Some(i));
            }
        }
        KeyCode::Char('r') => {
            if !app.show_delete_confirm && !app.show_archive_confirm {
                // Hapus data, maka app.rs akan otomatis men-trigger rescan di loop berikutnya
                app.scanned_files.clear();
                app.is_scanning = false;
            }
        }
        _ => {}
    }
}
