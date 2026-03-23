# Phase 2: Security — Process Monitor - Research

**Researched:** 2026-03-23
**Domain:** sysinfo process API, Ratatui multi-panel TUI, Unix signal handling
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Suspicious Flagging Criteria (D-01 through D-05)**
- D-01: Path outside trusted locations (`/usr/`, `/bin/`, `/sbin/`, `/System/`, `/Applications/`, `/Library/`) is a red flag
- D-02: Process name != executable basename at path (classic malware name-spoofing)
- D-03: UID 0 (root) but path not in system path (`/System/`, `/usr/sbin/`, `/sbin/`)
- D-04: CPU sustained >80% (cryptominer indicator)
- D-05: Process name contains known-bad strings: `xmrig`, `cryptonight`, `miner`
- Severity: >= 1 rule = warning color; >= 2 rules = danger color

**Process List Layout (D-06 through D-09)**
- D-06: Left panel 65% scrollable table, Right panel 35% detail panel
- D-07: Table columns: Name, PID, CPU%, RAM only
- D-08: Suspicious entries highlighted with `theme.warning()` or `theme.danger()`
- D-09: Right panel shows: full executable path, owner/user, parent PID, uptime, and "Why suspicious:" section (only when flagged)

**Kill Flow UX (D-10 through D-12)**
- D-10: Multi-select like App Uninstaller — select many before kill
- D-11: Kill modal has 3 buttons: `[Cancel]` `[Graceful Kill]` `[Force Kill]`
  - Graceful Kill = SIGTERM, Force Kill = SIGKILL
- D-12: Kill failure (permission denied) shown in footer/status bar, no panic

**Dashboard Navigation (D-13 through D-15)**
- D-13: Hotkey `p` -> Process Monitor mode
- D-14: Process Monitor added as new menu item in dashboard (alongside File Manager, Uninstaller, Settings)
- D-15: Footer help bar updated to include `[p] Process Monitor`

### Claude's Discretion
- Threshold for CPU "sustained" vs spike (window duration, moving average or simple threshold)
- Default sort order for process list (by name, CPU%, or suspicious flag first)
- Exact color mapping for severity (1 rule = warning, 2+ = danger — or different threshold)

### Deferred Ideas (OUT OF SCOPE)
- Antivirus/YARA rules integration
- Network connections per process (lsof-style)
- Process tree view (parent-child hierarchy)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SEC-01 | User bisa membuka screen Process Monitor dari dashboard | D-13/D-14/D-15 decisions + dashboard.rs integration points documented |
| SEC-02 | App scan dan tampilkan semua running processes (nama, PID, CPU%, memory, path executable) | `sys.processes()` returns HashMap<Pid, Process>; all required fields available via sysinfo API |
| SEC-03 | App flag processes yang mencurigakan dengan kriteria yang jelas | D-01 through D-05 rules fully defined; sysinfo provides all needed fields (exe, name, user_id, cpu_usage) |
| SEC-04 | User bisa melihat detail process yang dipilih | Right panel pattern cloned from app_uninstaller.rs; all detail fields available from sysinfo Process struct |
| SEC-05 | User bisa kill process dengan konfirmasi modal | `process.kill_with(Signal::Term/Kill)` available; 3-button modal extends existing 2-button pattern |
</phase_requirements>

---

## Summary

Phase 2 adds a Process Monitor screen to Clario. The core data source — `sysinfo::System` — is already instantiated in `App.sys` and refreshed every 2 seconds via `sys.refresh_all()`. No new Cargo dependencies are needed.

The process scanner reads from `sys.processes()` (a `HashMap<Pid, Process>`) and applies 5 deterministic rules to compute a per-process `SuspicionScore`. The UI follows the exact left-panel-table + right-panel-detail split used by App Uninstaller, with the addition of a 3-button kill modal (Cancel / Graceful Kill / Force Kill) instead of the existing 2-button pattern.

The single novel implementation challenge is the 3-button modal navigation: `kill_confirm_selected: u8` cycles through 0=Cancel, 1=Graceful, 2=Force, with left/right wrapping. Kill operations call `process.kill_with(Signal::Term)` or `process.kill_with(Signal::Kill)` and handle the `Result` by writing to a status message field rather than panicking.

