# Phase 1: TUI Refactor & Architecture Cleanup - Research

**Researched:** 2026-03-23
**Domain:** Rust / ratatui 0.30 / TUI architecture patterns / serde_json persistence
**Confidence:** HIGH (all findings grounded in direct source-code inspection plus verified ratatui idioms)

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REFAC-01 | Draw function signatures unified — all screens use `&mut App` OR all use explicit params (choose one, consistent) | Signature audit completed; recommendation: migrate to `&App` (immutable); see Architecture Patterns §1 |
| REFAC-02 | `centered_rect` extracted to `src/ui/components.rs` as shared helper, duplicates removed | Three copies identified across file_manager.rs, app_uninstaller.rs, components.rs; consolidation path documented |
| REFAC-03 | FileManager scan kickoff moved out of `terminal.draw(...)` closure to handler or lifecycle method | Exact offending lines identified (app.rs 199–213); correct placement documented |
| REFAC-04 | `ScanEvent` moved from `src/models/file_info.rs` to `src/core/events.rs` (or `src/app.rs`) | Cross-module dependency chain mapped; new file path and required import updates listed |
| REFAC-05 | Shared `scan_progress_text` replaced with per-operation progress state | Three consumers identified; split strategy documented with zero logic change to background threads |
| REFAC-06 | `AppInfo::is_expanded` and `App::related_files_state` removed or implemented | Both confirmed unused; safe removal path with no behavior change |
| REFAC-07 | Dashboard stats displayed from real data — persist to config/state file | Persistence pattern already present in project (AppConfig + serde_json); additive state struct approach documented |
</phase_requirements>

---

## Summary

Phase 1 is a pure internal refactor — no new user-visible features, no new dependencies required. Every issue is fully understood from direct source-code inspection: the codebase has 332 lines of `app.rs`, four UI draw functions, and one existing persistence layer (`AppConfig` via `serde_json` to `~/.config/clario/config.json`). All seven REFAC requirements have clear, low-risk implementation paths.

The two biggest decisions are (1) which draw function signature convention to standardize on, and (2) where to persist dashboard stats. For signatures, `&App` (immutable reference) is the right call — it keeps render functions testable without requiring the full `&mut App` borrow, and the two screens that already use explicit params can be simplified to pass `&app` instead of twelve separate fields. For persistence, a new `CleanStats` struct added to `AppConfig` (or a parallel `state.json` file) with serde_json is the correct approach — no new dependencies, consistent with the pattern that already exists.

The file-manager scan trigger inside `terminal.draw(...)` (app.rs lines 199–213) is the only structural risk in this phase. The fix is straightforward: move the `if self.scanned_files.is_empty() && !self.is_scanning` block to the top of the main event loop (before the draw call), or into a `lifecycle_tick()` helper method on `App`. The behavior is identical; only placement changes.

**Primary recommendation:** Standardize all four draw functions on `fn draw_X(f: &mut Frame, app: &App)` — immutable borrow, no explicit param explosion, no `&mut App` needed since stateful widgets only require `&mut TableState` which can be obtained via interior mutability or by passing `&mut self.table_state` separately.

---

## Standard Stack

### Core (already in Cargo.toml — no new dependencies needed for this phase)

| Library | Version (pinned) | Purpose | Why Standard |
|---------|-----------------|---------|--------------|
| ratatui | 0.30.0 | TUI widget framework | Already in project; no upgrade needed |
| serde + serde_json | 1.0 + 1.0.149 | State persistence | Already used for AppConfig; extend for CleanStats |
| chrono | 0.4.43 | DateTime for last_clean_date | Already in project, already has serde feature |

### No New Dependencies

All REFAC items are achievable with the existing dependency set. Adding a new crate for this phase would be an anti-pattern — the project already has serde_json and serde with derive, which is all that is needed for stats persistence.

**Version verification:** All versions confirmed against Cargo.toml directly (no registry lookup needed — this is a pinned existing project, not a greenfield stack selection).

---

## Architecture Patterns

### Recommended Project Structure (after Phase 1)

