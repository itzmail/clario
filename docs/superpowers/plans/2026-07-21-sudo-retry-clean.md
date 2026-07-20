# Sudo Retry for Permission-Denied Clean Deletes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When `clario clean` fails to delete an item because it's root-owned, offer the user a single batched `sudo rm -rf` retry instead of silently reporting `failed`.

**Architecture:** All changes live in `src/cli/clean.rs`. The existing per-item delete loop starts classifying failures: permission-denied failures go into a side list instead of being printed immediately. After the loop, if that list is non-empty and stdin is a TTY, prompt once and run one batched `sudo rm -rf <paths...>` call through the existing `spin()` helper. The final summary gains up to two new lines reporting sudo-recovered and still-skipped counts.

**Tech Stack:** Rust, `trash` crate v5 (`trash::Error::FileSystem` / `CouldNotAccess`), `std::process::Command`, existing `spin()` spinner helper, `std::io::IsTerminal`.

## Global Constraints

- Batch only: one `sudo rm -rf` call for all permission-failed items, never per-file. (spec: "Non-goals")
- No `sudo -n`: always show the real password prompt through the inherited TTY when the user confirms retry. (spec: "Non-goals")
- No new CLI flag; `--force` continues to mean "skip item selection", unrelated to the sudo retry prompt, which is gated only on `io::stdin().is_terminal()`. (spec: "Non-goals")
- Trash items and non-Trash items share the same retry batch — no separate code path. (spec: "Behavior" step 1)
- Docker cleanup is untouched. (spec: "Non-goals")

---

### Task 1: Classify permission-denied failures in the delete loop

**Files:**
- Modify: `src/cli/clean.rs:120-146` (the delete loop inside `run_clean`)

**Interfaces:**
- Consumes: existing `to_delete: Vec<&FileInfo>` (already in scope at this point in `run_clean`), `FileInfo` fields `.path: PathBuf`, `.size_bytes: u64`, `.category: FileCategory`, `.is_dir: bool` (all defined in `src/models/file_info.rs`).
- Produces: a `perm_failed: Vec<&FileInfo>` vector, populated during the loop, consumed by Task 2 immediately after the loop in the same function. `perm_failed` entries borrow from `to_delete`, so lifetimes match already (`to_delete: Vec<&FileInfo>` already holds `'_` refs into `filtered`).

This task changes the existing loop from "print done/failed" into "print done, or classify failed as permission vs. other". No test framework exists for this interactive CLI path (verified: `rg -n "fn test_" src/cli/clean.rs` returns nothing, and there is no `tests/` directory in the repo — confirm with `find . -maxdepth 1 -name tests` before starting, it should print nothing). Verification for this task is a `cargo build` compile check plus manual reasoning about the match arms; Task 5 covers end-to-end manual verification once the whole feature is wired up.

- [ ] **Step 1: Add the `perm_failed` collector and classify `trash::delete` errors**

Open `src/cli/clean.rs`. Locate the delete loop (currently lines 120-146):

```rust
    // Delete files
    let mut freed: u64 = 0;
    for item in &to_delete {
        let path = item.path.clone();
        let is_trash_item = item.category == FileCategory::Trash;
        let is_dir = item.is_dir;
        // Trash items are already in the Trash — delete them permanently instead of
        // re-trashing (trash::delete on a Trash entry would just nest it deeper).
        let result = spin(&format!("Removing {}", path.display()), move || {
            if is_trash_item {
                if is_dir {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                }
            } else {
                trash::delete(&path).map_err(|e| io::Error::other(e.to_string()))
            }
        });
        match result {
            Ok(_) => {
                freed += item.size_bytes;
                println!("{}", "done".green());
            }
            Err(e) => println!("{} ({})", "failed".red(), e),
        }
    }
```

Replace it with a version that keeps the delete closure returning `io::Result<()>` for the Trash-item branch, but switches the non-Trash branch to return the raw `trash::Result<()>` mapped into a small local enum so the caller can tell "permission" apart from "everything else" without losing the original error's `Display` text. Use this exact replacement:

