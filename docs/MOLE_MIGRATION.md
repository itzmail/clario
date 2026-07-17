# Clario × Mole — Migration Context

This file exists so a future session (or a different AI agent) can pick up
this work without re-deriving the reasoning from scratch. `AGENTS.md` and
`PLAN.md` are stale (they still describe the old TUI architecture) — treat
this file as the current source of truth for direction, and update it as
work continues.

## Why this direction

The user tried [Mole](https://github.com/tw93/mole) (`reference/Mole/`, a
git submodule/vendored copy kept for reference) on macOS and preferred its
CLI-subcommand UX (`mo clean`, `mo uninstall`, `mo purge`, ...) over Clario's
original ratatui TUI, which felt slow to navigate for what is fundamentally
a "run it, see the result, done" tool.

Decision made explicitly with the user (see conversation history, session
"implement-mole"): **rebuild Clario as a Mole-style CLI, dropping the TUI
entirely, Linux-first** (Mole is macOS-only bash; Clario's advantage is being
native Rust — no bash + coreutils dependency, faster scanning, no shell
quoting bugs). macOS support may return later but is not the current focus.

**Update**: macOS support returned. The user's dev machine is macOS, so
`uninstall` (session "implement-mole", see below), then `clean`/`purge` (a
later session) were both extended to run on macOS too. Linux remains the
primary target; macOS is now a fully supported second platform for these
three subcommands specifically — not a blanket "macOS is back" decision.

## What was removed (and why it's gone for good)

All ratatui-era code was deleted, not refactored:
- `src/app.rs`, `src/ui/*`, `src/handlers/*` — pure TUI rendering/event loop.
- `src/core/app_scanner.rs` — was `#![cfg(target_os = "macos")]` only
  (`.app` bundle + `Info.plist` parsing). No Linux equivalent exists yet;
  this is the biggest remaining gap (see "Not yet built" below).
- `src/core/events.rs`, `src/core/file_ops.rs` — mpsc-channel streaming and
  select/expand-tree helpers, meaningless without a TUI.
- `src/models/config.rs`, `app_info.rs`, `process_info.rs` — TUI-only state.
- Dependencies dropped: `ratatui`, `crossterm`, `uuid`, `plist`.

`FileInfo` (`src/models/file_info.rs`) was stripped down to just
`name/path/size_bytes/last_modified/is_dir/category/safety` — the
`children`/`is_expanded`/`is_selected`/`id` fields were TUI tree-view state.

`src/core/file_scanner.rs` was kept (rewritten to return a flat `Vec` instead
of a tree, and to not depend on the deleted `events.rs`) as the future basis
for a `clario analyze` subcommand, marked `#[allow(dead_code)]` until that
subcommand exists — nothing calls it yet.

## Current CLI surface (as of this writing)

```
clario clean [category] [--min-size] [--force] [--dry-run]
  categories: cargo, node, go, python, java, ruby, docker, cache, trash
clario purge [--min-size] [--force] [--dry-run] [--include-recent] [--paths]
clario update [version]
```

