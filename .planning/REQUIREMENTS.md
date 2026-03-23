# Requirements: Clario

**Defined:** 2026-03-23
**Core Value:** Bersihkan sistem secara menyeluruh dan aman — tanpa GUI overhead, tanpa residue yang tertinggal.

## v1 Requirements

### Refactor

- [x] **REFAC-01**: Draw function signatures diunifikasi — semua screen pakai `&mut App` atau semua pakai explicit params (pilih satu, konsisten)
- [x] **REFAC-02**: `centered_rect` diekstrak ke `src/ui/components.rs` sebagai shared helper, duplikasi dihapus
- [ ] **REFAC-03**: FileManager scan kickoff dipindah keluar dari `terminal.draw(...)` closure ke handler atau lifecycle method yang tepat
- [ ] **REFAC-04**: `ScanEvent` dipindah dari `src/models/file_info.rs` ke `src/core/events.rs` (atau `src/app.rs`)
- [ ] **REFAC-05**: Shared `scan_progress_text` diganti dengan per-operation progress state yang terpisah
- [ ] **REFAC-06**: `AppInfo::is_expanded` dan `App::related_files_state` dihapus atau diimplementasi
- [ ] **REFAC-07**: Dashboard stats (last clean, files deleted, space freed) ditampilkan dari data real — persist ke config/state file

### Security — Process Monitor

- [ ] **SEC-01**: User bisa membuka screen Process Monitor dari dashboard
- [ ] **SEC-02**: App scan dan tampilkan semua running processes (nama, PID, CPU%, memory, path executable)
- [ ] **SEC-03**: App flag processes yang mencurigakan dengan kriteria yang jelas (path di luar /usr, /Applications, executable tidak dikenal, dsb)
- [ ] **SEC-04**: User bisa melihat detail process yang dipilih
- [ ] **SEC-05**: User bisa kill process dengan konfirmasi modal

### Security — Vulnerability Audit

- [ ] **SEC-06**: User bisa membuka screen Vulnerability Audit dari dashboard
- [ ] **SEC-07**: App scan dan tampilkan LaunchAgents (`~/Library/LaunchAgents`, `/Library/LaunchAgents`) dengan flag untuk yang tidak dikenal
- [ ] **SEC-08**: App scan dan tampilkan LaunchDaemons (`/Library/LaunchDaemons`) dengan flag untuk yang tidak dikenal
- [ ] **SEC-09**: App scan dan deteksi files dengan SUID/SGID bit di luar path standar sistem (`/usr/bin`, `/bin`, dsb)
- [ ] **SEC-10**: App scan dan deteksi world-writable files/dirs di lokasi sensitif (`/Library`, `/usr`, dsb)
- [ ] **SEC-11**: User bisa melihat detail tiap vulnerability finding (path, permission, risk level)
- [ ] **SEC-12**: User bisa disable/remove LaunchAgent atau LaunchDaemon dengan konfirmasi


## v2 Requirements

### Security — Malware/External Tools

- **MALL-01**: Integrasi opsional dengan ClamAV untuk file-level malware scan
- **MALL-02**: YARA rules support untuk custom threat signatures
- **MALL-03**: Quarantine flow (isolate suspected files sebelum delete)

### Architecture (Tauri-ready)

- **ARCH-01**: Business logic diekstrak ke `clario-core` library crate yang tidak ada dependency ke ratatui/crossterm
- **ARCH-02**: TUI binary menjadi thin consumer dari `clario-core`
- **ARCH-03**: Public API `clario-core` terdokumentasi dengan doc comments yang cukup untuk Tauri consumer

### Desktop (Tauri)

- **TAURI-01**: Tauri app yang mengkonsumsi `clario-core` library
- **TAURI-02**: Feature parity antara TUI dan desktop untuk core workflows
- **TAURI-03**: Auto-update mechanism untuk distribusi

### Extended Cleanup

- **CLEAN-01**: Duplicate file finder
- **CLEAN-02**: Large file scanner dengan threshold yang configurable
- **CLEAN-03**: Mail attachments cleanup
- **CLEAN-04**: Browser cache cleanup per-browser (Chrome, Safari, Firefox)

## Out of Scope

| Feature | Reason |
|---------|--------|
| External antivirus (ClamAV, YARA) v1 | Distribusi binary eksternal, kompleksitas tinggi — v2 |
| Tauri desktop build | Menunggu core library extraction; API perlu stabil dulu |
| Windows/Linux security features | SUID/LaunchAgents macOS-specific; lintas platform nanti |
| Real-time file system watcher | Complexity tinggi, bukan prioritas cleaner workflow |
| Cloud sync / backup integration | Out of scope untuk system cleaner |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| REFAC-01 | Phase 1 | Complete (01-01) |
| REFAC-02 | Phase 1 | Complete (01-01) |
| REFAC-03 | Phase 1 | Pending |
| REFAC-04 | Phase 1 | Pending |
| REFAC-05 | Phase 1 | Pending |
| REFAC-06 | Phase 1 | Pending |
| REFAC-07 | Phase 1 | Pending |
| SEC-01 | Phase 2 | Pending |
| SEC-02 | Phase 2 | Pending |
| SEC-03 | Phase 2 | Pending |
| SEC-04 | Phase 2 | Pending |
| SEC-05 | Phase 2 | Pending |
| SEC-06 | Phase 3 | Pending |
| SEC-07 | Phase 3 | Pending |
| SEC-08 | Phase 3 | Pending |
| SEC-09 | Phase 3 | Pending |
| SEC-10 | Phase 3 | Pending |
| SEC-11 | Phase 3 | Pending |
| SEC-12 | Phase 3 | Pending |
**Coverage:**
- v1 requirements: 19 total
- Mapped to phases: 19
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-23*
*Last updated: 2026-03-23 after initial definition*