```rust
    // Delete files
    let mut freed: u64 = 0;
    let mut perm_failed: Vec<&FileInfo> = Vec::new();
    for item in &to_delete {
        let path = item.path.clone();
        let is_trash_item = item.category == FileCategory::Trash;
        let is_dir = item.is_dir;
        // Trash items are already in the Trash — delete them permanently instead of
        // re-trashing (trash::delete on a Trash entry would just nest it deeper).
        let result: Result<(), DeleteError> = spin(&format!("Removing {}", path.display()), move || {
            if is_trash_item {
                let r = if is_dir {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                };
                r.map_err(DeleteError::from_io)
            } else {
                trash::delete(&path).map_err(DeleteError::from_trash)
            }
        });
        match result {
            Ok(_) => {
                freed += item.size_bytes;
                println!("{}", "done".green());
            }
            Err(DeleteError::PermissionDenied(msg)) => {
                println!("{} ({})", "failed".red(), msg);
                perm_failed.push(item);
            }
            Err(DeleteError::Other(msg)) => {
                println!("{} ({})", "failed".red(), msg);
            }
        }
    }
```

Now add the `DeleteError` type. Place it near the top of the file, after the existing `use` statements and before `RECENT_THRESHOLD_DAYS` (i.e. right after line 8's `use std::io::{self, IsTerminal, Write};`):

```rust
/// Distinguishes a permission-denied delete failure (candidate for sudo retry)
/// from every other failure kind (path gone, filesystem error, etc.), which is
/// reported as failed and dropped, matching pre-existing behavior.
enum DeleteError {
    PermissionDenied(String),
    Other(String),
}

impl DeleteError {
    fn from_io(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::PermissionDenied {
            DeleteError::PermissionDenied(e.to_string())
        } else {
            DeleteError::Other(e.to_string())
        }
    }

    fn from_trash(e: trash::Error) -> Self {
        match &e {
            #[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios"), not(target_os = "android")))]
            trash::Error::FileSystem { source, .. } if source.kind() == io::ErrorKind::PermissionDenied => {
                DeleteError::PermissionDenied(e.to_string())
            }
            trash::Error::CouldNotAccess { .. } => DeleteError::PermissionDenied(e.to_string()),
            _ => DeleteError::Other(e.to_string()),
        }
    }
}
```

**Why this shape:** the spec (see `docs/superpowers/specs/2026-07-21-sudo-retry-clean-design.md`, "Behavior" step 2) requires matching `trash::Error::FileSystem { source, .. }` with `source.kind() == PermissionDenied` on the freedesktop (Linux/BSD) backend, and `trash::Error::CouldNotAccess` as a catch-all for backends without a wrapped `io::Error` (confirmed present in `trash` v5.2.5's `src/lib.rs` around line 136-163). The `#[cfg(...)]` guard matches the same cfg the `trash` crate itself uses for the `FileSystem` variant, so this compiles on both Linux and macOS targets (the crate simply won't have that variant on macOS, and the match arm is compiled out there).

- [ ] **Step 2: Compile check**

Run: `cargo build 2>&1 | tail -40`
Expected: compiles cleanly (or only pre-existing warnings unrelated to this change — run `git stash && cargo build 2>&1 | tail -40 && git stash pop` first if unsure what's pre-existing). If there's a "trash::Error variant not found" error, run `cargo doc -p trash --no-deps` and check the generated docs at `target/doc/trash/enum.Error.html` for the exact variant names/cfg for the installed version (should be 5.2.5 per `Cargo.lock`).

- [ ] **Step 3: Commit**

```bash
git add src/cli/clean.rs
git commit -m "[main] classify permission-denied clean delete failures

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

---

### Task 2: Batched sudo retry prompt and execution

**Files:**
- Modify: `src/cli/clean.rs` (add code right after the delete loop from Task 1, before the existing Docker cleanup block)

**Interfaces:**
- Consumes: `perm_failed: Vec<&FileInfo>` produced by Task 1, `freed: u64` (mutable, also from Task 1's scope), `spin()` from `crate::utils::spinner::spin` (already imported at top of file), `io::stdin().is_terminal()` (same pattern already used at `clean.rs:87`).
- Produces: updates `freed` in place for recovered items; produces `sudo_recovered_count: usize` and leaves `perm_failed` holding only the still-failed items after the retry attempt (drained on success, left alone on decline/failure) — both consumed by Task 3 for the summary lines.

The current file (after Task 1) has this shape right after the delete loop, before the Docker block:

```rust
    // Docker cleanup
    if docker_info.is_some() {
```

Insert the new sudo-retry block between the closing `}` of the delete loop and that `// Docker cleanup` comment.

- [ ] **Step 1: Add the retry prompt + batched sudo command**

Insert this block immediately after the delete loop's closing brace (i.e., right before the `// Docker cleanup` comment):

```rust
    // Offer a single batched sudo retry for permission-denied items. Only when a
    // TTY is present — non-interactive runs (piped stdin, CI) skip straight to
    // reporting them as skipped in the summary below.
    let mut sudo_recovered_count = 0usize;
    if !perm_failed.is_empty() && io::stdin().is_terminal() {
        println!(
            "\n{} {}",
            format!("{} item(s) failed due to permission", perm_failed.len()).yellow(),
            "— likely owned by root.".dimmed()
        );
        print!("{}", "Retry with sudo? [y/N] ".bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("y") {
            let paths: Vec<std::path::PathBuf> = perm_failed.iter().map(|f| f.path.clone()).collect();
            let status = spin("Retrying with sudo", move || {
                std::process::Command::new("sudo").arg("rm").arg("-rf").args(&paths).status()
            });
            match status {
                Ok(s) if s.success() => {
                    // Capture the count before draining — perm_failed.len() would be 0
                    // after drain(..) consumes it.
                    sudo_recovered_count = perm_failed.len();
                    for item in perm_failed.drain(..) {
                        freed += item.size_bytes;
                    }
                    println!("{}", "done".green());
                }
                _ => println!("{}", "failed".red()),
            }
        }
    }
```

- [ ] **Step 2: Compile check**

Run: `cargo build 2>&1 | tail -40`
Expected: compiles cleanly. If `freed` reports "cannot borrow as mutable" or similar, confirm `let mut freed: u64 = 0;` from Task 1 is still `mut` (it already was in the original code) and that this new block sits in the same function scope (inside `run_clean`, after the delete loop, before the Docker block) — no new function boundary was introduced.

- [ ] **Step 3: Commit**

```bash
git add src/cli/clean.rs
git commit -m "[main] add batched sudo retry prompt for permission-denied deletes

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

---

### Task 3: Summary lines for recovered/skipped counts

**Files:**
- Modify: `src/cli/clean.rs` (the final summary print, currently the single line right before `Ok(())` at the end of `run_clean`)

**Interfaces:**
- Consumes: `sudo_recovered_count: usize` and `perm_failed: Vec<&FileInfo>` (both from Task 2 — `perm_failed` is empty if fully recovered, or still holds entries if declined/non-TTY/sudo failed), `freed: u64`, `format_size` from `crate::utils::size::format_size` (already imported).
- Produces: final terminal output only; nothing downstream consumes this.

The current end of `run_clean` (after Task 2's insertion, still before this task) looks like:

```rust
    println!("\n{} {}", "Freed:".bold(), format_size(freed).green().bold());
    Ok(())
}
```

- [ ] **Step 1: Add the two conditional summary lines**

Replace:

```rust
    println!("\n{} {}", "Freed:".bold(), format_size(freed).green().bold());
    Ok(())
}
```

With:

```rust
    if sudo_recovered_count > 0 {
        println!(
            "{} {} item(s)",
            "Recovered via sudo:".bold(),
            sudo_recovered_count
        );
    }
    if !perm_failed.is_empty() {
        println!(
            "{} {} item(s) — rerun with sudo to retry",
            "Skipped (permission):".yellow(),
            perm_failed.len()
        );
    }
    println!("\n{} {}", "Freed:".bold(), format_size(freed).green().bold());
    Ok(())
}
```

Note: the recovered bytes are already summed into `freed` inside Task 2 Step 1 (`freed += item.size_bytes` inside the `drain(..)` loop), so `Recovered via sudo:` only needs to print the *count* — the freed-bytes total already includes the sudo-recovered amount and is shown on the `Freed:` line right below it.

- [ ] **Step 2: Compile check**

Run: `cargo build 2>&1 | tail -40`
Expected: compiles cleanly with no new warnings.

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --all-targets 2>&1 | tail -60`
Expected: no new warnings attributable to `src/cli/clean.rs` (pre-existing warnings elsewhere in the workspace are out of scope for this task).