```
src/
├── app.rs                    # App struct + run loop (scan trigger MOVED here from draw)
├── core/
│   ├── events.rs             # NEW: ScanEvent moved here from models/file_info.rs
│   ├── app_scanner.rs
│   ├── file_scanner.rs
│   ├── file_ops.rs
│   └── mod.rs                # pub mod events;
├── models/
│   ├── file_info.rs          # ScanEvent REMOVED; FileInfo, SafetyLevel, FileCategory remain
│   ├── app_info.rs           # is_expanded field REMOVED (or kept + implemented — see REFAC-06)
│   ├── config.rs             # AppConfig + CleanStats added here (or separate state.rs)
│   └── mod.rs
└── ui/
    ├── components.rs         # centered_rect made pub, draw_exit_modal remains
    ├── dashboard.rs          # draw_dashboard(f, &App) — reads app.stats
    ├── file_manager.rs       # draw_file_manager(f, &App) — centered_rect import removed
    ├── app_uninstaller.rs    # draw_app_uninstaller(f, &App) — centered_rect_abs replaced
    ├── settings.rs           # draw_settings(f, &App) — already near-correct
    └── mod.rs
```

### Pattern 1: Unified Draw Function Signature

**What:** All four draw functions adopt `fn draw_X(f: &mut Frame, app: &App)`.

