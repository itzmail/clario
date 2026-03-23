use crate::app::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent};
use std::sync::mpsc;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Jika modal exit terbuka (meskipun ini punya logic intercept tersendiri di run loop), untuk jaga-jaga block interaksi lain
    if app.show_exit_confirm {
        return;
    }

    // Modal Konfirmasi Hapus
    if app.show_delete_confirm {
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                app.delete_confirm_selected = 1 - app.delete_confirm_selected;
            }
            KeyCode::Enter => {
                if app.delete_confirm_selected == 0 {
                    app.is_deleting = true;
                    app.delete_progress_text = String::new();

                    // Kita bisa memakai execute_deletion milik mod lama yang sudah mumpuni (memproses array of Files),
                    // Tapi di struct AppInfo aslinya berbeda.
                    // Nanti kita akan extract files nya dan kirim ke worker.
                    let (tx, rx) = mpsc::channel::<Option<String>>();
                    app.delete_rx = Some(rx);

                    let mut delete_payload = Vec::new();
                    // Ambil file yang terklik centang
                    for app_obj in &app.apps {
                        if app_obj.is_selected {
                            // 1. Delete Root Bundle
                            let mut info = crate::models::file_info::FileInfo::new(
                                app_obj.name.clone(),
                                app_obj.path.clone(),
                                app_obj.app_size_bytes,
                                true,
                            );
                            info.is_selected = true;
                            delete_payload.push(info);

                            // 2. Delete Relational Libraries
                            for relation in &app_obj.related_files {
                                let mut rel = relation.clone();
                                rel.is_selected = true;
                                delete_payload.push(rel);
                            }
                        }
                    }

                    if !delete_payload.is_empty() {
                        crate::core::file_ops::FileOps::execute_deletion(&delete_payload, tx);
                    } else {
                        // Tidak jadi ada yang dihapus (prevent stuck loader)
                        let _ = tx.send(None);
                    }
                }
                app.show_delete_confirm = false;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                app.show_delete_confirm = false;
                app.delete_confirm_selected = 1;
            }
            _ => {}
        }
        return;
    }

    // Modal Wait / Loader Hapus
    if app.is_deleting || app.is_scanning {
        // Blokir semua input saat memburu directory
        return;
    }

    // Navigasi Standar Tabel Aplikasi (Utama)
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.apps.is_empty() {
                app.selected_app_index = (app.selected_app_index + 1) % app.apps.len();
                app.app_table_state.select(Some(app.selected_app_index));
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !app.apps.is_empty() {
                if app.selected_app_index == 0 {
                    app.selected_app_index = app.apps.len() - 1;
                } else {
                    app.selected_app_index -= 1;
                }
                app.app_table_state.select(Some(app.selected_app_index));
            }
        }
        KeyCode::Char(' ') => {
            // Centang status selected
            if let Some(app_obj) = app.apps.get_mut(app.selected_app_index) {
                app_obj.is_selected = !app_obj.is_selected;
            }
        }
        KeyCode::Delete | KeyCode::Backspace | KeyCode::Char('x') => {
            // Cek apakah ada yg divalidasi?
            let has_selected = app.apps.iter().any(|a| a.is_selected);
            if has_selected {
                app.show_delete_confirm = true;
                app.delete_confirm_selected = 1; // Default cursor NO safe mode
            }
        }
        KeyCode::Esc => {
            // Kembali ke Dashboard
            app.mode = AppMode::Dashboard;
        }
        _ => {}
    }
}
