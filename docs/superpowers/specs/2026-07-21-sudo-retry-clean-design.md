# Sudo retry for permission-denied deletes in `clean`

## Problem

`run_clean` in `src/cli/clean.rs` deletes each selected item via `trash::delete`
(non-Trash items) or `std::fs::remove_dir_all`/`remove_file` (Trash items).
When an item is root-owned (Docker overlay files, container-created
`node_modules`/`target` entries, etc.), the delete fails with a permission
error and is reported as `failed` with no recovery path. The user has to exit
and manually `sudo rm` the leftover, which defeats the point of a one-shot
cleanup tool.

Mole (`reference/Mole`, macOS bash tool) solves the equivalent problem with a
sudo-aware deletion funnel (`safe_sudo_remove`, `mole_delete`) that retries
privileged paths through `sudo -n` and classifies the failure. Clario is a
cross-platform (Linux + macOS) Rust CLI, so the concept is ported, not the
implementation: no AppleScript/Touch ID fallback, and unlike Mole's
`-n` (non-interactive, fail-closed) sudo probe, Clario prompts for a password
interactively when a TTY is available, since the user is present and has
already confirmed the delete once.

## Scope

`src/cli/clean.rs` only. No new module. No changes to scanners, protection
rules, or the confirm/select UI upstream of the delete loop.

## Behavior

1. The existing delete loop (`run_clean`, item loop) is unchanged in its
   happy path. Each failure is inspected: if the underlying error is a
   permission-denied condition, the item is added to a `perm_failed: Vec<&FileInfo>`
   list instead of just being printed as failed. Other failure kinds (path
   gone, filesystem error, etc.) keep today's behavior — printed as `failed`
   and dropped.
2. Permission-denied detection per delete path:
   - `trash::delete()` (non-Trash items): matches `trash::Error::FileSystem { source, .. }`
     where `source.kind() == io::ErrorKind::PermissionDenied` (Linux/BSD
     freedesktop backend), or `trash::Error::CouldNotAccess { .. }` (covers
     macOS/other backends where no wrapped `io::Error` is available).
   - `std::fs::remove_dir_all` / `remove_file()` (Trash items): matches
     `io::Error::kind() == PermissionDenied`.
3. After the loop, if `perm_failed` is non-empty:
   - If `io::stdin().is_terminal()`: print a one-line notice with the count,
     then prompt `Retry with sudo? [y/N]`.
     - On `y`: run a single batched command,
       `sudo rm -rf <path1> <path2> ...`, with all `perm_failed` paths as
       args (spinner label: "Retrying with sudo"). Exit success marks every
       path in the batch as recovered (`freed` gains each item's
       `size_bytes`). Exit failure (wrong password, cancelled, etc.) leaves
       all of them failed.
     - On anything else (`N`/empty/Esc): skip, items stay failed.
   - If stdin is not a TTY: skip the prompt entirely, items stay failed. This
     matches the existing non-interactive branch further up in `run_clean`
     (`--force` / piped stdin already bypasses the item `MultiSelect`).
4. Final summary (before the existing `Freed:` line) gains up to two
   conditional lines:
   - `Recovered via sudo: {N} item(s), {size}` — only if the batch retry
     succeeded and recovered at least one item.
   - `Skipped (permission): {N} item(s) — rerun with sudo to retry` — only if
     items remain in `perm_failed` after the retry step (declined, non-TTY,
     or sudo failed).

## Non-goals

- No per-file sudo retry — always batched into one `sudo rm -rf` call so the
  user enters a password once.
- No `sudo -n` / silent probing — Clario always shows the password prompt
  through the inherited TTY when retry is confirmed; there's no cached-
  credential short-circuit to design around.
- No new CLI flag. Existing `--force` continues to mean "skip the item
  selection confirmation"; it does not imply skipping the sudo retry
  prompt — that's gated purely on TTY presence per the existing pattern at
  `clean.rs:87`.
- Docker cleanup (`docker system prune`) is untouched — it already runs as
  a single subprocess call and is out of scope for this change.

## Testing

No automated test covers an interactive sudo prompt (requires a real TTY and
root-owned fixture files, which CI can't provide safely). Verification is
manual:

1. `sudo touch /tmp/clario-test-root-owned` (or similar) to create a
   root-owned file inside a path one of the scanners covers.
2. Run `clario clean` (or a targeted category) so the file is selected for
   deletion.
3. Confirm the delete loop reports the permission failure, the sudo retry
   prompt appears, and answering `y` with the correct password removes the
   file and the summary shows `Recovered via sudo: 1 item(s), ...`.
4. Repeat with `N` at the prompt and with stdin piped
   (`echo | clario clean --force`) to confirm the skip path and the
   `Skipped (permission)` summary line.
5. `cargo build` / `cargo clippy` for compile and lint correctness.
