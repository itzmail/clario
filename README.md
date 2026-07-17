# Clario

Clario is a fast, keyboard-driven CLI for cleaning developer caches, build artifacts, and system clutter. Inspired by Mole — subcommand-driven, with an interactive TUI for the `analyze` disk browser.

## Features

- `clean` — clean per-language dev caches (Cargo, Node, Go, Python, Java, Ruby), Docker, app caches, and Trash.
- `purge` — sweep `node_modules`, `target`, `dist`, and other build artifacts across all your projects.
- `uninstall` — remove an application and its leftover files.
- `analyze` — interactive TUI to browse a directory and drill into what's taking up space.
- `update` — self-update to the latest release.

Safety: `clean`/`purge`/`analyze` delete operations are filtered through a critical-path guard (never touches system roots or, on macOS, protected app bundles) plus a user-defined whitelist at `~/.config/clario/whitelist`.

## Tech Stack

- Rust 2021
- `ratatui` + `crossterm` for the `analyze` TUI
- `tokio` + `reqwest` for async and self-update
- `walkdir`, `clap`, `serde`, `anyhow` for scanning, CLI parsing, models, and error handling

Supported platforms: macOS (x86_64, aarch64) and Linux (x86_64, aarch64).

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

```bash
clario <COMMAND>
```

Commands:

- `clario clean [cargo|node|go|python|java|ruby|docker|cache|trash]` — clean dev caches (all categories if none given). `--dry-run`, `--force`, `--min-size`.
- `clario purge` — sweep build artifacts across all projects. `--dry-run`, `--force`, `--min-size`, `--include-recent`, `--paths`.
- `clario uninstall [NAME]` — remove an app and its leftovers. `--list`, `--dry-run`, `--force`.
- `clario analyze [PATH]` — open the interactive disk-usage TUI (defaults to home).
- `clario update` — check for and install the latest release.

Run `clario <command> --help` for full options.

### Analyze TUI keys

- `↑↓` / `j k` — move selection
- `Enter` — open directory
- `Space` — multi-select
- `o` — open with system default app
- `p` — preview file/directory contents
- `Backspace` — delete selected (guarded by the safety filter)
- `Esc` — go up a level / back to categories
- `q` — quit

## Uninstall

To remove Clario itself and its local config:

```bash
curl -fsSL https://raw.githubusercontent.com/itzmail/clario/main/uninstall.sh | bash
```

## Project Structure

```text
clario/
├── src/
│   ├── cli/       # subcommand entry points (clean, purge, uninstall, analyze, update)
│   ├── core/      # scanners, protection guard, presets
│   ├── tui/       # analyze TUI (ratatui)
│   ├── models/    # shared data types
│   └── utils/     # paths, sizing, platform helpers
├── Cargo.toml
├── install.sh
└── uninstall.sh
```
