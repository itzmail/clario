# Clario

Clario is a terminal-based system cleaner (TUI) built with Rust. It is designed as a fast, keyboard-driven cleanup utility inspired by desktop cleaner apps.

## Features

- Dashboard for quick system overview.
- File Manager to scan cleanup targets (cache, temp, logs).
- App Uninstaller to detect installed apps and remove related files.
- Process Monitor to inspect and manage running processes.
- Settings page for theme and safety preferences.

## Tech Stack

- Rust 2021
- `ratatui` + `crossterm` for terminal UI
- `tokio` for async runtime
- `sysinfo`, `walkdir`, `serde`, `anyhow` for scanning, models, and error handling

## Installation

### Option 1: Install latest release (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/itzmail/clario/main/install.sh | bash
```

The installer will:

- Detect your OS and architecture.
- Download the latest release binary from GitHub.
- Install `clario` to `~/.local/bin`.

If `~/.local/bin` is not in your `PATH`, add this to your shell profile:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Then reload your shell or open a new terminal.

### Option 2: Build from source

Prerequisite: Rust and Cargo installed.

```bash
cargo build --release
```

Run with:

```bash
./target/release/clario
```

## Usage

After installation, start Clario with:

```bash
clario
```

Basic hotkeys:

- `d` go to Dashboard
- `f` go to File Manager
- `u` go to App Uninstaller
- `p` go to Process Monitor
- `s` go to Settings
- `q` quit or open exit confirmation (depends on current mode)

## Uninstall

To remove Clario and its local data:

```bash
curl -fsSL https://raw.githubusercontent.com/itzmail/clario/main/uninstall.sh | bash
```

## Project Structure

```text
clario/
├── src/
├── Cargo.toml
├── PLAN.md
├── AGENTS.md
├── install.sh
└── uninstall.sh
```
