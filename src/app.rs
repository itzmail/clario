use crate::ui::{dashboard::draw_dashboard, file_manager::draw_file_manager, settings::draw_settings};
use crossterm::{execute, terminal::{
  EnterAlternateScreen, LeaveAlternateScreen
}};
use ratatui::{
  backend::CrosstermBackend,
  Terminal,
};

pub struct App {
    should_quit: bool,
    mode: AppMode,
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
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        use crossterm::event::{poll, read, Event, KeyCode};
        use std::time::Duration;

        while !self.should_quit {
            // Render UI sesuai mode
            terminal.draw(|f| {
                match self.mode {
                    AppMode::Dashboard => draw_dashboard::<CrosstermBackend<std::io::Stdout>>(f),
                    AppMode::FileManager => draw_file_manager::<CrosstermBackend<std::io::Stdout>>(f),
                    AppMode::Settings => draw_settings::<CrosstermBackend<std::io::Stdout>>(f),
                }
            })?;

            if poll(Duration::from_millis(100))? {
                if let Event::Key(key) = read()? {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true, // Tekan 'q' untuk keluar
                        KeyCode::Char('f') => self.mode = AppMode::FileManager, // 'f' untuk FileManager
                        KeyCode::Char('s') => self.mode = AppMode::Settings, // 's' untuk Settings
                        KeyCode::Char('d') => self.mode = AppMode::Dashboard, // 'd' untuk Dashboard
                        _ => {}
                    }
                }
            }
        }

        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }
}
