use crate::app::{App, AppMode};
use crate::core::updater::{UpdateEvent, UpdateState};
use crossterm::event::{KeyCode, KeyEvent};
use std::sync::mpsc;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Dashboard;
            app.selected_menu = 0;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.update_selected = app.update_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.update_selected + 1 < app.update_releases.len() {
                app.update_selected += 1;
            }
        }
        KeyCode::Char('r') => {
            // Re-check for updates
            if app.update_state != UpdateState::Checking
                && app.update_state != UpdateState::Downloading
            {
                app.update_state = UpdateState::Checking;
                app.update_status = "Checking for updates...".to_string();
                app.update_releases.clear();
                app.update_selected = 0;
                let (tx, rx) = mpsc::channel();
                app.update_rx = Some(rx);
                tokio::spawn(async move {
                    match crate::core::updater::fetch_releases().await {
                        Ok(releases) => {
                            let _ = tx.send(UpdateEvent::ReleasesLoaded(releases));
                        }
                        Err(e) => {
                            let _ = tx.send(UpdateEvent::Error(e.to_string()));
                        }
                    }
                });
            }
        }
        KeyCode::Enter => {
            // Install the selected release
            if app.update_state == UpdateState::Downloading
                || app.update_state == UpdateState::Checking
            {
                return;
            }
            if let Some(release) = app.update_releases.get(app.update_selected).cloned() {
                let tag = release.tag_name.clone();
                app.update_state = UpdateState::Downloading;
                app.update_status = format!("Starting download of {}...", tag);
                let (tx, rx) = mpsc::channel();
                app.update_rx = Some(rx);
                tokio::spawn(async move {
                    if let Err(e) =
                        crate::core::updater::download_and_install(&tag, tx.clone()).await
                    {
                        let _ = tx.send(UpdateEvent::Error(e.to_string()));
                    }
                });
            }
        }
        _ => {}
    }
}