- [ ] **Step 4: Commit**

```bash
git add src/cli/clean.rs
git commit -m "[main] report sudo-recovered and permission-skipped counts in clean summary

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```

---

### Task 4: Manual end-to-end verification

**Files:** none (verification only, no code changes expected — if this task finds a bug, fix it in `src/cli/clean.rs` and fold the fix into this task's commit)

**Interfaces:**
- Consumes: the full feature built across Tasks 1-3.
- Produces: confidence the feature works against a real root-owned file; no downstream task depends on this one.

This is the manual verification called for in the spec (`docs/superpowers/specs/2026-07-21-sudo-retry-clean-design.md`, "Testing" section), since an interactive sudo prompt can't be exercised by an automated test in this environment.

- [ ] **Step 1: Build the binary**

Run: `cargo build --release 2>&1 | tail -20`
Expected: `Compiling clario ...` then `Finished` with no errors.

- [ ] **Step 2: Create a root-owned fixture file inside a scanned path**

Run:
```bash
mkdir -p ~/.cache/clario-test-fixture
sudo touch ~/.cache/clario-test-fixture/root-owned-file
sudo chmod 600 ~/.cache/clario-test-fixture/root-owned-file
ls -la ~/.cache/clario-test-fixture/
```
Expected: the file is listed as owned by `root`, not your user.

Note: `~/.cache/clario-test-fixture` is an ad hoc throwaway directory for this manual check, not a path any scanner in `src/core/dev_scanner.rs` recognizes automatically. If `clario clean --category cache` (or the relevant scan) doesn't pick it up, the simplest path is to run `clario clean` against a *real* known-permission-restricted target on your machine (e.g. a Docker-created file under a project's `node_modules` if you have Docker Desktop/Engine creating root-owned files — check with `find . -user root -path "*/node_modules/*" 2>/dev/null` in a project directory first). Adapt the fixture location to whatever the scanners in this repo actually cover; the point of this step is only to have at least one root-owned file that ends up in the `filtered` list inside `run_clean`.

