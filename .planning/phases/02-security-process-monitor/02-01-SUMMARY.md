---
phase: 02-security-process-monitor
plan: 01
subsystem: security
tags: [sysinfo, process-monitor, rust, security-scanning]

# Dependency graph
requires: []
provides:
  - ProcessInfo struct with all 9 required fields (pid, name, exe_path, cpu_usage, memory_bytes, user_id, parent_pid, run_time_secs, suspicion_flags)
  - SuspicionFlag enum with 5 variants (D-01 through D-05) with associated data
  - SuspicionSeverity enum (Clean/Warning/Danger) with severity() method
  - ProcessScanner::scan() reading sysinfo::System and returning Vec<ProcessInfo> sorted suspicious-first
  - apply_rules() standalone helper for testable rule logic without real System instance
  - format_memory() and format_uptime() display helpers
  - 17 unit tests covering all 5 rules with positive/negative/exe-None cases
affects: [02-02-security-process-monitor-tui]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Stateless scanner struct (ProcessScanner) — mirrors AppScanner pattern from Phase 1"
    - "Standalone rule-checking function (apply_rules) decoupled from sysinfo::System for testability"
    - "TDD with behavior tests written inline alongside implementation for data models"

key-files:
  created:
    - src/models/process_info.rs
    - src/core/process_scanner.rs
  modified:
    - src/models/mod.rs
    - src/core/mod.rs

key-decisions:
  - "apply_rules() extracted as standalone function rather than private method — enables unit testing without needing a real sysinfo::System instance"
  - "exe=None explicitly skips D-01, D-02, D-03 — prevents false flags on SIP-protected macOS system processes"
  - "D-04 (HighCpu) threshold set at > 80.0% — no exe required since CPU is independent of path"
  - "format_memory and format_uptime co-located in process_scanner.rs — tested alongside scanner, consumed by TUI in Plan 02"
  - "Pid::from(1usize) not u32 — sysinfo 0.38.2 Pid only implements From<usize>"

patterns-established:
  - "Rule logic extracted to standalone fn apply_rules(name, exe, uid, cpu) -> Vec<SuspicionFlag> for testability without mock sysinfo"
  - "Sorting: b.suspicion_flags.len().cmp(&a.suspicion_flags.len()).then(a.name.cmp(&b.name)) — suspicious-first, then alphabetical"

requirements-completed: [SEC-02, SEC-03]

# Metrics
duration: 8min
completed: 2026-03-25
---

# Phase 02 Plan 01: ProcessInfo Data Model and ProcessScanner Summary

**ProcessInfo and SuspicionFlag data layer with ProcessScanner applying 5 detection rules (D-01 through D-05) via sysinfo, 17 unit tests all green**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-25T07:47:07Z
- **Completed:** 2026-03-25T07:55:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- ProcessInfo struct with 10 fields and SuspicionFlag/SuspicionSeverity enums powering severity() and display_reason() methods
- ProcessScanner::scan() iterates sysinfo::System processes, applies 5 detection rules, and returns sorted Vec<ProcessInfo>
- 17 unit tests covering all 5 rules with positive/negative cases plus exe=None safety and format helpers

## Task Commits

Each task was committed atomically:

1. **Task 1: ProcessInfo data model and SuspicionFlag enum** - `9e3c5d9` (feat)
2. **Task 2: ProcessScanner with 5 suspicion rules and unit tests** - `274a219` (feat)

**Plan metadata:** _(this commit)_ (docs: complete plan)

## Files Created/Modified

- `src/models/process_info.rs` - ProcessInfo struct, SuspicionFlag enum (5 variants D-01-D-05), SuspicionSeverity enum with severity() and display_reason() methods + 4 unit tests
- `src/core/process_scanner.rs` - ProcessScanner::scan(), apply_rules() standalone helper, format_memory(), format_uptime() + 13 unit tests
- `src/models/mod.rs` - Added `pub mod process_info`
- `src/core/mod.rs` - Added `pub mod process_scanner`

## Decisions Made

- apply_rules() extracted as standalone function rather than private method — enables unit testing without needing a real sysinfo::System instance
- exe=None explicitly skips D-01, D-02, D-03 — prevents false flags on SIP-protected macOS system processes
- D-04 (HighCpu) threshold set at > 80.0% — no exe required since CPU is independent of path
- format_memory and format_uptime co-located in process_scanner.rs — tested alongside scanner, consumed by TUI in Plan 02
- Pid::from(1usize) not u32 — sysinfo 0.38.2 Pid only implements From<usize> (minor deviation from initial test code)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Pid::from type argument**
- **Found during:** Task 1 (ProcessInfo data model)
- **Issue:** Test helper used `Pid::from(1u32)` but sysinfo 0.38.2 only implements `From<usize>` for Pid, causing compile error
- **Fix:** Changed `Pid::from(1u32)` to `Pid::from(1usize)`
- **Files modified:** src/models/process_info.rs
- **Verification:** `cargo test process_info` passes 4/4
- **Committed in:** 9e3c5d9 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor type fix, no scope change. Plan executed as specified.

## Issues Encountered

- Binary crate (main.rs only) requires `cargo test process_info` instead of `cargo test --lib process_info` — adjusted test commands accordingly

## Known Stubs

None — all data is real sysinfo data, no hardcoded values or placeholder returns in the public API.

## Next Phase Readiness

- All data layer types ready for Plan 02-02 TUI consumption
- ProcessScanner::scan(sys) is the single integration point — TUI passes App's existing sysinfo::System
- format_memory/format_uptime helpers ready for process list rendering
- Dead code warnings expected until TUI wires up these types in the next plan

---
*Phase: 02-security-process-monitor*
*Completed: 2026-03-25*

## Self-Check: PASSED

- src/models/process_info.rs — FOUND
- src/core/process_scanner.rs — FOUND
- 02-01-SUMMARY.md — FOUND
- Commit 9e3c5d9 (Task 1) — FOUND
- Commit 274a219 (Task 2) — FOUND
