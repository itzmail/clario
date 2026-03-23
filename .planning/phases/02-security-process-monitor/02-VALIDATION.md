---
phase: 2
slug: security-process-monitor
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + tokio-test (dev-dep) |
| **Config file** | none — uses `cargo test` |
| **Quick run command** | `cargo test process` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test process`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-xx | 01 | 1 | SEC-02 | unit | `cargo test process_scanner` | ❌ W0 | ⬜ pending |
| 02-01-xx | 01 | 1 | SEC-03 | unit | `cargo test suspicion_flags` | ❌ W0 | ⬜ pending |
| 02-01-xx | 01 | 1 | SEC-03 | unit | `cargo test severity` | ❌ W0 | ⬜ pending |
| 02-02-xx | 02 | 2 | SEC-01 | manual | Manual TUI test | N/A | ⬜ pending |
| 02-02-xx | 02 | 2 | SEC-04 | manual | Manual TUI test | N/A | ⬜ pending |
| 02-02-xx | 02 | 2 | SEC-05 | unit | `cargo test kill_result_handling` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/core/process_scanner.rs` — unit tests for each of the 5 suspicion rules (D-01 through D-05)
- [ ] `src/models/process_info.rs` — `severity()` method unit test (0 flags = Clean, 1 = Warning, 2+ = Danger)
- [ ] `src/core/process_scanner.rs` — `kill_result_handling` test stub for SEC-05 kill error path

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Process Monitor accessible via `p` hotkey | SEC-01 | No headless TUI test infra | Run app, press `p`, verify Process Monitor screen loads |
| Detail panel shows selected process fields | SEC-04 | No headless TUI test infra | Select a process, verify right panel shows exe path, owner, parent PID, uptime |
| Kill modal navigation (3 buttons) | SEC-05 | No headless TUI test infra | Select process, press `k`, navigate Cancel/Graceful/Force with arrow keys |
| Permission denied shown in footer (no panic) | SEC-05 | Requires privileged process | Attempt to kill a system process, verify error in status bar, no crash |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