Both `clean` and `purge` are **global by design** — this was an explicit
correction mid-session. The very first cut of `clean` had Cargo/Node/
Python/Java project-artifact scanning scoped to the current working
directory (`cwd_has_marker` + `cwd.join("target")` etc., mirroring the
original pre-Mole code). The user pointed out Mole always scans globally
(`mo clean` walks the whole Home/Library, never "the folder you happen to
be in"), so that CWD logic was deleted from `dev_scanner.rs` entirely.
Project-artifact detection (`target/`, `node_modules/`, `.venv/`,
`__pycache__/`, `.gradle/`) now lives in one place, `purge_scanner.rs`, and
`clean` calls into it (filtered to the artifact names relevant to each
language category) instead of having its own duplicate CWD-based scanner.
**Do not reintroduce a CWD-scoped scan mode** without the user asking for it
again — it was deliberately removed, not an oversight.

### `clean` categories and what backs them

| Category | Source | Scope |
|---|---|---|
| `cargo` | `dev_scanner::scan_cargo` (global `~/.cargo/registry/*`) + `purge_scanner` filtered to `target` | global |
| `node` | `dev_scanner::scan_node` (global `~/.npm/_cacache`, pnpm stores) + `purge_scanner` filtered to `node_modules` | global |
| `go` | `dev_scanner::scan_go` (global `~/go/pkg/mod`, go-build cache) | global |
| `python` | `dev_scanner::scan_python` (global pip cache) + `purge_scanner` filtered to `.venv`/`venv`/`__pycache__` | global |
| `java` | `dev_scanner::scan_java` (global Gradle/Maven cache) + `purge_scanner` filtered to `.gradle` | global |
| `ruby` | `dev_scanner::scan_ruby` (global `~/.gem`) | global |
| `docker` | `dev_scanner::scan_docker` (`docker system df`) | n/a (daemon-wide) |
| `cache` | `dev_scanner::scan_cache` — **per-subfolder breakdown** of `~/.cache` (Linux) or `~/Library/Caches` + `/Library/Caches` (macOS), generic scan not a hardcoded per-app list like Mole's `app_caches.sh` | global |
| `trash` | `dev_scanner::scan_trash` — `~/.local/share/Trash/files` (Linux) or `~/.Trash` (macOS), one `FileInfo` per direct child | global, Linux + macOS (`#[cfg(any(target_os = "linux", target_os = "macos"))]`) |

All other `dev_scanner` scanners (`scan_cargo`, `scan_node`, `scan_go`,
`scan_python`, `scan_java`, `scan_ruby`) were already cross-platform before
this — they read from `Paths` (`src/utils/paths.rs`), which already had a
macOS branch for every field. Only `scan_trash` needed a genuinely new
per-OS implementation (different Trash location/layout). `purge_scanner.rs`
needed **no changes at all** to work on macOS — it has zero `#[cfg]` gates
and its default search paths (`~/dev`, `~/Projects`, etc.) are already
OS-agnostic. The only macOS-specific default Mole has that Clario
deliberately does not (`~/Library/CloudStorage`) is still skipped — that
was an explicit prior decision, not an oversight, and stays that way unless
asked again.

### macOS path/app protection (`src/core/protection.rs`)

Ported from Mole's `lib/core/app_protection.sh` +
`lib/core/app_protection_data.sh` + the validation half of
`lib/core/file_ops.sh`. macOS-only (`#[cfg(target_os = "macos")]`) since the
data (bundle IDs, `/System` paths) is macOS-specific — Linux `clean`/`purge`
has no equivalent and doesn't need one (its scanners never touch
arbitrary app-bundle paths).

What's ported (verbatim data, subset of logic):
- `SYSTEM_CRITICAL_BUNDLES` (~110 patterns) and `DATA_PROTECTED_BUNDLES`
  (~280 patterns, 30 categories: password managers, IDEs, AI tools, VPNs,
  etc.) — copied 1:1 from `app_protection_data.sh`.
- `ENDPOINT_SECURITY_BUNDLE_PREFIXES` (CrowdStrike, SentinelOne, etc.) —
  EDR agent caches under `/private/var/folders/*` that look regenerable but
  trip tamper detection if touched.
- `should_protect_path()` — the clean/purge-relevant subset of Mole's
  ~460-line function: system UI keyword matches, sandboxed container
  bundle-ID extraction, endpoint-security check, critical preference files,
  iCloud/Keychain/Mail/Contacts/Calendars, CoreAudio caches, and the full
  bundle-pattern sweep against both the full path and the filename.
- `is_critical_deletion_path()` (from `_mole_is_critical_deletion_path` in
  `file_ops.sh`) — hardcoded critical roots (`/System`, `/Library`,
  `/Users/<name>` itself, etc.), the last-resort backstop even if a category
  scanner somehow produced a path this broad.
- `is_safe_to_delete()` — combines the above three checks, mirrors Mole's
  `validate_path_for_deletion()`.
- Whitelist: `~/.config/clario/whitelist`, one pattern per line, `#`
  comments, `~` expansion, `*` glob + parent/child containment — mirrors
  `is_path_whitelisted()` (the glob-aware version used at actual delete
  time, not the exact-match-only version used by Mole's `whitelist.sh` UI
  management commands, since Clario has no whitelist-management subcommand
  yet).

**Explicitly NOT ported** (out of scope — this is a clean/purge filter, not
an uninstall leftover finder):
- `find_app_files()` and all uninstall-leftover heuristics (vendor-nested
  matching, embedded bundle-ID scanning, sibling-guard, LaunchAgent name
  matching). That logic belongs to `uninstall.rs`'s own leftover detection
  (`app_scanner.rs`), which already has its own simpler heuristic and was
  built in an earlier session.
- Uninstall-mode branches inside `should_protect_path()` (the
  `MOLE_UNINSTALL_MODE` conditionals that relax protection when the user
  explicitly chose to remove an app).
- Whitelist management commands (`clario whitelist add/remove/list`) — only
  the read side (`load_whitelist()`) exists; there's no CLI surface to edit
  the file yet, same as Mole's file being hand-edited before any UI existed.

