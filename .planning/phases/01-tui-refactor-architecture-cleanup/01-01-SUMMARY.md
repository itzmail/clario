---
phase: 01-tui-refactor-architecture-cleanup
plan: "01"
subsystem: ui
tags: [ratatui, rust, tui, refactor, draw-functions]

# Dependency graph
requires: []
provides:
  - Single public centered_rect helper in src/ui/components.rs
  - Unified draw function signatures across all four screens
  - draw_dashboard(f, &App) — no more explicit selected_menu/sys/theme params
  - draw_file_manager(f, &mut App) — no more 12-param signature
  - draw_settings(f, &App) — unchanged, already correct
  - draw_app_uninstaller(f, &mut App) — unchanged, already correct
affects:
  - 01-02-PLAN (scan trigger extraction — builds on simplified FileManager callsite)
  - Any future plan adding new draw functions (must follow &App / &mut App pattern)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - draw functions take (f: &mut Frame, app: &App) for read-only screens
    - draw functions take (f: &mut Frame, app: &mut App) for stateful screens (table scroll)
    - shared UI helpers go in src/ui/components.rs as pub fn

key-files:
  created: []
  modified:
    - src/ui/components.rs
    - src/ui/file_manager.rs
    - src/ui/app_uninstaller.rs
    - src/ui/dashboard.rs
    - src/app.rs

key-decisions:
  - "Extract local variables (selected_menu, sys, theme) from app inside draw_dashboard body rather than changing all references — minimal diff, preserves readability"
  - "Keep AppTheme import in file_manager.rs because nested build_rows fn uses &AppTheme parameter type annotation"
  - "centered_rect_abs had a different implementation (absolute Rect::new) vs centered_rect (Layout-based) — replaced with centered_rect since callers pass same args and both produce same visual result"

patterns-established:
  - "draw_X(f, &App) pattern for read-only screens (dashboard, settings)"
  - "draw_X(f, &mut App) pattern for screens that mutate state during render (file_manager needs &mut file_table_state, app_uninstaller needs &mut app_table_state)"
  - "shared UI primitives live in src/ui/components.rs as pub fn"

requirements-completed: [REFAC-01, REFAC-02]

# Metrics
duration: 3min
completed: "2026-03-23"
---

# Phase 01 Plan 01: Unify Draw Signatures & Extract centered_rect Summary

**Collapsed 12-param draw_file_manager and inconsistent dashboard params into uniform (f, &App/&mut App) signatures, and consolidated three duplicate centered_rect implementations into one pub fn in components.rs**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-23T04:02:13Z
- **Completed:** 2026-03-23T04:05:11Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `centered_rect` is now a single `pub fn` in `src/ui/components.rs` — no copies in `file_manager.rs` or `app_uninstaller.rs`
- `draw_dashboard` signature changed from `(f, selected_menu, sys, theme)` to `(f, &App)` — callsite in app.rs is now `draw_dashboard(f, self)`
- `draw_file_manager` signature changed from 12 explicit parameters to `(f, &mut App)` — callsite in app.rs is now `draw_file_manager(f, self)`
- All four draw functions now follow a consistent 2-param pattern: `draw_X(f, &App)` or `draw_X(f, &mut App)`
- `cargo build` compiles with zero errors

## Task Commits

1. **Task 1: Extract centered_rect to public in components.rs and remove duplicates** - `9c567d0` (refactor)
2. **Task 2: Unify draw function signatures to &App / &mut App pattern** - `ce261ec` (refactor)

## Files Created/Modified

- `src/ui/components.rs` - Made `centered_rect` public (`fn` → `pub fn`)
- `src/ui/file_manager.rs` - Removed duplicate `fn centered_rect`, added `use crate::ui::components::centered_rect`, updated signature to `(f, &mut App)`, extracted params as locals
- `src/ui/app_uninstaller.rs` - Removed `fn centered_rect_abs`, added `use crate::ui::components::centered_rect`, replaced all `centered_rect_abs` calls with `centered_rect`
- `src/ui/dashboard.rs` - Updated signature to `(f, &App)`, replaced `use sysinfo::System` kept (needed for `System::long_os_version()`), extracted params as locals
- `src/app.rs` - Simplified draw callsites: `draw_dashboard(f, self)` and `draw_file_manager(f, self)`, removed 12-param call block

## Decisions Made

- Extracted local variables from `app` at the top of each refactored function body (e.g., `let selected_menu = app.selected_menu;`). This approach minimizes diff in the function body — all existing references to `selected_menu`, `theme`, etc. continue to work without per-use substitution.
- `centered_rect_abs` in `app_uninstaller.rs` used a different implementation (direct `Rect::new` with absolute coordinates) vs the Layout-based `centered_rect` in `components.rs`. Both produce the same visual result for centered modals, so replaced with the shared version.
- Kept `use sysinfo::System` in `dashboard.rs` because `System::long_os_version()` is a static method call that requires the type in scope even though `sys` is now accessed via `app.sys`.
- Kept `AppTheme` import in `file_manager.rs` because the inner nested `build_rows` function uses `theme: &AppTheme` in its parameter signature.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing sysinfo::System import in dashboard.rs after removing it**
- **Found during:** Task 2 (cargo build verification)
- **Issue:** Removed `use sysinfo::System` import since `sys` was no longer a direct param, but `System::long_os_version()` static method call on line 85 still needs the type in scope
- **Fix:** Re-added `use sysinfo::System` to imports
- **Files modified:** `src/ui/dashboard.rs`
- **Verification:** `cargo build` passes
- **Committed in:** `ce261ec` (Task 2 commit)

**2. [Rule 1 - Bug] Missing AppTheme import in file_manager.rs after changing imports**
- **Found during:** Task 2 (cargo build verification)
- **Issue:** Replaced `use crate::models::{config::AppTheme, file_info::{...}}` with individual imports but forgot `AppTheme` which is required by the nested `build_rows` function's `theme: &AppTheme` parameter
- **Fix:** Added `use crate::models::config::AppTheme` to imports
- **Files modified:** `src/ui/file_manager.rs`
- **Verification:** `cargo build` passes
- **Committed in:** `ce261ec` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - missing imports discovered during build verification)
**Impact on plan:** Both were import omissions surfaced by the compiler, fixed inline before final commit. No scope creep.

## Issues Encountered

None — both compilation errors were straightforward missing imports fixed immediately.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 01-02 can now proceed: the `draw_file_manager(f, self)` callsite is in place, and the scan trigger logic block (`if self.scanned_files.is_empty() && !self.is_scanning { ... }`) is still present in the FileManager branch of `app.rs` ready for extraction to a handler
- All draw function signatures are now consistent — future screens should follow the established `(f, &App)` or `(f, &mut App)` pattern

---
*Phase: 01-tui-refactor-architecture-cleanup*
*Completed: 2026-03-23*
