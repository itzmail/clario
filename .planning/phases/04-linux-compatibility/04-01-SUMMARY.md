---
phase: 04-linux-compatibility
plan: 01
subsystem: platform-compatibility
tags: [rust, cfg, cross-platform, linux, macos, ratatui, reqwest, plist]

# Dependency graph
requires:
  - phase: 02-security-process-monitor
    provides: process_scanner.rs with TRUSTED_PATHS and ROOT_SYSTEM_PATHS constants

provides:
  - reqwest without native-tls/openssl (default-features=false, rustls-tls only)
  - plist crate gated to macOS-only target dependency
  - app_scanner module gated behind #![cfg(target_os = "macos")]
  - process_scanner with platform-specific TRUSTED_PATHS for linux (/opt/, /snap/, /nix/)
  - App Uninstaller graceful degradation message on non-macOS
  - platform-neutral UI text (system instead of Mac)
  - Dashboard menu label appends "(macOS)" for Deep App Uninstaller on linux

affects: [05-cli-compat, tauri-desktop-build, any-phase-adding-new-ui-text]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "cfg(target_os) gating for macOS-only modules at file level (#![cfg])"
    - "target.'cfg(...)'.dependencies in Cargo.toml for platform-specific crates"
    - "Inline #[cfg] blocks inside fn bodies for platform-specific spawn logic"

key-files:
  created: []
  modified:
    - Cargo.toml
    - src/core/app_scanner.rs
    - src/core/mod.rs
    - src/app.rs
    - src/core/process_scanner.rs
    - src/ui/file_manager.rs
    - src/ui/app_uninstaller.rs
    - src/ui/dashboard.rs

key-decisions:
  - "reqwest default-features=false: removes native-tls/openssl dependency that fails on Linux without libssl-dev"
  - "plist moved to [target.cfg(target_os=macos).dependencies] so it is not even resolved on Linux"
  - "app_scanner.rs gated with #![cfg(target_os=macos)] at file level — cleaner than wrapping every item"
  - "apps Vec<AppInfo> stays available on all platforms (empty on Linux) — avoids type-gating the App struct"
  - "FinishedApps event handler kept as-is in event_loop — will never fire on Linux so no functional risk"

patterns-established:
  - "Module-level cfg gate: add #![cfg(target_os = \"macos\")] as first line of macOS-only modules"
  - "Cargo platform deps: use [target.'cfg(...)'.dependencies] for crates only buildable on one OS"
  - "Graceful degradation UI: on non-macOS, render early-return message rather than empty table"

requirements-completed: [LINUX-01, LINUX-02, LINUX-03, LINUX-04]

# Metrics
duration: 3min
completed: 2026-03-29
---

# Phase 04 Plan 01: Linux Compatibility Foundation Summary

**Clario now compiles for Linux targets: plist and native-tls gated to macOS, app_scanner module fully cfg-guarded, process scanner has Linux-specific trusted paths, and all UI text is platform-neutral.**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-29T10:23:34Z
- **Completed:** 2026-03-29T10:26:14Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Fixed reqwest to use `default-features = false` eliminating native-tls/openssl dependency that blocks Linux builds
- Moved plist crate to macOS-only target dependency section so Linux builds don't require libplist
- Gated entire `app_scanner` module behind `#![cfg(target_os = "macos")]` at file level and `#[cfg]` in mod.rs
- Replaced macOS-only app scan spawn in app.rs with `#[cfg(target_os = "macos")]` block
- Added Linux-specific TRUSTED_PATHS (`/opt/`, `/snap/`, `/flatpak/`, `/nix/`) and ROOT_SYSTEM_PATHS with `/usr/lib/systemd/`
- Changed "Scanning your Mac" to "Scanning your system" in file_manager.rs
- Added graceful degradation message in app_uninstaller.rs for non-macOS platforms
- Added "(macOS)" suffix to Deep App Uninstaller dashboard label on non-macOS

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix Cargo.toml dependencies and cfg-gate macOS-only modules** - `b38f2e6` (feat)
2. **Task 2: Adapt process scanner trusted paths and platform-aware UI text** - `06b68cd` (feat)

**Plan metadata:** (docs: complete plan — see below)

## Files Created/Modified

- `Cargo.toml` - reqwest default-features=false; plist moved to macOS target dep section
- `src/core/app_scanner.rs` - `#![cfg(target_os = "macos")]` added as first line
- `src/core/mod.rs` - `#[cfg(target_os = "macos")]` before `pub mod app_scanner`
- `src/app.rs` - `#[cfg(target_os = "macos")]` wraps app scan spawn block
- `src/core/process_scanner.rs` - Platform-specific TRUSTED_PATHS and ROOT_SYSTEM_PATHS
- `src/ui/file_manager.rs` - "Scanning your system" instead of "Scanning your Mac"
- `src/ui/app_uninstaller.rs` - Non-macOS early-return with "macOS-only feature" message
- `src/ui/dashboard.rs` - "(macOS)" label suffix on Deep App Uninstaller for non-macOS

## Decisions Made

- reqwest `default-features = false` removes native-tls/openssl; rustls-tls alone is sufficient and builds on Linux without system SSL libraries
- plist moved to `[target.'cfg(target_os = "macos")'.dependencies]` so it is not fetched or compiled on Linux at all
- app_scanner.rs uses file-level `#![cfg]` rather than item-level `#[cfg]` on each pub item — simpler and intent is clearer
- `apps: Vec<AppInfo>` field in App struct kept on all platforms (stays empty on Linux) — avoids requiring `#[cfg]` throughout all consumers

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. `cargo check` passed on first attempt after each change, all 19 tests still pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 04-02 (CLI Linux compatibility) can proceed: compile-blocking issues are now resolved
- On Linux, App Uninstaller shows a clear platform message instead of hanging on empty scan
- Process scanner is ready to evaluate Linux processes with appropriate trusted path definitions

---
*Phase: 04-linux-compatibility*
*Completed: 2026-03-29*

## Self-Check: PASSED

- FOUND: Cargo.toml
- FOUND: src/core/app_scanner.rs
- FOUND: src/core/mod.rs
- FOUND: src/app.rs
- FOUND: src/core/process_scanner.rs
- FOUND: 04-01-SUMMARY.md
- FOUND commit: b38f2e6
- FOUND commit: 06b68cd