**Wiring**: `is_safe_to_delete()` + `is_path_whitelisted()` are applied in
the **filtering stage** in both `clean.rs` and `purge.rs` — right after the
`min_size`/`SystemCritical` filters, before the summary table is printed.
This was a deliberate fix mid-session: an earlier version checked protection
only in the delete loop, which meant `--dry-run` (which returns before the
delete loop) showed protected items in its preview as if they'd be deleted,
then silently skipped them on a real run. **Protection must be a filter on
`filtered`, not a guard inside the delete loop** — if this code is touched
again, keep that invariant so dry-run stays an honest preview.

Deliberate choice on `cache`: Mole hardcodes ~40 named apps
(`lib/clean/app_caches.sh` — Slack, Spotify, Xcode, etc). We chose a
**generic XDG scan** instead — walk `~/.cache` one level deep, report each
subfolder as its own line — because Linux's XDG convention already gives us
this for free without per-app maintenance. If asked to add per-app special
casing later, push back first; that was an explicit user decision, not a
placeholder.

### `purge` design

`src/core/purge_scanner.rs` — scans configured project directories for
build-artifact directories. Modeled directly on Mole's
`lib/clean/purge_shared.sh` + `lib/clean/project.sh`:

- **Search paths**: `~/.config/clario/purge_paths`, one path per line,
  `#`-comments allowed. First run with no config auto-discovers from a
  fixed candidate list (`~/dev`, `~/Projects`, `~/GitHub`, `~/Code`,
  `~/Workspace`, `~/Repos`, `~/Development`, `~/www` — filtered to ones that
  exist) and saves the result, mirroring Mole's `load_purge_config`. Skipped
  Mole's `~/Library/CloudStorage` (macOS-only).
- **Project root detection**: presence of `package.json`, `Cargo.toml`,
  `go.mod`, `.git`, etc. (`PROJECT_INDICATORS`) — a directory only counts as
  a project if one of these exists in it.
