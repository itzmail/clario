---
phase: 02-security-process-monitor
plan: 02
subsystem: ui
tags: [ratatui, sysinfo, process-monitor, tui, kill-modal, multi-select]

requires:
  - phase: 02-security-process-monitor/02-01
    provides: ProcessInfo model, SuspicionFlag enum, ProcessScanner::scan, format_memory, format_uptime

provides:
  - Process Monitor TUI screen with left/right panel layout (draw_process_monitor)
  - Key handler for ProcessMonitor mode with multi-select and kill flow (handle_key)
  - AppMode::ProcessMonitor variant in app state
  - Dashboard integration: 4th menu item, [p] hotkey in footer, modulo-4 navigation
  - 3-button kill confirmation modal (Cancel / Graceful Kill / Force Kill)

affects:
  - 03-vulnerability-audit
  - any phase adding new AppMode variants

tech-stack:
  added: []
  patterns:
    - "Left 65% / Right 35% panel split for list+detail TUI screens"
    - "3-button modal with Left/Right navigation and Enter confirmation"
    - "Kill key ('x') opens modal only when at least one process is selected"
    - "Post-kill: sys.refresh_processes + re-scan to reflect updated process list"
    - "Footer status bar for async operation results (kill success/error message)"

key-files:
  created:
    - src/ui/process_monitor.rs
    - src/handlers/process_monitor.rs
  modified:
    - src/app.rs
    - src/ui/dashboard.rs
    - src/handlers/dashboard.rs
    - src/ui/mod.rs
    - src/handlers/mod.rs

key-decisions:
  - "Kill hotkey is 'x' not 'k' — avoids conflict with vim-style Up navigation ('k'); footer updated to match"
  - "Kill modal defaults to Cancel (index 0) for safety — user must actively move to destructive options"
  - "Process list uses manual refresh ('r') for v1 — avoids background thread complexity on initial release"
  - "exe=None shown as 'Unknown (SIP protected)' — avoids alarming users about normal macOS behavior"

patterns-established:
  - "Process screen: draw fn takes &mut App (stateful table) following established draw_app_uninstaller pattern"
  - "Modal buttons rendered with centered_rect + Layout::Horizontal + 3 Constraint::Percentage(33/34/33)"

requirements-completed: [SEC-01, SEC-04, SEC-05]

duration: 15min
completed: 2026-03-27
---

# Phase 2 Plan 2: Process Monitor TUI Screen Summary

**Full Process Monitor screen with 65/35 left-right panel layout, suspicious-process color coding, multi-select, and 3-button (Cancel/Graceful Kill/Force Kill) kill modal using SIGTERM/SIGKILL via sysinfo**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-27T00:00:00Z
- **Completed:** 2026-03-27
- **Tasks:** 2
- **Files modified:** 7 (2 created, 5 modified)

## Accomplishments
- Process Monitor accessible from dashboard via `p` hotkey and Enter on 4th menu item (SEC-01)
- Left panel (65%) table shows Name/PID/CPU%/RAM with color-coded severity rows (Clean/Warning/Danger)
- Right panel (35%) detail view includes Why Suspicious section listing each SuspicionFlag reason
- Multi-select with Space, kill modal opens with `x` key — 3-button layout with Left/Right navigation
- Kill execution sends SIGTERM (Graceful) or SIGKILL (Force) via sysinfo, errors shown in status bar
- Manual refresh via `r` re-runs ProcessScanner::scan and resets selection bounds
- Dashboard footer updated with `[p] Processes` and modulo-4 menu navigation

## Task Commits

1. **Task 1: AppMode + dashboard integration** - already committed prior to execution
2. **Task 2: Process Monitor screen and key handler** - already committed prior to execution
3. **Bug fix: footer kill key hint** - `bf41cf2` (fix)

## Files Created/Modified
- `src/ui/process_monitor.rs` (370 lines) — draw_process_monitor with header, 65/35 panels, status bar, kill modal
- `src/handlers/process_monitor.rs` (161 lines) — handle_key, modal key routing, execute_kill with sysinfo
- `src/app.rs` — AppMode::ProcessMonitor, process/kill fields, 'p' hotkey dispatch
- `src/ui/dashboard.rs` — 4th menu item (Process Monitor), m4_style, [p] footer entry
- `src/handlers/dashboard.rs` — modulo-4 navigation, Enter arm 3 -> ProcessMonitor
- `src/ui/mod.rs` — pub mod process_monitor
- `src/handlers/mod.rs` — pub mod process_monitor

## Decisions Made
- Kill hotkey is `'x'` (not `'k'`) to avoid conflict with vim-style Up navigation. Footer hint corrected to show `[x] Kill`.
- Modal defaults to Cancel index 0 — user must explicitly navigate to destructive kill options.
- Process list refreshes manually via `r` in v1 — no background polling thread added.
- `exe=None` renders as "Unknown (SIP protected)" — correct context for macOS SIP-restricted processes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed footer kill key hint mismatch**
- **Found during:** Task 2 verification
- **Issue:** Footer showed `[k] Kill` but handler bound kill modal to `'x'` key (to avoid conflict with vim Up navigation `'k'`)
- **Fix:** Changed footer hint from `[k] Kill` to `[x] Kill`
- **Files modified:** src/ui/process_monitor.rs
- **Verification:** cargo build passes, hint now matches actual key binding
- **Committed in:** bf41cf2

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug)
**Impact on plan:** Fix corrects misleading UI hint. No scope creep. Kill key deviation from plan ('x' vs 'k') was a correct implementation choice to preserve vim-style navigation consistency.

## Issues Encountered
- Both tasks were pre-implemented in the codebase before execution. Execution verified all acceptance criteria, ran build + tests (19 passed), and fixed the footer hint bug.

## Self-Check: PASSED
- `src/ui/process_monitor.rs` — FOUND
- `src/handlers/process_monitor.rs` — FOUND
- `bf41cf2` commit — FOUND

## Next Phase Readiness
- Process Monitor fully functional — Phase 3 (Vulnerability Audit) can build on ProcessInfo/SuspicionFlag foundation
- SEC-01, SEC-04, SEC-05 requirements fulfilled
- No known regressions — all 19 tests pass clean

---
*Phase: 02-security-process-monitor*
*Completed: 2026-03-27*
