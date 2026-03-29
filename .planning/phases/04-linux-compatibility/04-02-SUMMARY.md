---
phase: 04-linux-compatibility
plan: 02
subsystem: platform-compatibility
tags: [rust, cfg, cross-platform, linux, macos, testing, process-scanner]

# Dependency graph
requires:
  - phase: 04-linux-compatibility/04-01
    provides: Platform-specific TRUSTED_PATHS/ROOT_SYSTEM_PATHS in process_scanner, app_scanner cfg-gated to macOS

provides:
  - Linux-specific trusted path tests for process scanner (opt, snap, systemd)
  - Cross-platform /tmp suspicious path test that runs on all platforms
  - cfg-gated app_scanner spawn in dashboard.rs handler (was missing the guard)
  - Verified CLI clean pathway is fully cross-platform (trash::delete, docker Command)

affects: [05-cli-compat, any-phase-adding-process-scanner-tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "#[cfg(target_os = \"linux\")] on test functions for platform-specific test coverage"
    - "Cross-platform tests (no cfg) run on all OSes to verify universal invariants"

key-files:
  created: []
  modified:
    - src/core/process_scanner.rs
    - src/handlers/dashboard.rs

key-decisions:
  - "Linux-specific tests gated with #[cfg(target_os = \"linux\")] — they test paths that only exist/matter on Linux"
  - "Cross-platform /tmp test has no cfg guard — invariant that holds on all platforms deserves universal coverage"
  - "dashboard.rs app_scanner spawn fixed: missing #[cfg(target_os = \"macos\")] guard added — would have failed to compile on Linux"

patterns-established:
  - "Platform test pattern: wrap Linux-only tests with #[cfg(target_os = \"linux\")], add corresponding cross-platform variant with no cfg for universal invariants"

requirements-completed: [LINUX-05, LINUX-06]

# Metrics
duration: 6min
completed: 2026-03-29
---

# Phase 04 Plan 02: Linux Compatibility Validation Summary

**Linux process scanner tests added (opt/snap/systemd trusted paths), cross-platform /tmp invariant test, and missing macOS cfg-gate fixed in dashboard handler.**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-03-29T10:23:34Z
- **Completed:** 2026-03-29T10:29:54Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `test_tmp_suspicious_all_platforms` — cross-platform test proving /tmp is untrusted everywhere (runs on macOS too)
- Added 3 Linux-specific tests: `test_linux_opt_trusted`, `test_linux_snap_trusted`, `test_linux_systemd_root_not_flagged` — all under `#[cfg(target_os = "linux")]`
- Verified `src/cli/clean.rs` uses only cross-platform APIs: `trash::delete` (freedesktop spec on Linux) and `std::process::Command::new("docker")` — no macOS-only code
- Fixed missing `#[cfg(target_os = "macos")]` guard on app_scanner spawn in `dashboard.rs` menu handler (would have been a compile error on Linux)
- `cargo check` and `cargo test` both clean: 0 warnings, 20 tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Linux-specific process scanner tests and verify CLI clean pathway** - `eeaae75` (test)
2. **Task 2: Final compilation audit and dead-code cleanup** - `3c33548` (fix)

**Plan metadata:** (docs: complete plan — see below)

## Files Created/Modified

- `src/core/process_scanner.rs` - 4 new tests added: 1 cross-platform + 3 Linux-specific under `#[cfg(target_os = "linux")]`
- `src/handlers/dashboard.rs` - Added `#[cfg(target_os = "macos")]` guard to app_scanner spawn block in menu item 1

## Decisions Made

- Linux-specific tests wrapped with `#[cfg(target_os = "linux")]` — these test path constants (`/opt/`, `/snap/`, `/usr/lib/systemd/`) that only exist on Linux and would never be valid on macOS
- Cross-platform `/tmp` test has no cfg guard — an invariant that must hold on all platforms deserves to run on all platforms
- `dashboard.rs` app_scanner spawn fix classified as Rule 1 (bug): the missing cfg guard would cause a compile error on Linux, matching the same pattern already correctly applied in `app.rs` hotkey handler

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing #[cfg(target_os = "macos")] in dashboard.rs app_scanner spawn**
- **Found during:** Task 2 (compilation audit)
- **Issue:** `src/handlers/dashboard.rs` menu item 1 called `crate::core::app_scanner::AppScanner::scan_applications` without any cfg guard. Since `app_scanner` is gated to macOS in `mod.rs`, this would fail to compile on Linux.
- **Fix:** Added `#[cfg(target_os = "macos")]` attribute on the scan spawn block (lines 41-54), mirroring the identical pattern in `app.rs` hotkey handler
- **Files modified:** `src/handlers/dashboard.rs`
- **Verification:** `cargo check` passes with 0 warnings; `cargo test` passes with 20 tests
- **Committed in:** `3c33548` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug)
**Impact on plan:** Essential correctness fix — without it, Linux builds would fail at the dashboard handler. No scope creep.

## Issues Encountered

None. Both `cargo check` and `cargo test` passed cleanly on first attempt after each change.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 04 complete: Linux compile-blocking issues resolved (Plan 01), tests added and handler bug fixed (Plan 02)
- On macOS: all 20 tests pass, zero warnings, full feature parity
- On Linux: process scanner trusted paths correct, app_scanner spawn properly gated, graceful degradation in App Uninstaller UI
- Ready for Phase 05 (CLI Linux compatibility) or any future cross-platform work

---
*Phase: 04-linux-compatibility*
*Completed: 2026-03-29*
