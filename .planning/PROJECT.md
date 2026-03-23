# Clario

## What This Is

Clario adalah macOS system cleaner berbasis TUI (terminal UI) yang ditulis dalam Rust, terinspirasi dari CleanMyMac. App ini memberi pengguna kontrol penuh untuk membersihkan cache/junk files, uninstall aplikasi secara menyeluruh (termasuk Library artifacts), dan memantau keamanan sistem — semuanya dari terminal. Target jangka panjang: codebase yang sama bisa mentenagai versi desktop via Tauri.

## Core Value

Bersihkan sistem secara menyeluruh dan aman — tanpa GUI overhead, tanpa residue yang tertinggal.

## Requirements

### Validated

- ✓ File Manager — scan, tampilkan, dan hapus/archive cache & log files — Phase 0 (existing)
- ✓ App Uninstaller — deep scan + hapus app bundle beserta semua Library artifacts — Phase 0 (existing)
- ✓ Settings — theme switcher, archive dir, safety threshold — Phase 0 (existing)
- ✓ Theme system — 5 tema dengan color palette semantik end-to-end — Phase 0 (existing)
- ✓ TUI consistency refactor — unified draw signatures, no I/O in render, shared centered_rect, split progress state, real dashboard stats — Phase 1

### Active

- [ ] TUI consistency refactor — unify draw function signatures, fix architectural smells, dashboard stats real
- [ ] Security: Process Monitor — scan running processes, flag yang suspicious, opsi kill
- [ ] Security: Vulnerability Audit — SUID/SGID, world-writable dirs, LaunchAgents/Daemons audit

### Out of Scope

- External antivirus/malware tool integration (ClamAV, YARA) — deferred, kompleksitas tinggi, butuh distribusi binary eksternal
- Tauri desktop build — deferred, menunggu core library extraction selesai dulu
- Windows/Linux support aktif — cross-platform paths sudah ada tapi bukan prioritas sekarang

## Context

- Stack: Rust 2021, ratatui 0.30, crossterm 0.29, tokio 1.49, sysinfo (sudah di deps untuk RAM stats)
- 5 commit history — project masih early stage tapi core features sudah fungsional
- sysinfo crate sudah ada — bisa langsung dipakai untuk process scanning
- Inkonsistensi UI yang ada: `draw_dashboard`/`draw_file_manager` pakai explicit params (12 params!), `draw_app_uninstaller`/`draw_settings` pakai `&mut App` langsung
- Dashboard stats ("2 days ago", "142 files") masih hardcoded — perlu persistence layer sederhana
- `ScanEvent` dan `centered_rect` diduplikasi — perlu dikonsolidasikan
- Scan kickoff untuk FileManager ada di dalam render closure — architectural smell

## Constraints

- **Tech Stack**: Rust + ratatui — tidak boleh ganti framework TUI
- **Architecture**: Business logic harus terpisah dari UI layer — syarat untuk Tauri compatibility di masa depan
- **macOS Primary**: Fitur security (SUID, LaunchAgents) macOS-specific — wrap dengan `#[cfg(target_os = "macos")]`
- **No External Binaries**: Semua fitur harus pure Rust, tidak boleh shell out ke tools eksternal untuk fitur inti

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Refactor TUI dulu sebelum fitur baru | Inkonsistensi signature akan makin parah kalau terus di-add fitur; lebih murah fix sekarang | — Pending |
| sysinfo untuk process monitoring | Sudah ada di deps, proven untuk macOS/Linux, tidak perlu `ps` shell-out | — Pending |
| Core library extraction di Phase 4 | Fitur user-facing lebih prioritas dulu; ekstrak library setelah API-nya stabil lewat penggunaan real | — Pending |

---
*Last updated: 2026-03-23 after Phase 1 — TUI Refactor & Architecture Cleanup*
