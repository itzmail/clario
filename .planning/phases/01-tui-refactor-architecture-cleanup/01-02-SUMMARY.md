---
phase: 01-tui-refactor-architecture-cleanup
plan: "02"
subsystem: core
tags: [rust, architecture, refactor, events, progress-fields, draw-closure]

# Dependency graph
requires:
  - 01-01 (unified draw signatures, draw_file_manager(f, self) callsite)
provides:
  - ScanEvent defined in src/core/events.rs (canonical location)
  - Scan trigger runs before terminal.draw() — no side-effects in render path
  - Three distinct progress fields: scan_progress_text, delete_progress_text, archive_progress_text
  - AppInfo and App structs free of unused fields (is_expanded, related_files_state removed)
affects:
  - Any future scanner additions must import ScanEvent from crate::core::events
  - UI overlays for delete/archive now read from operation-specific progress fields

# Tech tracking
tech-stack:
  added: []
  patterns:
    - ScanEvent lives in src/core/events.rs — all scan-related channel communication uses this module
    - Background I/O trigger goes BEFORE terminal.draw(), not inside the closure
    - Per-operation progress fields: scan_progress_text (scan), delete_progress_text (delete), archive_progress_text (archive)

key-files:
  created:
    - src/core/events.rs
  modified:
    - src/core/mod.rs
    - src/models/file_info.rs
    - src/models/app_info.rs
    - src/core/file_scanner.rs
    - src/core/app_scanner.rs
    - src/app.rs
    - src/handlers/dashboard.rs
    - src/handlers/file_manager.rs
    - src/handlers/app_uninstaller.rs
    - src/ui/file_manager.rs
    - src/ui/app_uninstaller.rs

key-decisions:
  - "ScanEvent moved to src/core/events.rs — belongs in core (logic) not models (data), since it is a channel message type not a domain entity"
  - "Used crate::core::events::ScanEvent full paths in app.rs match arms for clarity, consistent with existing style across file"
  - "archive_progress_text and delete_progress_text initialized to String::new() in App::new() for consistency — no lazy init"

requirements-completed: [REFAC-03, REFAC-04, REFAC-05, REFAC-06]

# Metrics
duration: 8min
completed: "2026-03-23"
---

# Phase 01 Plan 02: Fix Architectural Smells — ScanEvent Migration, Scan Trigger, Progress Fields Summary

**Moved ScanEvent to core/events.rs, extracted file scan trigger out of terminal.draw() closure, split scan_progress_text into three per-operation fields, and removed dead fields AppInfo::is_expanded and App::related_files_state**

## Performance

- **Duration:** ~8 min
- **Completed:** 2026-03-23
- **Tasks:** 2
- **Files created:** 1
- **Files modified:** 10

## Accomplishments

- `src/core/events.rs` created with canonical `ScanEvent` enum definition
- All 5 import sites updated: `file_scanner.rs`, `app_scanner.rs`, `app.rs` (3 match arms + scan_rx type + FinishedApps send), `handlers/dashboard.rs`
- `ScanEvent` fully removed from `src/models/file_info.rs`
- File scan trigger (`spawn_blocking` + channel setup) moved BEFORE `terminal.draw()` — render closure is now side-effect free
- Three distinct progress fields on `App`: `scan_progress_text`, `delete_progress_text`, `archive_progress_text`
- `delete_rx` and `archive_rx` drain loops write to their dedicated fields
- Handler code (`file_manager.rs`, `app_uninstaller.rs`) initializes the correct field before launching each operation
- UI overlays select the correct field: `delete_progress_text` when `is_deleting`, `archive_progress_text` when `is_archiving`, `scan_progress_text` for scan loading
- `AppInfo::is_expanded` and `App::related_files_state` removed — zero dead field warnings from these
- `cargo build` compiles with zero errors

## Task Commits

1. **Task 1: Move ScanEvent to core/events.rs and remove unused fields** - `7748cfe` (refactor)
2. **Task 2: Move scan trigger out of draw closure and split progress text fields** - `2263c16` (refactor)

## Files Created/Modified

- `src/core/events.rs` — New file: `pub enum ScanEvent { Progress, Finished, FinishedApps }`
- `src/core/mod.rs` — Added `pub mod events;`
- `src/models/file_info.rs` — Removed `ScanEvent` enum block (lines 4-8)
- `src/models/app_info.rs` — Removed `is_expanded: bool` field and its `new()` initializer
- `src/core/file_scanner.rs` — Updated import from `file_info::ScanEvent` to `core::events::ScanEvent`
- `src/core/app_scanner.rs` — Updated import and function signature to use `ScanEvent` directly
- `src/app.rs` — Updated scan_rx type, all 3 match arms, FinishedApps send; added delete/archive_progress_text fields; moved scan trigger before draw; removed related_files_state
- `src/handlers/dashboard.rs` — Updated FinishedApps send path
- `src/handlers/file_manager.rs` — Changed 4 occurrences of scan_progress_text init to delete/archive_progress_text
- `src/handlers/app_uninstaller.rs` — Changed scan_progress_text init to delete_progress_text
- `src/ui/file_manager.rs` — Added delete/archive_progress_text locals; updated delete/archive overlay to read correct field
- `src/ui/app_uninstaller.rs` — Updated delete overlay to read delete_progress_text

## Decisions Made

- `ScanEvent` belongs in `core/` not `models/` — it is a channel message type for background worker communication, not a domain data entity. Moving it to `core/events.rs` makes the dependency direction correct: core modules import from models, not the other way around.
- Used full paths (`crate::core::events::ScanEvent`) in `app.rs` match arms to stay consistent with the surrounding codebase style (other full-path references exist nearby).
- `archive_progress_text` and `delete_progress_text` both initialize to `String::new()` in `App::new()` — lazy init would add complexity with no benefit given these are cheap empty strings.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ScanEvent reference in handlers/dashboard.rs not listed in plan**
- **Found during:** Task 1 (cargo build verification)
- **Issue:** `src/handlers/dashboard.rs` line 50 had a `crate::models::file_info::ScanEvent::FinishedApps` send that the plan did not list as an import site to update
- **Fix:** Updated to `crate::core::events::ScanEvent::FinishedApps`
- **Files modified:** `src/handlers/dashboard.rs`
- **Commit:** `7748cfe` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — missed import site discovered during build verification)
**Impact on plan:** Single-line fix, no scope change.

## Known Stubs

None — all three progress text fields are wired to real channel data from background threads.

## Self-Check: PASSED

- FOUND: src/core/events.rs
- FOUND: .planning/phases/01-tui-refactor-architecture-cleanup/01-02-SUMMARY.md
- FOUND: commit 7748cfe (Task 1)
- FOUND: commit 2263c16 (Task 2)
