use crate::app::{App, AppMode};
use crate::models::file_info::FileInfo;
use crossterm::event::{KeyCode, KeyEvent};
use std::fs;
use std::path::Path;

fn read_directories(path: &Path) -> Vec<FileInfo> {
    let mut dirs = Vec::new();

    // Selalu tambahkan parent ".." di paling atas (kecuali di root "/").
    if let Some(parent) = path.parent() {
        dirs.push(FileInfo::new(
            "..".to_string(),
            parent.to_path_buf(),
            0,
            true,
        ));
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip file hidden (berawalan '.') dan pastikan hanya folder
            if name.starts_with('.') || !path.is_dir() {
                continue;
            }

            dirs.push(FileInfo::new(name, path, 0, true));
        }
    }

    // Sort: ".." selalu di atas, sisanya A-Z
    dirs.sort_by(|a, b| {
        if a.name == ".." {
            std::cmp::Ordering::Less
        } else if b.name == ".." {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    dirs
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.is_dir_picker_open {
        handle_picker_key(app, key);
    } else {
        handle_normal_key(app, key);
    }
}

fn handle_picker_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.is_dir_picker_open = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.dir_picker_selected = app.dir_picker_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.dir_picker_selected + 1 < app.dir_picker_items.len() {
                app.dir_picker_selected += 1;
            }
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            if let Some(selected) = app.dir_picker_items.get(app.dir_picker_selected) {
                app.dir_picker_path = selected.path.clone();
                app.dir_picker_selected = 0;
                app.dir_picker_items = read_directories(&app.dir_picker_path);
            }
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace => {
            if let Some(parent) = app.dir_picker_path.parent() {
                app.dir_picker_path = parent.to_path_buf();
                app.dir_picker_selected = 0;
                app.dir_picker_items = read_directories(&app.dir_picker_path);
            }
        }
        KeyCode::Char(' ') => {
            // Confirm selection
            app.config.archive_dir = app.dir_picker_path.clone();
            app.is_dir_picker_open = false;
            app.config.save();
        }
        _ => {}
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            // Autowrite & back to dashboard
            app.config.save();
            app.mode = AppMode::Dashboard;
        }
        KeyCode::Enter => {
            if app.settings_selected_index == 1 {
                app.is_dir_picker_open = true;
                app.dir_picker_path = app.config.archive_dir.clone();
                if !app.dir_picker_path.exists() {
                    app.dir_picker_path =
                        dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
                }
                app.dir_picker_selected = 0;
                app.dir_picker_items = read_directories(&app.dir_picker_path);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.settings_selected_index = app.settings_selected_index.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            // We have 3 settings options for now
            if app.settings_selected_index < 2 {
                app.settings_selected_index += 1;
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if app.settings_selected_index == 0 {
                app.config.theme = app.config.theme.prev();
            } else if app.settings_selected_index == 2 {
                if app.config.safety_threshold_days > 0 {
                    app.config.safety_threshold_days -= 1;
                }
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.settings_selected_index == 0 {
                app.config.theme = app.config.theme.next();
            } else if app.settings_selected_index == 2 {
                app.config.safety_threshold_days += 1;
            }
        }
        _ => {}
    }
}