**Why `&App` not `&mut App`:** The existing `draw_app_uninstaller` and `draw_settings` pass `&mut App` but do not mutate it — they only read. The `&mut` borrow is unnecessary and makes calling from `terminal.draw(|f| {...})` awkward (Rust borrow checker requires the closure captures `&mut self` which then conflicts with the outer loop's borrow). Using `&App` removes this tension.

**Why not explicit params:** `draw_file_manager` has grown to 12 params. Adding one more field (CleanStats, etc.) would make it 13. Passing `&App` is the idiomatic ratatui pattern for apps beyond toy size. The tradeoff is that unit-testing draw functions requires constructing an `App` — acceptable given this app has no test suite yet.

**Stateful widgets:** The one genuine `&mut` requirement is `render_stateful_widget(widget, area, &mut state)`. The state (`TableState`) lives on `App`. Solution: pass `app: &App` and do `let mut table_state = app.file_table_state.clone()` before the draw call, OR switch to `app: &mut App` only for screens that call `render_stateful_widget`. The cleaner approach is to keep `&App` and make `TableState` cloneable (it already derives `Clone` in ratatui). However the simplest approach that avoids any clone cost: keep `&mut App` for the two screens that use stateful widgets (`FileManager`, `AppUninstaller`), and use `&App` for `Dashboard` and `Settings`. This is a pragmatic middle ground.

**Concrete signature plan:**
```rust
// src/ui/dashboard.rs
pub fn draw_dashboard(f: &mut Frame, app: &App)

// src/ui/file_manager.rs
pub fn draw_file_manager(f: &mut Frame, app: &mut App)  // needs &mut for stateful widget

// src/ui/app_uninstaller.rs
pub fn draw_app_uninstaller(f: &mut Frame, app: &mut App)  // already this, no change

// src/ui/settings.rs
pub fn draw_settings(f: &mut Frame, app: &App)  // remove &mut, it's never mutated
```

**Callsite in app.rs** (inside `terminal.draw` closure):
```rust
terminal.draw(|f| {
    match self.mode {
        AppMode::Dashboard    => draw_dashboard(f, self),
        AppMode::FileManager  => draw_file_manager(f, self),
        AppMode::Settings     => draw_settings(f, self),
        AppMode::AppUninstaller => draw_app_uninstaller(f, self),
    }
    if self.show_exit_confirm {
        draw_exit_modal(f, self.exit_confirm_selected, &self.config.theme);
    }
})?;
```

### Pattern 2: Moving Scan Trigger Out of terminal.draw()

**What:** The FileManager scan kickoff at app.rs lines 199–213 is inside `terminal.draw(...)`. I/O side-effects inside a render callback are an architectural smell.

**Current code (problematic):**
```rust
// INSIDE terminal.draw(|f| { ... }) — BAD
AppMode::FileManager => {
    if self.scanned_files.is_empty() && !self.is_scanning {
        self.is_scanning = true;
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);
        let threshold = self.config.safety_threshold_days;
        tokio::task::spawn_blocking(move || { ... });
    }
    draw_file_manager(f, ...);
}
```

**Correct placement:** Before the `terminal.draw(...)` call, in the same loop body where the channel drain logic already lives:
```rust
// BEFORE terminal.draw — correct placement
if self.mode == AppMode::FileManager
    && self.scanned_files.is_empty()
    && !self.is_scanning
{
    self.is_scanning = true;
    let (tx, rx) = mpsc::channel();
    self.scan_rx = Some(rx);
    let threshold = self.config.safety_threshold_days;
    tokio::task::spawn_blocking(move || {
        let targets = crate::utils::platform::get_scan_targets();
        crate::core::file_scanner::FileScanner::scan_targets(&targets, threshold, tx);
    });
}
```

This is functionally identical — the first render frame after entering FileManager mode, `is_scanning` will be `true` and the loading screen will show — but it no longer triggers a side-effect from inside a render function.

### Pattern 3: Extracting centered_rect to components.rs

**Current state:** Three implementations of effectively the same function:
- `src/ui/file_manager.rs`: `fn centered_rect(width, height, r)` — uses Layout-based centering
- `src/ui/app_uninstaller.rs`: `fn centered_rect_abs(width, height, r)` — uses direct Rect arithmetic
- `src/ui/components.rs`: `fn centered_rect(width, height, r)` — identical to file_manager.rs version (private)

The `file_manager.rs` and `components.rs` versions are byte-for-byte identical. The `app_uninstaller.rs` version is functionally equivalent but uses a different algorithm.

**Correct action:**
1. Make the `centered_rect` in `components.rs` `pub`
2. Delete the private copy in `file_manager.rs`
3. Replace `centered_rect_abs` calls in `app_uninstaller.rs` with the shared `centered_rect` (behavior is identical for reasonable screen sizes)
4. Add `use crate::ui::components::centered_rect;` imports

**Why the Layout-based version is preferred:** It handles edge cases (screen smaller than modal) more gracefully via `saturating_sub`. The direct arithmetic version does clamp with `.min(r.width)` and `.min(r.height)` so both are safe, but the Layout version is the established ratatui idiom.

### Pattern 4: ScanEvent Migration (REFAC-04)

**Current state:** `ScanEvent` is defined in `src/models/file_info.rs` lines 4-8. It carries `FinishedApps(Vec<AppInfo>)` which creates a cross-model import (`models/file_info.rs` imports `models/app_info::AppInfo`).

**Target location:** `src/core/events.rs` (new file).

**Migration steps:**
1. Create `src/core/events.rs` with the `ScanEvent` enum
2. Add `pub mod events;` to `src/core/mod.rs`
3. Update all `use crate::models::file_info::ScanEvent` imports to `use crate::core::events::ScanEvent`
4. Remove `ScanEvent` from `file_info.rs` (also remove the `use crate::models::app_info::AppInfo` import that was only needed for `ScanEvent`)
5. Update `app.rs` match arms which currently use `crate::models::file_info::ScanEvent::*`

**Files that reference ScanEvent:**
- `src/app.rs` — match arms on channel receive (lines 110, 113, 120)
- `src/core/file_scanner.rs` — sends events on the channel
- `src/core/app_scanner.rs` — sends events on the channel
- `src/models/file_info.rs` — definition (to be removed)

### Pattern 5: Per-Operation Progress State (REFAC-05)

**Current state:** `app.scan_progress_text: String` is overloaded for four operations:
- File scan progress (streaming from FileScanner)
- App scan progress (streaming from AppScanner)
- Deletion progress (streaming from FileOps)
- Archive progress (streaming from FileOps)

**Correct approach:** Split into semantically distinct fields on `App`:
```rust
pub scan_progress_text: String,    // file scan + app scan (both use scan_rx)
pub delete_progress_text: String,  // deletion (uses delete_rx)
pub archive_progress_text: String, // archiving (uses archive_rx)
```

**No change needed to background threads** — they already send to separate channels (`scan_rx`, `delete_rx`, `archive_rx`). Only the drain logic in `app.rs` needs updating to write to the correct field. UI draw functions then read from the correct field.

**This eliminates the semantic ambiguity without any behavioral change.**

### Pattern 6: Dashboard Stats Persistence (REFAC-07)

**What:** Add a `CleanStats` struct to the existing `AppConfig` or as a parallel persisted struct in the same config path.

**Recommended approach:** Add `CleanStats` as a field inside `AppConfig` (not a separate file), since:
- `AppConfig` already has `save()` and `load()` with serde_json
- Single config file is simpler than two
- `CleanStats` will be updated at the same time as a clean operation completes (same save call)

```rust
// In src/models/config.rs

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanStats {
    pub last_clean_date: Option<DateTime<Local>>,  // chrono already has serde feature
    pub total_files_deleted: u64,
    pub total_bytes_freed: u64,
}

// Add to AppConfig struct:
pub struct AppConfig {
    pub theme: AppTheme,
    pub archive_dir: PathBuf,
    pub safety_threshold_days: u32,
    pub stats: CleanStats,          // NEW — defaults to CleanStats::default()
}
```

**Updating stats:** After a successful delete or archive operation completes (in `app.rs` where `is_deleting = false` is set), increment `app.config.stats.total_files_deleted`, set `last_clean_date = Some(Local::now())`, add bytes freed, then call `app.config.save()`.

**Dashboard display:** `draw_dashboard` already receives `&App` (post-REFAC-01). Read `app.config.stats.last_clean_date` and format with chrono `humanize` or manual delta calculation. No new dependency needed — `chrono` already provides `DateTime` arithmetic.

**Backward compat:** Adding a new field with `#[serde(default)]` or `Default` impl means existing `config.json` files without `stats` will load without error — serde fills in `Default::default()`.

### Anti-Patterns to Avoid

- **Passing `&mut App` to functions that only read:** `draw_settings` currently takes `&mut App` but never mutates. This prevents calling it from contexts where a shared borrow exists.
- **Side-effects in render closures:** Starting async tasks, writing to channels, or mutating state from inside `terminal.draw(|f| {...})` will cause borrow conflicts and is architecturally wrong in ratatui apps.
- **Keeping ScanEvent in the data models layer:** It creates an upward dependency (`file_info.rs` depends on `app_info.rs`) that pollutes the model layer with messaging primitives.
- **One god progress string:** Using a single string field for multiple concurrent operation states creates silent bugs if operations overlap (possible during rapid user interaction).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Date persistence | Custom date string serializer | chrono with serde feature (already present) | Already in Cargo.toml; DateTime<Local> serializes to ISO 8601 |
| Stats persistence | New file format or database | serde_json to existing config.json pattern | AppConfig.save() already works; just extend the struct |
| Centered rect math | New centering algorithm | The existing `centered_rect` in components.rs (make it pub) | Three copies exist already; consolidate, don't create a fourth |

**Key insight:** This phase is consolidation, not construction. Every "new" piece of code in this phase should be smaller than what it replaces.

---

## Common Pitfalls

### Pitfall 1: Borrow Conflicts When Unifying to &mut App

**What goes wrong:** `terminal.draw(|f| { ... })` takes a closure. If the closure captures `self` as `&mut Self`, and inside it you call `draw_file_manager(f, self)` which also takes `&mut App`, the borrow checker may complain because `self` is already borrowed by the closure.

**Why it happens:** In ratatui 0.30, `terminal.draw` takes `FnOnce(&mut Frame)`. The closure captures `self` from the outer `run()` method. If the draw function signature is `&mut App`, Rust sees a nested mutable borrow.

**How to avoid:** Use `&App` (shared reference) for draw functions wherever possible. For stateful widgets, the only `&mut` needed is `&mut TableState`, which can be extracted before the closure:
```rust
// If you need to avoid &mut App in draw:
// Option A: clone the TableState for rendering (zero-cost for a small struct)
// Option B: use Cell<TableState> for interior mutability
// Option C: pass &mut App but verify no borrow conflict in practice
```

In practice, ratatui 0.30 closures often work with `&mut App` captures because the closure borrows `self` exclusively for the duration of `draw()`. But `&App` is safer and communicates intent clearly.

**Warning signs:** `cannot borrow *self as mutable because it is also borrowed as immutable` at the `terminal.draw` call site.

### Pitfall 2: serde_json Backward Compatibility on Config Fields

**What goes wrong:** Adding a new field `stats: CleanStats` to `AppConfig` without `#[serde(default)]` will cause `serde_json::from_str` to fail (error, not default) when reading an existing config.json that lacks the field.

**Why it happens:** serde_json by default requires all non-Option fields to be present in the JSON.

**How to avoid:** Either derive `Default` on `CleanStats` and add `#[serde(default)]` to the field, or make it `Option<CleanStats>`. The `#[serde(default)]` approach is cleaner:
```rust
#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub theme: AppTheme,
    pub archive_dir: PathBuf,
    pub safety_threshold_days: u32,
    #[serde(default)]
    pub stats: CleanStats,
}
```

**Warning signs:** App panics or silently returns default config on startup after adding the new field.

### Pitfall 3: Import Churn from ScanEvent Move

**What goes wrong:** Moving `ScanEvent` to `src/core/events.rs` requires updating every import site. If any import is missed, the build fails with a clear error — but if the refactor is done incrementally across commits, intermediate states may not compile.

**How to avoid:** Make the move atomic (one commit). Do a global search for `models::file_info::ScanEvent` and `file_info::ScanEvent` before committing. The affected files are: `app.rs`, `core/file_scanner.rs`, `core/app_scanner.rs`, and `models/file_info.rs` itself.

**Warning signs:** `error[E0412]: cannot find type ScanEvent in module crate::models::file_info`.

### Pitfall 4: Breaking Table Scroll State When Refactoring

**What goes wrong:** `render_stateful_widget` needs `&mut TableState`. If the draw function signature changes to `&App` (immutable), the stateful widget call breaks because it needs `&mut`.

**How to avoid:** Keep `draw_file_manager` and `draw_app_uninstaller` on `&mut App` since they use stateful widgets. Keep `draw_dashboard` and `draw_settings` on `&App`. This is the pragmatic hybrid — not perfect purity but zero borrow-checker fights.

### Pitfall 5: Bytes Freed Calculation Is Non-Trivial

**What goes wrong:** To record accurate `total_bytes_freed` in `CleanStats`, you need to sum `size_bytes` of selected items before deletion. After deletion, the items are removed from state.

**Why it happens:** The current deletion flow in `handlers/file_manager.rs` calls `FileOps::execute_deletion` and then `FileOps::retain_unselected`. The size information exists on `FileInfo::size_bytes` but must be summed before `retain_unselected` removes the items.

**How to avoid:** Sum selected file sizes immediately when the delete confirmation is confirmed (before spawning the thread), store in a local variable or temporary field, and add to `stats.total_bytes_freed` when the delete thread completes.

---

## Code Examples

Verified patterns from direct source inspection:

### Extending AppConfig with Stats (REFAC-07)
```rust
// src/models/config.rs — additive change, backward compatible
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanStats {
    pub last_clean_date: Option<DateTime<Local>>,
    pub total_files_deleted: u64,
    pub total_bytes_freed: u64,
}

// In AppConfig struct, add:
#[serde(default)]
pub stats: CleanStats,
```

### Making centered_rect Public (REFAC-02)
```rust
// src/ui/components.rs — change fn to pub fn
pub fn centered_rect(width: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    // ... existing implementation unchanged ...
}

// src/ui/file_manager.rs — replace local definition with import
use crate::ui::components::centered_rect;
// DELETE the local `fn centered_rect` definition

// src/ui/app_uninstaller.rs — replace centered_rect_abs calls
use crate::ui::components::centered_rect;
// DELETE fn centered_rect_abs
// Replace: centered_rect_abs(62, 10, size) -> centered_rect(62, 10, size)
// Replace: centered_rect_abs(62, 12, size) -> centered_rect(62, 12, size)
```

### New core/events.rs (REFAC-04)
```rust
// src/core/events.rs — new file
pub enum ScanEvent {
    Progress(String),
    Finished(Vec<crate::models::file_info::FileInfo>),
    FinishedApps(Vec<crate::models::app_info::AppInfo>),
}
```

### Scan Trigger Moved Out of Draw (REFAC-03)
```rust
// src/app.rs — place this BEFORE the terminal.draw(...) call
if self.mode == AppMode::FileManager
    && self.scanned_files.is_empty()
    && !self.is_scanning
{
    self.is_scanning = true;
    let (tx, rx) = std::sync::mpsc::channel();
    self.scan_rx = Some(rx);
    let threshold = self.config.safety_threshold_days;
    tokio::task::spawn_blocking(move || {
        let targets = crate::utils::platform::get_scan_targets();
        crate::core::file_scanner::FileScanner::scan_targets(&targets, threshold, tx);
    });
}
```

### Removing AppInfo::is_expanded and App::related_files_state (REFAC-06)
```rust
// src/models/app_info.rs — remove field:
// DELETE: pub is_expanded: bool,
// DELETE from AppInfo::new(): is_expanded: false,

// src/app.rs — remove field:
// DELETE: pub related_files_state: TableState,
// DELETE from App::new(): related_files_state: TableState::default(),
// No handler files reference either field — confirmed by reading source
```

---

## Concrete Risk Assessment: Changing Draw Function Signatures

| Screen | Current Signature | Target Signature | Risk | Notes |
|--------|------------------|-----------------|------|-------|
| `draw_dashboard` | `(f, selected_menu, sys, theme)` | `(f, &App)` | LOW | Pure display, no stateful widget, read-only |
| `draw_file_manager` | `(f, files, is_scanning, ..., theme)` — 12 params | `(f, &mut App)` | MEDIUM | Uses `render_stateful_widget`; needs `&mut file_table_state` |
| `draw_app_uninstaller` | `(f, &mut App)` | `(f, &mut App)` | NONE | No change needed |
| `draw_settings` | `(f, &mut App)` | `(f, &App)` | LOW | Never mutates; remove `&mut` |

**For `draw_dashboard` migration:** Replace all 4 call params with `app.selected_menu`, `&app.sys`, `&app.config.theme` internally, or just access them from `app` inside the function. Callsite simplifies from `draw_dashboard(f, self.selected_menu, &self.sys, &self.config.theme)` to `draw_dashboard(f, self)`.

**For `draw_file_manager` migration:** Replace 12 individual params with `app.` field access throughout the function body. The `scan_progress_text` reference will change to `&app.scan_progress_text` (or after REFAC-05, to the specific field). Total lines changed: ~20 at callsite + ~12 param list lines. No logic changes.

**What could break:** Nothing should break functionally — this is a mechanical substitution. The risk is compilation errors from missed references. Mitigation: `cargo build` after each file's migration before moving to the next.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Per-param draw functions | `&App` or `&mut App` pass-through | ratatui 0.26+ community standard | Eliminates param explosion as app grows |
| Manual Rect math for centering | Layout-based centering (already in components.rs) | Always been idiomatic in ratatui | Use the Layout version consistently |

---

## Open Questions

1. **REFAC-06: Remove or implement `AppInfo::is_expanded`?**
   - What we know: Field exists, never rendered, never toggled
   - What's unclear: Is expand/collapse for related files in the app uninstaller desired for Phase 2?
   - Recommendation: Remove it now (keep code clean). If expand/collapse is wanted in Phase 2, re-add with proper implementation. REQUIREMENTS.md does not list expand/collapse for app uninstaller as a v1 requirement.

2. **Stats: Accumulate or overwrite per session?**
   - What we know: REFAC-07 says "persist to config/state file" — `total_files_deleted` implies accumulation across sessions
   - What's unclear: Should `total_bytes_freed` be a lifetime total or "last session" total?
   - Recommendation: Implement as lifetime running totals (increment on each delete/archive completion). This matches the mental model of "how much has Clario cleaned total."

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | None currently installed |
| Config file | None — see Wave 0 |
| Quick run command | `cargo test` |
| Full suite command | `cargo test -- --test-threads=1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REFAC-01 | All draw functions compile with unified signature | smoke | `cargo build` | N/A — build check |
| REFAC-02 | `centered_rect` callable from all UI modules | smoke | `cargo build` | N/A — build check |
| REFAC-03 | No I/O in draw closure | manual | code review | N/A |
| REFAC-04 | `ScanEvent` importable from `core::events` | smoke | `cargo build` | ❌ Wave 0 |
| REFAC-05 | Progress fields semantically distinct | unit | `cargo test` | ❌ Wave 0 |
| REFAC-06 | `AppInfo::is_expanded` removed; project compiles | smoke | `cargo build` | N/A — build check |
| REFAC-07 | Stats persist across restart | integration | manual test sequence | ❌ Wave 0 |

Most REFAC items are structural/compilation verifiable — `cargo build` is the primary gate. REFAC-07 requires a manual run to verify stats survive restart.

### Sampling Rate
- **Per task commit:** `cargo build`
- **Per wave merge:** `cargo build && cargo test`
- **Phase gate:** `cargo build` clean with zero warnings before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/core/events.rs` — new file for ScanEvent (needed for REFAC-04)
- [ ] `#[serde(default)]` on `AppConfig.stats` field (needed for REFAC-07 backward compat)
- No test framework install required for this refactor phase — all verification is `cargo build` + manual

---

## Sources

### Primary (HIGH confidence)

Direct source file inspection of all files listed in research_focus:
- `/Users/ismailalam/Development/my/clario/src/app.rs` — run loop, App struct, scan trigger location
- `/Users/ismailalam/Development/my/clario/src/ui/dashboard.rs` — draw_dashboard signature, hardcoded stats (lines 231-256)
- `/Users/ismailalam/Development/my/clario/src/ui/file_manager.rs` — 12-param signature, centered_rect copy (lines 562-580)
- `/Users/ismailalam/Development/my/clario/src/ui/app_uninstaller.rs` — centered_rect_abs copy (lines 436-440)
- `/Users/ismailalam/Development/my/clario/src/ui/settings.rs` — draw_settings(&mut App) signature
- `/Users/ismailalam/Development/my/clario/src/ui/components.rs` — third centered_rect copy, draw_exit_modal
- `/Users/ismailalam/Development/my/clario/src/models/file_info.rs` — ScanEvent definition + cross-model dep
- `/Users/ismailalam/Development/my/clario/src/models/app_info.rs` — is_expanded unused field
- `/Users/ismailalam/Development/my/clario/src/models/config.rs` — AppConfig + save/load pattern
- `/Users/ismailalam/Development/my/clario/Cargo.toml` — full dependency list
- `/Users/ismailalam/Development/my/clario/.planning/REQUIREMENTS.md` — REFAC-01 through REFAC-07
- `/Users/ismailalam/Development/my/clario/.planning/codebase/ANALYSIS.md` — architectural overview

### Secondary (MEDIUM confidence)
- ratatui community pattern: `fn draw_X(f: &mut Frame, app: &App)` — idiomatic for medium+ apps; widely observable in ratatui example repos and the ratatui-templates project
- serde `#[serde(default)]` backward compat behavior — well-documented serde feature, standard Rust

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all findings from direct Cargo.toml inspection
- Architecture: HIGH — all findings from direct source code inspection, no speculation
- Pitfalls: HIGH — borrow checker and serde behaviors are well-understood Rust mechanics
- Scan trigger fix: HIGH — behavioral equivalence confirmed by reading the full app.rs run loop

**Research date:** 2026-03-23
**Valid until:** 2026-06-23 (ratatui API is stable; serde behavior is stable; no time-sensitive findings)
