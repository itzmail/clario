---
phase: 01-tui-refactor-architecture-cleanup
plan: 03
subsystem: persistence-stats
tags: [config, stats, dashboard, persistence, serde]
dependency_graph:
  requires: ["01-02"]
  provides: ["CleanStats persistence", "real dashboard stats"]
  affects: ["src/models/config.rs", "src/app.rs", "src/ui/dashboard.rs", "src/handlers/file_manager.rs", "src/handlers/app_uninstaller.rs", "src/core/file_ops.rs"]
tech_stack:
  added: []
  patterns: ["serde #[serde(default)] for backward compat", "stats accumulation before in-memory cleanup"]
key_files:
  created: []
  modified:
    - src/models/config.rs
    - src/app.rs
    - src/core/file_ops.rs
    - src/handlers/file_manager.rs
    - src/handlers/app_uninstaller.rs
    - src/ui/dashboard.rs
decisions:
  - "Stats are recorded before retain_unselected/retain removes items from memory — avoids counting zero after cleanup"
  - "pending_bytes_to_free is set at confirm-time (not completion-time) to capture bytes before background thread removes files"
  - "Hardcoded score (85/100) replaced with generic Keep cleaning! text — real score requires future scoring logic"
metrics:
  duration: "3min"
  completed_date: "2026-03-23"
  tasks_completed: 2
  files_modified: 6
---

# Phase 01 Plan 03: CleanStats Persistence and Real Dashboard Statistics Summary

**One-liner:** Added CleanStats struct to AppConfig with serde backward compat, tracks lifetime delete/archive totals persisted to config.json, dashboard now renders real last_clean_date, total_files_deleted, and total_bytes_freed.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Add CleanStats struct to AppConfig with serde backward compatibility | fcfe9f8 | config.rs, app.rs, file_ops.rs |
| 2 | Calculate pending bytes before operations and update dashboard display | bd2469f | file_manager.rs, app_uninstaller.rs, dashboard.rs |

## What Was Built

**CleanStats struct (`src/models/config.rs`):**
- `last_clean_date: Option<DateTime<Local>>` — timestamp of most recent clean operation
- `total_files_deleted: u64` — lifetime cumulative file count
- `total_bytes_freed: u64` — lifetime cumulative bytes
- `#[serde(default)]` on the `stats` field in `AppConfig` — existing config.json files without a `stats` key load without error

**Stats accumulation (`src/app.rs`):**
- `pending_bytes_to_free: u64` field on `App` struct — set at confirm-time before background thread starts
- Delete completion block: increments `total_files_deleted` (mode-aware: AppUninstaller vs FileManager), adds `pending_bytes_to_free` to `total_bytes_freed`, sets `last_clean_date`, calls `config.save()`
- Archive completion block: same pattern for archive operations

**FileOps helpers (`src/core/file_ops.rs`):**
- `count_selected(files)` — recursively counts selected items in the tree
- `sum_selected_bytes(files)` — recursively sums bytes of selected items

**Bytes calculation (`src/handlers/file_manager.rs`, `src/handlers/app_uninstaller.rs`):**
- Set `pending_bytes_to_free` at every delete/archive confirmation path (Enter key and 'y' shortcut)
- AppUninstaller uses `total_size_bytes` from `AppInfo` (includes app bundle + related Library files)

**Real dashboard stats (`src/ui/dashboard.rs`):**
- Last Clean: elapsed time display (N days ago / N hours ago / Just now / Never)
- Files Deleted: `{n} files` from `app.config.stats.total_files_deleted`
- Space Freed: human-readable bytes with GB/MB/KB/B unit selection

## Decisions Made

- Stats are recorded BEFORE `retain_unselected` removes items from memory — avoids counting zero.
- `pending_bytes_to_free` is set at confirm-time (not completion-time) to capture accurate sizes before files are physically removed.
- The hardcoded score "85/100" was replaced with "Keep cleaning! ⚡" since computing a real score requires future design; this avoids misleading hardcoded data.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All stats fields are wired to real persistent data. The "Score" field was replaced with a generic motivational label rather than a stub numeric value.

## Self-Check: PASSED

- `src/models/config.rs` — FOUND, contains `pub struct CleanStats`
- `src/app.rs` — FOUND, contains `pending_bytes_to_free` and `config.stats.total_files_deleted`
- `src/ui/dashboard.rs` — FOUND, contains `app.config.stats.last_clean_date`
- Commit fcfe9f8 — FOUND
- Commit bd2469f — FOUND
- `cargo build` — 0 errors
