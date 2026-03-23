# Phase 2: Security — Process Monitor - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

User bisa melihat semua running processes dari dalam Clario, dengan flagging otomatis untuk yang mencurigakan, dan opsi untuk kill process dengan konfirmasi. Process Monitor bisa diakses dari dashboard via hotkey 'p' baru.

</domain>

<decisions>
## Implementation Decisions

### Suspicious Flagging Criteria
Multi-rule combination — proses diberi flag jika memenuhi ≥1 kriteria berikut. Semakin banyak kriteria terpenuhi, semakin tinggi severity-nya (warning color → danger color):

- **D-01:** Path di luar trusted locations (`/usr/`, `/bin/`, `/sbin/`, `/System/`, `/Applications/`, `/Library/`) → executable dari `/tmp/`, `/var/folders/`, atau home dir adalah red flag karena legitimate macOS software selalu install di lokasi standar
- **D-02:** Nama process ≠ nama executable di path (misal: process bernama "Safari" tapi jalan dari path non-standar) → teknik klasik malware untuk blend in dengan trusted app names
- **D-03:** UID 0 (root) tapi path bukan system path (`/System/`, `/usr/sbin/`, `/sbin/`) → privilege escalation red flag, legitimate root processes selalu di system dirs
- **D-04:** CPU sustained >80% → indikasi cryptominer atau runaway process (bukan spike sesaat)
- **D-05:** Nama process mengandung known-bad strings: `xmrig`, `cryptonight`, `miner` → blacklist kecil hardcoded

### Process List Layout
- **D-06:** Layout mirip App Uninstaller: Left panel (65%) = scrollable table, Right panel (35%) = detail panel
- **D-07:** Kolom tabel (left panel): Name, PID, CPU%, RAM — cukup, tidak perlu lebih
- **D-08:** Suspicious entries di-highlight dengan warna `theme.warning()` atau `theme.danger()` sesuai severity
- **D-09:** Right panel detail: full executable path, owner/user, parent PID, uptime, dan **"⚠ Why suspicious:" section** berisi daftar rules yang terpenuhi (hanya muncul jika process diflag)

### Kill Flow UX
- **D-10:** Multi-select seperti App Uninstaller — user bisa pilih banyak process sekaligus sebelum kill
- **D-11:** Kill modal memiliki **3 tombol**: `[Cancel]` `[Graceful Kill]` `[Force Kill ⚡]`
  - Graceful Kill = SIGTERM (proses bisa cleanup sebelum mati, sopan)
  - Force Kill = SIGKILL (OS langsung terminate, tidak bisa di-resist — pakai kalau proses hang)
- **D-12:** Jika kill gagal (permission denied), tampilkan error message di footer/status bar — tidak panic

### Dashboard Navigation
- **D-13:** Hotkey baru: `p` → masuk ke Process Monitor mode
- **D-14:** Process Monitor ditambahkan sebagai item menu baru di dashboard (sejajar File Manager, Uninstaller, Settings)
- **D-15:** Footer help bar di dashboard diupdate untuk mencantumkan `[p] Process Monitor`

### Claude's Discretion
- Threshold CPU "sustained" vs spike (durasi window, bisa pakai moving average atau simple threshold pada refresh interval)
- Sorting default process list (by name? by CPU%? by suspicious flag first?)
- Exact color mapping untuk severity levels (1 rule = warning, 2+ rules = danger, atau threshold lain)

</decisions>

<specifics>
## Specific Ideas

- "Dari kamu perlu di jelaskan kenapa itu mencurigakan" — setiap flag harus ada reasoning, bukan hanya warna merah. Right panel wajib explain alasan suspicious-nya.
- Kill modal 3 tombol (bukan 2 seperti delete confirm yang ada) — Cancel / Graceful Kill / Force Kill
- Multi-select first, baru kill — bukan click langsung kill

</specifics>

<canonical_refs>
## Canonical References

No external specs — requirements fully captured in decisions above.

### Existing code to read before planning/implementing
- `src/app.rs` — `App` struct dan `AppMode` enum; tambahkan `ProcessMonitor` variant dan process state fields di sini
- `src/ui/app_uninstaller.rs` — Reference untuk pola Left panel (Table) + Right panel (detail) + confirm modal
- `src/handlers/app_uninstaller.rs` — Reference untuk pola multi-select + modal key handling
- `src/ui/dashboard.rs` — Tambah hotkey 'p' + menu item baru di action menu + footer help bar
- `Cargo.toml` — `sysinfo = "0.38.2"` sudah ada, tidak perlu tambah dep baru

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `App.sys: sysinfo::System` — sudah ada dan sudah refresh tiap 2 detik via `sys.refresh_all()`. Process scanner tinggal call `sys.processes()` tanpa perlu setup ulang.
- `ui/components.rs::centered_rect` — untuk kill confirm modal (3-button modal baru)
- `models/app_info.rs::AppInfo` pattern — buat struct `ProcessInfo` serupa untuk process data
- Confirm modal pattern (`show_delete_confirm`, `delete_confirm_selected`) — adaptasi untuk kill modal dengan 3 state (0=Cancel, 1=Graceful, 2=Force)

### Established Patterns
- Screen pattern: `draw_X(f: &mut Frame, app: &mut App)` di `src/ui/`, handler di `src/handlers/`, mode di `AppMode` enum
- Navigation: global hotkeys di `app.rs::run()` main loop, mode-specific keys di `handlers::X::handle_key()`
- Background tasks: `mpsc::channel` + `tokio::task::spawn_blocking` — pakai ini jika butuh async kill (tidak langsung di main thread)
- `#[cfg(target_os = "macos")]` — gunakan untuk mac-specific process logic

### Integration Points
- `AppMode` enum di `src/app.rs` — tambah variant `ProcessMonitor`
- `terminal.draw()` match block — tambah arm `AppMode::ProcessMonitor => draw_process_monitor(f, self)`
- Global hotkey block — tambah `KeyCode::Char('p') => { self.mode = AppMode::ProcessMonitor; ... }`
- Dashboard menu render di `src/ui/dashboard.rs` — tambah item dan update footer

</code_context>

<deferred>
## Deferred Ideas

- Integrasi antivirus/YARA rules — sudah di-scope out di PROJECT.md, bukan phase ini
- Network connections per process (lsof-style) — interesting tapi bukan goal Phase 2, bisa jadi Phase 4+
- Process tree view (parent-child hierarchy) — nice to have, defer ke phase berikutnya jika ada

</deferred>

---

*Phase: 02-security-process-monitor*
*Context gathered: 2026-03-23*