**Primary recommendation:** Model ProcessMonitor state fields in `App` directly (no separate state struct needed), follow the AppUninstaller pattern exactly, and implement the `ProcessScanner` as a pure function that takes `&HashMap<Pid, Process>` and returns `Vec<ProcessInfo>`.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sysinfo | 0.38.2 | Process enumeration, CPU/memory, kill signals | Already in Cargo.toml; provides all required fields |
| ratatui | 0.30.0 | TUI rendering, Table/Paragraph/Layout widgets | Already in Cargo.toml; entire UI built on it |
| crossterm | 0.29.0 | Keyboard event handling | Already in Cargo.toml; used by all handlers |

### No New Dependencies Required
All required functionality is available through existing crates. `sysinfo` provides:
- Process listing: `sys.processes()` -> `&HashMap<Pid, Process>`
- CPU usage: `process.cpu_usage()` -> `f32`
- Memory: `process.memory()` -> `u64` (bytes, RSS)
- Executable path: `process.exe()` -> `Option<&Path>`
- Process name: `process.name()` -> `&OsStr`
- User ID: `process.user_id()` -> `Option<&Uid>` (deref to u32)
- Parent PID: `process.parent()` -> `Option<Pid>`
- Uptime/run time: `process.run_time()` -> `u64` (seconds)
- Kill: `process.kill_with(Signal::Term)` and `process.kill_with(Signal::Kill)` -> `bool`

**Installation:** No new packages needed.

**Version verification:** `sysinfo = "0.38.2"` confirmed in Cargo.toml.

---

## Architecture Patterns

### New Files to Create
```
src/
├── models/
│   └── process_info.rs       # ProcessInfo struct + SuspicionFlags
├── core/
│   └── process_scanner.rs    # Pure scanning + flagging logic
├── ui/
│   └── process_monitor.rs    # draw_process_monitor(f, &mut App)
└── handlers/
    └── process_monitor.rs    # handle_key for ProcessMonitor mode
```

### Files to Modify
```
src/app.rs                    # AppMode::ProcessMonitor + state fields
src/ui/dashboard.rs           # Add menu item + footer hotkey
src/handlers/dashboard.rs     # Handle 'p' -> ProcessMonitor + Enter on new menu item
src/models/mod.rs             # pub mod process_info
src/core/mod.rs               # pub mod process_scanner
src/ui/mod.rs                 # pub mod process_monitor
src/handlers/mod.rs           # pub mod process_monitor
```

### Pattern 1: ProcessInfo Struct (modeled after AppInfo)
**What:** Domain model for a single process with pre-computed suspicion state
**When to use:** Built once per refresh cycle, passed to UI for render

```rust
// Source: modeled after src/models/app_info.rs
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: sysinfo::Pid,
    pub name: String,
    pub exe_path: Option<std::path::PathBuf>,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub user_id: Option<u32>,        // Deref from sysinfo::Uid
    pub parent_pid: Option<sysinfo::Pid>,
    pub run_time_secs: u64,
    pub suspicion_flags: Vec<SuspicionFlag>,  // Empty = clean
    pub is_selected: bool,
}

#[derive(Debug, Clone)]
pub enum SuspicionFlag {
    SuspiciousPath(String),           // D-01: path outside trusted dirs
    NameExeMismatch(String),          // D-02: name != exe basename
    RootOutsideSystemPath(String),    // D-03: UID 0, non-system path
    HighCpu(f32),                     // D-04: CPU > 80%
    KnownBadName(String),             // D-05: blacklisted string in name
}

impl ProcessInfo {
    pub fn severity(&self) -> SuspicionSeverity {
        match self.suspicion_flags.len() {
            0 => SuspicionSeverity::Clean,
            1 => SuspicionSeverity::Warning,
            _ => SuspicionSeverity::Danger,
        }
    }
}

pub enum SuspicionSeverity { Clean, Warning, Danger }
```

### Pattern 2: ProcessScanner (pure function)
**What:** Takes live sysinfo data, returns sorted Vec<ProcessInfo>
**When to use:** Called from App.run() loop when mode is ProcessMonitor, same trigger pattern as FileScanner

