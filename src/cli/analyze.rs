use anyhow::Result;
use std::path::PathBuf;

/// Entry point for `clario analyze` — a full-screen Mole-style disk browser
/// (ratatui TUI, not a one-shot report). With no path, starts on the preset
/// category screen; with a path, jumps straight into the browser there.
pub async fn run_analyze(path: Option<PathBuf>) -> Result<()> {
    crate::tui::run(path)
}
