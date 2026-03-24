use crate::app::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // If kill confirm modal is active, intercept all keys
    if app.show_kill_confirm {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                app.kill_confirm_selected = if app.kill_confirm_selected == 0 {
                    2
                } else {
                    app.kill_confirm_selected - 1
                };
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.kill_confirm_selected = (app.kill_confirm_selected + 1) % 3;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                app.show_kill_confirm = false;
                app.kill_confirm_selected = 0;
            }
            KeyCode::Enter => {
                match app.kill_confirm_selected {
                    0 => {
                        app.show_kill_confirm = false;
                    }
                    1 => {
                        execute_kill(app, sysinfo::Signal::Term);
                    }
                    2 => {
                        execute_kill(app, sysinfo::Signal::Kill);
                    }
                    _ => {}
                }
                app.kill_confirm_selected = 0;
            }
            _ => {}
        }
        return;
    }

    // Normal mode key handling
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.processes.is_empty() {
                app.selected_process_index =
                    (app.selected_process_index + 1) % app.processes.len();
                app.process_table_state
                    .select(Some(app.selected_process_index));
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !app.processes.is_empty() {
                app.selected_process_index = if app.selected_process_index == 0 {
                    app.processes.len() - 1
                } else {
                    app.selected_process_index - 1
                };
                app.process_table_state
                    .select(Some(app.selected_process_index));
            }
        }
        KeyCode::Char(' ') => {
            // Toggle selection on current process (multi-select)
            if let Some(proc) = app.processes.get_mut(app.selected_process_index) {
                proc.is_selected = !proc.is_selected;
            }
        }
        KeyCode::Char('x') => {
            // Open kill modal only if at least one process is selected
            let has_selected = app.processes.iter().any(|p| p.is_selected);
            if has_selected {
                app.show_kill_confirm = true;
                app.kill_confirm_selected = 0; // Default to Cancel (safe)
                app.kill_status_message = None; // Clear previous status
            }
        }
        KeyCode::Char('r') => {
            // Manual refresh
            app.processes =
                crate::core::process_scanner::ProcessScanner::scan(&app.sys);
            if app.selected_process_index >= app.processes.len() {
                app.selected_process_index = app.processes.len().saturating_sub(1);
            }
            app.process_table_state
                .select(Some(app.selected_process_index));
            app.kill_status_message = None;
        }
        KeyCode::Esc => {
            // Return to dashboard
            app.mode = AppMode::Dashboard;
            app.selected_menu = 0;
            app.show_kill_confirm = false;
            app.kill_status_message = None;
        }
        _ => {}
    }
}

fn execute_kill(app: &mut App, signal: sysinfo::Signal) {
    let pids: Vec<sysinfo::Pid> = app
        .processes
        .iter()
        .filter(|p| p.is_selected)
        .map(|p| p.pid)
        .collect();

    let signal_name = match signal {
        sysinfo::Signal::Term => "SIGTERM",
        sysinfo::Signal::Kill => "SIGKILL",
        _ => "signal",
    };

    let mut errors: Vec<String> = Vec::new();
    let mut killed: u32 = 0;

    for pid in &pids {
        if let Some(process) = app.sys.process(*pid) {
            match process.kill_with(signal) {
                Some(true) => {
                    killed += 1;
                }
                Some(false) => {
                    errors.push(format!("PID {}: failed", pid));
                }
                None => {
                    errors.push(format!("PID {}: signal not supported", pid));
                }
            }
        } else {
            errors.push(format!("PID {}: process not found", pid));
        }
    }

    if !errors.is_empty() {
        app.kill_status_message = Some(format!(
            "{} sent to {} process(es), errors: {}",
            signal_name,
            killed,
            errors.join(", ")
        ));
    } else {
        app.kill_status_message = Some(format!(
            "{} sent to {} process(es) successfully",
            signal_name, killed
        ));
    }

    // Refresh process list after kill
    app.sys
        .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    app.processes = crate::core::process_scanner::ProcessScanner::scan(&app.sys);

    // Reset selection index to be in bounds
    app.selected_process_index = app
        .selected_process_index
        .min(app.processes.len().saturating_sub(1));
    app.process_table_state
        .select(Some(app.selected_process_index));
    app.show_kill_confirm = false;
}