```rust
// Source: pattern from src/core/app_scanner.rs style
pub struct ProcessScanner;

impl ProcessScanner {
    /// Reads sys.processes() and applies all 5 suspicion rules.
    /// Returns list sorted by: suspicious first, then by name ascending.
    pub fn scan(sys: &sysinfo::System) -> Vec<ProcessInfo> {
        let trusted_paths = ["/usr/", "/bin/", "/sbin/", "/System/", "/Applications/", "/Library/"];
        let system_paths  = ["/System/", "/usr/sbin/", "/sbin/"];
        let bad_names     = ["xmrig", "cryptonight", "miner"];

        let mut results: Vec<ProcessInfo> = sys.processes()
            .values()
            .map(|p| {
                let exe = p.exe().map(|e| e.to_path_buf());
                let exe_str = exe.as_ref()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                let name = p.name().to_string_lossy().to_string();
                let uid: Option<u32> = p.user_id().map(|u| **u);
                let cpu = p.cpu_usage();

                let mut flags = Vec::new();

                // D-01: path outside trusted locations
                if !exe_str.is_empty() {
                    let in_trusted = trusted_paths.iter().any(|t| exe_str.starts_with(t));
                    if !in_trusted {
                        flags.push(SuspicionFlag::SuspiciousPath(exe_str.clone()));
                    }
                }

                // D-02: name != exe basename
                if !exe_str.is_empty() {
                    if let Some(basename) = std::path::Path::new(&exe_str).file_name() {
                        if basename.to_string_lossy() != name {
                            flags.push(SuspicionFlag::NameExeMismatch(
                                format!("name='{}' exe='{}'", name, basename.to_string_lossy())
                            ));
                        }
                    }
                }

                // D-03: UID 0 but path not in system path
                if uid == Some(0) && !exe_str.is_empty() {
                    let in_system = system_paths.iter().any(|t| exe_str.starts_with(t));
                    if !in_system {
                        flags.push(SuspicionFlag::RootOutsideSystemPath(exe_str.clone()));
                    }
                }

                // D-04: CPU > 80%
                if cpu > 80.0 {
                    flags.push(SuspicionFlag::HighCpu(cpu));
                }

                // D-05: known-bad name strings
                let name_lower = name.to_lowercase();
                for bad in &bad_names {
                    if name_lower.contains(bad) {
                        flags.push(SuspicionFlag::KnownBadName((*bad).to_string()));
                        break;
                    }
                }

                ProcessInfo {
                    pid: p.pid(),
                    name,
                    exe_path: exe,
                    cpu_usage: cpu,
                    memory_bytes: p.memory(),
                    user_id: uid,
                    parent_pid: p.parent(),
                    run_time_secs: p.run_time(),
                    suspicion_flags: flags,
                    is_selected: false,
                }
            })
            .collect();

        // Sort: suspicious first, then alphabetical
        results.sort_by(|a, b| {
            b.suspicion_flags.len().cmp(&a.suspicion_flags.len())
                .then(a.name.cmp(&b.name))
        });

        results
    }
}
```

### Pattern 3: App State Fields for ProcessMonitor
**What:** New fields to add to `App` struct in app.rs
**When to use:** Added alongside existing AppUninstaller fields

```rust
// In App struct — add after apps/app_table_state fields:
pub processes: Vec<crate::models::process_info::ProcessInfo>,
pub process_table_state: ratatui::widgets::TableState,
pub selected_process_index: usize,
pub show_kill_confirm: bool,
pub kill_confirm_selected: u8,  // 0=Cancel, 1=Graceful(SIGTERM), 2=Force(SIGKILL)
pub kill_status_message: Option<String>, // Error message from failed kill
```

### Pattern 4: 3-Button Kill Modal Navigation
**What:** Extends the existing 2-button modal pattern to 3 buttons
**When to use:** Kill confirm modal in ProcessMonitor

```rust
// In handlers/process_monitor.rs — kill modal key handling
if app.show_kill_confirm {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            app.kill_confirm_selected = if app.kill_confirm_selected == 0 { 2 }
                                        else { app.kill_confirm_selected - 1 };
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.kill_confirm_selected = (app.kill_confirm_selected + 1) % 3;
        }
        KeyCode::Esc | KeyCode::Char('n') => {
            app.show_kill_confirm = false;
            app.kill_confirm_selected = 0; // Reset to Cancel (safe default)
        }
        KeyCode::Enter => {
            match app.kill_confirm_selected {
                0 => { app.show_kill_confirm = false; } // Cancel
                1 => { /* execute SIGTERM */ }
                2 => { /* execute SIGKILL */ }
                _ => {}
            }
        }
        _ => {}
    }
    return;
}
```

