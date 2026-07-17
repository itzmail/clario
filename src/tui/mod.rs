mod app;
mod browser;
mod categories;
mod scan;

use anyhow::Result;
use app::{App, Overlay, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::path::PathBuf;
use std::time::Duration;

/// Run the Analyze TUI. `path` set skips the category screen and opens the
/// browser directly at that directory (`clario analyze <path>`).
pub fn run(path: Option<PathBuf>) -> Result<()> {
    let mut terminal = enter()?;
    let mut app = match path {
        Some(p) => App::new_browser(p),
        None => App::new_categories(),
    };

    let result = event_loop(&mut terminal, &mut app);

    leave(&mut terminal)?;
    result
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort: if we're already mid-panic-unwind, ignore further errors.
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
    }
}

fn enter() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    Ok(terminal)
}

fn leave(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    let _guard = TerminalGuard;

    loop {
        app.poll_scans();

        terminal.draw(|frame| {
            let area = frame.area();
            match &app.screen {
                Screen::Categories(s) => categories::draw(frame, area, s),
                Screen::Browser(s) => browser::draw(frame, area, s),
            }
        })?;

        if event::poll(Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        app.should_quit = true;
                    } else {
                        handle_key(app, key.code);
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_key(app: &mut App, code: KeyCode) {
    match &mut app.screen {
        Screen::Categories(_) => handle_categories_key(app, code),
        Screen::Browser(_) => handle_browser_key(app, code),
    }
}

fn handle_categories_key(app: &mut App, code: KeyCode) {
    let Screen::Categories(screen) = &mut app.screen else { return };
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if screen.selected > 0 {
                screen.selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if screen.selected + 1 < screen.rows.len() {
                screen.selected += 1;
            }
        }
        KeyCode::Enter => {
            let idx = screen.selected;
            app.enter_browser_from_category(idx);
        }
        KeyCode::Esc | KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_browser_key(app: &mut App, code: KeyCode) {
    let Screen::Browser(screen) = &mut app.screen else { return };

    // Overlay swallows input first.
    if !matches!(screen.overlay, Overlay::None) {
        handle_overlay_key(screen, code);
        return;
    }

    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if screen.selected > 0 {
                screen.selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if screen.selected + 1 < screen.entries.len() {
                screen.selected += 1;
            }
        }
        KeyCode::Char(' ') => {
            if !screen.entries.is_empty() {
                let idx = screen.selected;
                if !screen.multi_selected.remove(&idx) {
                    screen.multi_selected.insert(idx);
                }
            }
        }
        KeyCode::Enter => screen.enter_selected(),
        KeyCode::Esc => {
            let went_up = screen.go_up();
            if !went_up {
                *app = App::new_categories();
            }
        }
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('o') => open_selected(screen),
        KeyCode::Char('p') => preview_selected(screen),
        KeyCode::Backspace => start_delete(screen),
        _ => {}
    }
}

fn handle_overlay_key(screen: &mut app::BrowserScreen, code: KeyCode) {
    match &screen.overlay {
        Overlay::ConfirmDelete { .. } => match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => confirm_delete(screen),
            _ => screen.overlay = Overlay::None,
        },
        Overlay::Preview { .. } | Overlay::Message(_) => {
            screen.overlay = Overlay::None;
        }
        Overlay::None => {}
    }
}

fn selected_paths(screen: &app::BrowserScreen) -> Vec<PathBuf> {
    if screen.multi_selected.is_empty() {
        screen
            .entries
            .get(screen.selected)
            .map(|e| vec![e.path.clone()])
            .unwrap_or_default()
    } else {
        let mut idxs: Vec<_> = screen.multi_selected.iter().copied().collect();
        idxs.sort_unstable();
        idxs.into_iter().filter_map(|i| screen.entries.get(i).map(|e| e.path.clone())).collect()
    }
}

fn is_safe_to_delete(_path: &std::path::Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        crate::core::protection::is_safe_to_delete(_path)
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

fn start_delete(screen: &mut app::BrowserScreen) {
    let targets = selected_paths(screen);
    if targets.is_empty() {
        return;
    }

    let (safe, blocked): (Vec<_>, Vec<_>) = targets.into_iter().partition(|p| is_safe_to_delete(p));

    if safe.is_empty() {
        screen.overlay = Overlay::Message(format!("All {} selected item(s) are protected — nothing to delete.", blocked.len()));
        return;
    }

    let total_bytes: u64 = screen
        .entries
        .iter()
        .filter(|e| safe.contains(&e.path))
        .map(|e| e.size_bytes)
        .sum();

    screen.overlay = Overlay::ConfirmDelete { targets: safe, total_bytes, blocked };
}

fn confirm_delete(screen: &mut app::BrowserScreen) {
    let Overlay::ConfirmDelete { targets, .. } = &screen.overlay else { return };
    let targets = targets.clone();

    let mut failed = Vec::new();
    for path in &targets {
        if let Err(e) = trash::delete(path) {
            failed.push(format!("{}: {e}", path.display()));
        }
    }

    screen.overlay = if failed.is_empty() {
        Overlay::None
    } else {
        Overlay::Message(format!("Some items could not be deleted:\n{}", failed.join("\n")))
    };
    screen.rescan();
}

fn open_selected(screen: &app::BrowserScreen) {
    let Some(entry) = screen.entries.get(screen.selected) else { return };
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    let _ = std::process::Command::new(opener).arg(&entry.path).spawn();
}

fn preview_selected(screen: &mut app::BrowserScreen) {
    let Some(entry) = screen.entries.get(screen.selected) else { return };
    let path = entry.path.clone();
    let title = entry.name.clone();

    let lines = if entry.is_dir {
        let top = scan::preview_top_n(&path, 10);
        if top.is_empty() {
            vec!["(empty directory)".to_string()]
        } else {
            top.iter()
                .map(|f| format!("{:>10}  {}", crate::utils::size::format_size(f.size_bytes), f.name))
                .collect()
        }
    } else {
        preview_text_file(&path)
    };

    screen.overlay = Overlay::Preview { title, lines };
}

fn preview_text_file(path: &std::path::Path) -> Vec<String> {
    let Ok(content) = std::fs::read(path) else {
        return vec!["(could not read file)".to_string()];
    };
    if content.iter().take(1024).any(|&b| b == 0) {
        return vec!["(binary file, no preview)".to_string()];
    }
    String::from_utf8_lossy(&content)
        .lines()
        .take(20)
        .map(|l| l.to_string())
        .collect()
}