- [ ] **Step 3: Run clean and exercise the "yes" path**

Run: `./target/release/clario clean` (or `cargo run --release -- clean` if the binary name differs — check with `grep '^name' Cargo.toml`)

Interact with the prompt:
1. When the item multi-select appears, ensure the root-owned fixture item is selected (it's a default-selected item per the existing `defaults = vec![true; filtered.len()]` behavior).
2. Confirm the delete.
3. Observe the delete loop print `failed (Permission denied)` (or the `trash`-crate equivalent message) for that item.
4. Observe the new prompt: `1 item(s) failed due to permission — likely owned by root.` followed by `Retry with sudo? [y/N]`.
5. Type `y` and enter your sudo password when prompted.

Expected: the command succeeds, the file is gone (`ls ~/.cache/clario-test-fixture/` shows it removed), and the final summary includes `Recovered via sudo: 1 item(s)` followed by the `Freed:` line with a non-zero size.

- [ ] **Step 4: Exercise the "no" path**

Repeat Step 2 to recreate the fixture, run `clario clean` again, but type `N` (or just press Enter) at the sudo retry prompt.

Expected: the summary shows `Skipped (permission): 1 item(s) — rerun with sudo to retry` and the file still exists on disk (`ls` confirms).

- [ ] **Step 5: Exercise the non-TTY path**

Recreate the fixture again, then run:
```bash
echo | ./target/release/clario clean --force
```
Expected: no sudo prompt appears at all (stdin is piped, so `io::stdin().is_terminal()` is `false`), the summary shows `Skipped (permission): 1 item(s) — rerun with sudo to retry`, and the file still exists on disk.

- [ ] **Step 6: Clean up the fixture**

Run: `sudo rm -rf ~/.cache/clario-test-fixture`
Expected: exits 0, directory gone.

- [ ] **Step 7: Final commit (only if Step 1-6 required a code fix)**

If verification passed with no code changes, skip this step — there is nothing to commit. If a bug was found and fixed:

```bash
git add src/cli/clean.rs
git commit -m "[main] fix issue found during sudo retry manual verification

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>"
```