### Pattern 5: Kill Execution (no background thread needed)
**What:** Synchronous kill via sysinfo; no channel/spawn_blocking needed
**When to use:** Kill is instant syscall, result returned immediately

```rust
// Execute kill — synchronous, result communicated via status message
fn execute_kill(app: &mut App, signal: sysinfo::Signal) {
    let pids: Vec<sysinfo::Pid> = app.processes.iter()
        .filter(|p| p.is_selected)
        .map(|p| p.pid)
        .collect();

    let mut errors = Vec::new();
    for pid in pids {
        if let Some(process) = app.sys.process(pid) {
            if !process.kill_with(signal) {
                errors.push(format!("PID {}: permission denied", pid));
            }
        }
    }

    if errors.is_empty() {
        app.kill_status_message = None;
    } else {
        app.kill_status_message = Some(format!("Kill failed: {}", errors.join(", ")));
    }

    // Refresh process list after kill attempt
    app.processes = crate::core::process_scanner::ProcessScanner::scan(&app.sys);
    // Deselect all and reset index
    app.selected_process_index = 0;
    app.process_table_state.select(Some(0));
    app.show_kill_confirm = false;
}
```

### Pattern 6: ProcessMonitor Refresh Trigger
**What:** When to re-scan processes (leverage existing sys.refresh_all() at 2s interval)
**When to use:** In the main run() loop

```rust
// In app.rs run() loop — add after existing auto-trigger block:
// Process list stays fresh because sys.refresh_all() runs every 2 seconds.
// Re-compute ProcessInfo vec when mode == ProcessMonitor AND processes is stale.
// Simplest approach: re-scan on every sys refresh when in ProcessMonitor mode.
if self.mode == AppMode::ProcessMonitor
    && self.last_sys_refresh.elapsed() < std::time::Duration::from_millis(50)
{
    // Re-scan after each sys refresh
    self.processes = crate::core::process_scanner::ProcessScanner::scan(&self.sys);
    // Preserve selection index (clamp to new len)
    if self.selected_process_index >= self.processes.len() {
        self.selected_process_index = self.processes.len().saturating_sub(1);
    }
    self.process_table_state.select(Some(self.selected_process_index));
}
```