- **Artifact target list** (`PURGE_TARGETS`): a trimmed-down version of
  Mole's `MOLE_PURGE_TARGETS` — kept the common cross-language ones
  (`node_modules`, `target`, `dist`, `.venv`/`venv`, `__pycache__`,
  `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, `.tox`, `.gradle`, `.next`,
  `.nuxt`, `vendor`, `.turbo`, `.parcel-cache`), dropped the
  language-specific long tail Mole has (`.dart_tool`, `.zig-cache`,
  `DerivedData`, `Pods`, `.expo`, `.build` for Swift, etc.) — add back only
  if the user actually hits that language.
- **Recency guard**: a project directory modified in the last 7 days
  (`RECENT_THRESHOLD_DAYS`) is tagged `[Recent]` in the summary table and
  **excluded from deletion by default** — same intent as Mole's "default
  unselected if category has recent items" in its interactive checkbox UI,
  adapted for a non-interactive CLI as an opt-in flag: `--include-recent`
  overrides the exclusion.
- **No sudo anywhere in purge** — confirmed by reading Mole's source
  (`grep` for `ensure_sudo_session` across `bin/purge.sh` and
  `lib/clean/project.sh` found zero hits). Mole only prompts for sudo in
  `clean.sh`'s "System" section and a few Xcode Simulator/documentation
  caches in `dev.sh` — all root-owned macOS paths. Purge targets
  (`node_modules`, `target`, etc.) live inside the user's own project
  directories and never need elevated privilege. **Sudo handling is
  explicitly out of scope until a system-level cache category is built**
  (see "Not yet built").
- Deletion is **permanent** (`std::fs::remove_dir_all`), not routed through
  Trash — build artifacts are large and trivially regenerable, unlike the
  Trash-routed `clean` deletions.

### Trash-deletion bug fixed mid-session (don't reintroduce)

The first cut of `scan_trash()` returned **one `FileInfo` for the whole
`~/.local/share/Trash/files` directory** (via the generic `dir_info()`
helper). Since `clean`'s delete loop called `trash::delete(&item.path)`
unconditionally, running `clario clean trash` would have called
`trash::delete()` **on the Trash folder itself** — nesting it inside the
Trash rather than emptying it. Fixed by:
1. `scan_trash()` now returns one `FileInfo` per **direct child** of
   `Trash/files`.
2. The delete loop in `cli/clean.rs` special-cases
   `FileCategory::Trash` items to use `std::fs::remove_file`/
   `remove_dir_all` (permanent, direct) instead of `trash::delete`
   (which would just re-trash an already-trashed item).

If Trash-handling code is touched again, keep this invariant: **Trash
items are deleted directly, never via `trash::delete`.**

## Progress/spinner UX (mirrors Mole's `start_section_spinner`)

Mole's bash implementation (`lib/core/base.sh`) uses an inline spinner that
rewrites the same terminal line via `\r`, plus permanent "section" headers
that stay in scrollback once a category finishes. The user explicitly asked
for the **append-per-category-when-done** variant (not a single spinner
that vanishes into a final table) — see the two `AskUserQuestion` exchanges
in this session where the "1 baris live, update in-place" option was
presented first but the user picked "Append tiap kategori selesai, baris
baru per kategori" instead. Two pieces exist now:

- **`src/utils/spinner.rs`** — `spin<T>(label, work) -> T`. Spawns a plain
  `std::thread` that repaints `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` frames over the same line via
  `\r` every 80ms while `work` runs on the calling thread (blocking).
  Skips animation entirely when stdout isn't a TTY (`IsTerminal`) — piped/
  redirected output gets no escape codes, just the final line. No new
  dependency (`indicatif` was considered and explicitly rejected by the
  user in favor of a manual thread — see `AskUserQuestion` in this session).
- **`cli/clean.rs`'s `scan_step()`** wraps every scanner call in `spin()`
  and prints the result permanently right after (`→ Cargo cache... 107.0
  MB` or `nothing found`), so each category's line stays in scrollback —
  this is the "append per category" behavior the user asked for, built on
  top of the lower-level spinner primitive.
- The delete loops in both `cli/clean.rs` and `cli/purge.rs`, and the
  Docker-prune step, also go through `spin()` now — this was a follow-up
  request ("aku ingin diimplementasi ketika ada proses loading juga",
  meaning: not just scanning, deletion too).

If you need a similar progress pattern elsewhere (e.g. a future `analyze`
or `uninstall`), reuse `spin()` — don't reach for a progress-bar crate
unless multiple concurrent bars are actually needed at once (noted as a
`ponytail:` comment in `spinner.rs` itself).

## Not yet built (ranked by what was discussed, not by priority)

1. **`clario uninstall`** — the user's favorite Mole feature, explicitly
   named as such. Needs a from-scratch Linux implementation: enumerate
   installed apps via `.desktop` entries (`/usr/share/applications`,
   `~/.local/share/applications`), find leftover config/cache/data under
   XDG dirs (`~/.config`, `~/.cache`, `~/.local/share`) by app/package name
   instead of macOS bundle-ID/plist matching. The old `app_scanner.rs` (now
   deleted) is not reusable — it was 100% `.app`-bundle/`Info.plist`
   specific and had zero Linux logic.
2. **`clario analyze`** — disk explorer. `src/core/file_scanner.rs` already
   exists (flat-list rewrite, no TUI dependency) as a starting point but is
   currently unused dead code (`#[allow(dead_code)]` on the module in
   `core/mod.rs`) — no subcommand wires it up yet.
3. **`clario status`** — lightweight snapshot (not a live dashboard —
   explicitly decided against mirroring Mole's live-updating `mo status`,
   to avoid "becoming an iStat clone", quoting Mole's own product
   philosophy in `reference/Mole/CLAUDE.md`). `src/core/process_scanner.rs`
   was deleted with the TUI; a new one-shot version would need to be
   written.
4. **System-level cache cleanup** (`/var/log`, journalctl, `/var/cache/apt`)
   — explicitly deferred twice in this session. This is the only category
   that would need sudo/privilege handling, analogous to Mole's
   `ensure_sudo_session` pattern in `lib/core/sudo.sh` (Touch ID detection,
   password prompt, session keepalive — all macOS-specific, would need a
   Linux equivalent built from scratch, likely just `sudo -v` + a keepalive
   loop). Do not build sudo scaffolding before this category is actually
   started — there is nothing else in the codebase that needs it yet.
5. Deeper per-app cache targeting, Windows support, and TUI revival are all
   explicitly **not** planned — see "What was removed" above.

## Reference material

`reference/Mole/` is a vendored copy of the upstream Mole repo, kept
read-only for cross-referencing behavior — never edit files under it.
Its `CLAUDE.md` documents Mole's own product philosophy and safety rules in
detail (Trash-routing, protected-path checks, the `MOLE_PURGE_TARGETS` list,
sudo/Touch ID handling, etc.) and is worth re-reading before extending any
of the areas above, since several of Clario's design choices were made by
directly reading that file and the corresponding shell source rather than
guessing.