**Alternative (Claude's Discretion):** Re-scan only on mode entry + manual refresh key `r`. Simpler to reason about, avoids list jumping under cursor during navigation. Recommended for v1.

### Pattern 7: Dashboard Integration Points
**What:** Exact locations to modify for `p` hotkey and menu item

```rust
// 1. app.rs AppMode enum — add variant:
pub enum AppMode {
    Dashboard,
    FileManager,
    Settings,
    AppUninstaller,
    ProcessMonitor,  // NEW
}

// 2. app.rs run() — global hotkey block:
KeyCode::Char('p') => {
    self.mode = AppMode::ProcessMonitor;
    // Trigger initial process scan on entry
    if self.processes.is_empty() {
        self.processes = crate::core::process_scanner::ProcessScanner::scan(&self.sys);
        if !self.processes.is_empty() {
            self.process_table_state.select(Some(0));
        }
    }
    continue;
}

// 3. app.rs terminal.draw() match block:
AppMode::ProcessMonitor => draw_process_monitor(f, self),

// 4. app.rs event dispatch match block:
AppMode::ProcessMonitor => handlers::process_monitor::handle_key(self, key),

// 5. dashboard.rs — menu item count changes from 3 to 4
// selected_menu cycles 0..=3 (update % 3 -> % 4 in dashboard handler)
// Add m4_style block and "🔍 [p] Process Monitor" line in action_menu_text

// 6. dashboard.rs footer:
" [p] ".fg(theme.danger()).bold(),
"Processes  ".fg(theme.muted_text()),

// 7. handlers/dashboard.rs — update modulo and add new Enter arm:
KeyCode::Down => app.selected_menu = (app.selected_menu + 1) % 4,  // was % 3
KeyCode::Up   => app.selected_menu = (app.selected_menu + 3) % 4,  // was % 2 trick
// Enter arm 3 => app.mode = AppMode::ProcessMonitor
```

### Anti-Patterns to Avoid
- **Spawning a background thread for process scanning:** `sys.processes()` is synchronous and already populated by `sys.refresh_all()`. No spawn_blocking needed for the scan itself. Only kill might warrant background execution if multiple targets, but in practice it's instant.
- **Re-creating sysinfo::System for process monitor:** `App.sys` is the single shared instance. Call `sys.process(pid)` directly for kill, not a new System.
- **Calling `sys.refresh_all()` inside ProcessScanner::scan():** Scan is a pure read; refresh happens in the main loop only.
- **Storing `&Process` references in ProcessInfo:** Process struct cannot be stored long-term — copy all needed values into owned `ProcessInfo` fields during scan.
- **Using `unwrap()` on `process.exe()`:** Many system processes return `None` for exe on macOS due to sandboxing/permissions. Always handle `Option`.
- **CPU spike vs sustained (D-04 precision):** `cpu_usage()` in sysinfo already returns an averaged value per refresh interval, not instantaneous. At 2-second refresh, this is inherently smoothed. No additional moving average needed for v1.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process enumeration | Custom `/proc` or `ps` subprocess | `sys.processes()` | Cross-platform, already warmed up in App.sys |
| Signal sending | libc::kill() FFI | `process.kill_with(Signal::Term/Kill)` | Safe Rust, handles cross-platform signal mapping |
| Memory formatting | Custom byte formatter | Copy pattern from dashboard.rs `bytes` formatter | Already battle-tested in this codebase |
| Modal centering | Custom Rect math | `components::centered_rect(w, h, size)` | Already extracted in Phase 1 |

**Key insight:** The entire data layer (process enumeration, signal sending) is already abstracted by sysinfo. This phase is almost entirely UI work layered on top of an already-warmed System instance.

---

## Common Pitfalls

### Pitfall 1: exe() Returns None on macOS for System Processes
**What goes wrong:** Calling `process.exe().unwrap()` panics on processes like kernel_task, launchd, etc.
**Why it happens:** macOS SIP prevents reading exe paths for certain privileged processes.
**How to avoid:** Always `process.exe().map(|p| p.to_path_buf())` — store as `Option<PathBuf>`. In D-01/D-02/D-03 logic, skip rule evaluation if exe is None (don't flag as suspicious just because exe is unreadable).
**Warning signs:** Any test that uses a real system will hit this immediately.

### Pitfall 2: cpu_usage() Always Returns 0.0 on First Refresh
**What goes wrong:** All processes show 0% CPU on initial process list load.
**Why it happens:** sysinfo needs two refresh cycles to compute a CPU delta. First call has no baseline.
**How to avoid:** This is expected behavior. Show 0.0 on first render. On second and subsequent 2-second refreshes, values will be accurate. No workaround needed — just document it.
**Warning signs:** User reports "all processes show 0% CPU" only for the first 2 seconds.

### Pitfall 3: Dashboard Menu Index Overflow
**What goes wrong:** Adding a 4th menu item but leaving `% 3` modulo in `handlers/dashboard.rs` causes the new item to be unreachable.
**Why it happens:** `selected_menu = (selected_menu + 1) % 3` wraps at index 3 back to 0.
**How to avoid:** Update modulo to `% 4` in dashboard handler. Also update the Up arrow calculation (`% 2` trick becomes `% 3` trick: `(selected_menu + 3) % 4`).
**Warning signs:** Pressing Down from Settings menu item jumps directly to first item, never reaches Process Monitor.

### Pitfall 4: Kill Confirmation State Not Reset on Mode Exit
**What goes wrong:** User opens kill modal, presses Esc to go to Dashboard (global hotkey), returns to ProcessMonitor, modal is still showing.
**Why it happens:** Global `Esc` key handling in app.rs switches mode without resetting ProcessMonitor-specific state.
**How to avoid:** In the global `Esc`/`d` mode-switching handlers, add: `app.show_kill_confirm = false; app.kill_status_message = None;`

### Pitfall 5: Stale PIDs After Kill
**What goes wrong:** Killing a process then immediately accessing `sys.process(old_pid)` returns Some() until the next `sys.refresh_all()` cycle.
**Why it happens:** sysinfo caches process state until refresh.
**How to avoid:** After kill, call `app.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true)` immediately, then re-scan `ProcessScanner::scan(&app.sys)`. This is safe to call outside the 2-second throttle window — it's a targeted post-kill refresh.

### Pitfall 6: D-01 Over-Flagging Helper Processes
**What goes wrong:** Many legitimate Apple helper processes run from paths like `/private/var/folders/...` (which is a tmp-like path) or from inside `.app` bundles in `/Applications/` subdirs.
**Why it happens:** D-01 check is path prefix only. `/Applications/MyApp.app/Contents/MacOS/helper` starts with `/Applications/` so is trusted. But `/private/var/folders/abc/T/com.apple.helper` would be flagged — this is actually correct behavior (Apple tmp helpers are transient, legitimately suspicious-looking).
**How to avoid:** The trusted list is intentionally conservative. Accepting some false positives is the design intent — the "Why suspicious" detail panel explains the reasoning. No change needed; document this as expected behavior.

---

## Code Examples

### Iterating processes and checking sysinfo fields
```rust
// Source: https://docs.rs/sysinfo/0.38.2/sysinfo/struct.System.html
use sysinfo::{System, Signal};

let sys = System::new_all();

for (pid, process) in sys.processes() {
    let name = process.name().to_string_lossy();
    let cpu  = process.cpu_usage();    // f32, 0.0-100.0 per core
    let mem  = process.memory();       // u64, bytes (RSS)
    let exe  = process.exe();          // Option<&Path>
    let uid  = process.user_id().map(|u| **u); // Option<u32>
    let ppid = process.parent();       // Option<Pid>
    let rt   = process.run_time();     // u64 seconds
}
```

### Sending signals via sysinfo
```rust
// Source: https://docs.rs/sysinfo/0.38.2/sysinfo/struct.Process.html
use sysinfo::Signal;

if let Some(process) = sys.process(target_pid) {
    let success = process.kill_with(Signal::Term); // SIGTERM
    if !success {
        // Permission denied or process already dead
        eprintln!("Kill failed for PID {:?}", target_pid);
    }
}

// Force kill
if let Some(process) = sys.process(target_pid) {
    let _ = process.kill_with(Signal::Kill); // SIGKILL — cannot be ignored
}
```

### 3-button horizontal modal layout (extending existing 2-button pattern)
```rust
// Source: adapted from src/ui/app_uninstaller.rs confirm modal pattern
// 3 buttons: Cancel | Graceful Kill | Force Kill
let btn_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Length(1),      // left margin
        Constraint::Percentage(33), // Cancel
        Constraint::Percentage(33), // Graceful Kill
        Constraint::Percentage(33), // Force Kill
        Constraint::Length(1),      // right margin
    ])
    .split(confirm_chunks[1]);

// kill_confirm_selected: 0=Cancel, 1=Graceful, 2=Force
let style_for = |idx: u8, color: ratatui::style::Color| {
    if app.kill_confirm_selected == idx {
        Style::default().fg(theme.bg()).bg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted_text())
    }
};
```

### Formatting run_time for display
```rust
// Convert sysinfo run_time (seconds) to human-readable string
fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}
```

### Formatting memory for table display
```rust
// Consistent with existing bytes formatter in dashboard.rs
fn format_memory(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}G", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.0}M", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.0}K", bytes as f64 / 1024.0)
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `System::new_all()` per feature screen | Single shared `App.sys` refreshed every 2s | Phase 1 (established) | No new System allocation needed for ProcessMonitor |
| `centered_rect` duplicated per file | `components::centered_rect` shared helper | Phase 1 (REFAC-02) | Kill modal uses this directly |
| Progress via shared `scan_progress_text` | Per-operation progress fields | Phase 1 (REFAC-05) | ProcessMonitor does not need a new progress field — scan is synchronous |

---

## Claude's Discretion Recommendations

### CPU "Sustained" Threshold (D-04)
**Recommendation:** Use simple per-refresh threshold at 80% — no moving average.
**Rationale:** `sysinfo.cpu_usage()` is already averaged over the refresh interval (2 seconds). A process sustaining >80% for one 2-second window is a meaningful signal. Adding a moving average adds state complexity for minimal benefit. The user can see CPU% in the table and make their own judgment.

### Default Sort Order
**Recommendation:** Suspicious first (by flag count descending), then alphabetical by name.
**Rationale:** The primary use case is "show me what's suspicious." Clean processes can be browsed alphabetically. This is already shown in the ProcessScanner::scan() example above.

### Severity Color Mapping
**Recommendation:** 1 rule = `theme.warning()`, 2+ rules = `theme.danger()`.
**Rationale:** Matches existing warning/danger semantic split in the codebase. Clean processes use `theme.text()` (default). This is the simplest mapping consistent with D-08.

---

## Validation Architecture

> `workflow.nyquist_validation` is not set in config.json — treating as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | tokio-test (dev-dep in Cargo.toml) + Rust built-in `#[test]` |
| Config file | None — uses `cargo test` |
| Quick run command | `cargo test process` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SEC-01 | ProcessMonitor mode accessible via 'p' hotkey | manual | Manual TUI test | N/A |
| SEC-02 | All required process fields populated | unit | `cargo test process_scanner` | Wave 0 |
| SEC-03 | Suspicious flagging rules D-01 through D-05 correct | unit | `cargo test suspicion_flags` | Wave 0 |
| SEC-04 | Detail panel shows correct process fields | manual | Manual TUI test | N/A |
| SEC-05 | Kill modal appears, signal sent, error handled | unit (kill path only) | `cargo test kill_result_handling` | Wave 0 |

### Wave 0 Gaps
- [ ] `src/core/process_scanner.rs` — unit tests for each of the 5 suspicion rules
- [ ] `src/models/process_info.rs` — `severity()` method unit test (0 flags = Clean, 1 = Warning, 2+ = Danger)

*(UI and navigation tests are manual-only — no headless TUI test infrastructure exists in this project)*

---

## Open Questions

1. **ProcessMonitor list refresh during active navigation**
   - What we know: `sys.refresh_all()` runs every 2 seconds; re-scanning replaces the `Vec<ProcessInfo>` which could move the cursor
   - What's unclear: Should the list auto-refresh (list jumps) or stay static until manual `r` key?
   - Recommendation: Static list with manual refresh key `r` for v1. Auto-refresh is disorienting when user is reading the detail panel. Mark as Claude's Discretion.

2. **UID 0 detection on macOS with SIP**
   - What we know: `process.user_id()` returns `Option<&Uid>`; some SIP-protected processes may return None
   - What's unclear: Whether Apple processes that should show UID 0 will actually return Some(0) or None
   - Recommendation: D-03 rule only triggers when `uid == Some(0)` AND exe is Some — safe default.

---

## Sources

### Primary (HIGH confidence)
- [https://docs.rs/sysinfo/0.38.2/sysinfo/struct.Process.html] — All Process methods verified: cpu_usage, memory, exe, name, user_id, parent, run_time, kill_with
- [https://docs.rs/sysinfo/0.38.2/sysinfo/struct.System.html] — System::processes(), process(), refresh_processes() verified
- [https://docs.rs/sysinfo/0.38.2/sysinfo/enum.Signal.html] — Signal::Term and Signal::Kill variants confirmed
- [https://docs.rs/sysinfo/0.38.2/sysinfo/struct.ProcessRefreshKind.html] — ProcessRefreshKind builder pattern confirmed
- src/app.rs — App struct fields, AppMode enum, run() loop pattern (direct codebase read)
- src/ui/app_uninstaller.rs — Left/right panel layout, confirm modal pattern, 2-button layout (direct codebase read)
- src/handlers/app_uninstaller.rs — Multi-select + modal key handling pattern (direct codebase read)
- src/ui/dashboard.rs — Menu item structure, footer format, hotkey list (direct codebase read)
- src/ui/components.rs — centered_rect signature, draw_exit_modal 2-button pattern (direct codebase read)

### Secondary (MEDIUM confidence)
- sysinfo CPU averaging behavior — documented in ProcessRefreshKind docs; 2-sample delta is standard for cpu_usage()

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — sysinfo 0.38.2 already in Cargo.toml; all required APIs verified against docs.rs
- Architecture: HIGH — direct codebase analysis of all 4 reference files; patterns are concrete and copy-exact
- Pitfalls: HIGH for exe()/uid pitfalls (official docs), MEDIUM for D-01 over-flagging (reasoning from macOS path conventions)

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (sysinfo API is stable; ratatui 0.30 is stable)
