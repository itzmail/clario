# Roadmap: Clario

## Overview

Dari codebase yang sudah ada (File Manager + App Uninstaller), Clario dikembangkan menjadi macOS system cleaner yang komprehensif: pertama membersihkan technical debt di TUI, lalu menambah fitur security (process monitor + vulnerability audit), dan akhirnya mempersiapkan arsitektur yang reusable untuk Tauri di masa depan.

## Phases

- [ ] **Phase 1: TUI Refactor & Architecture Cleanup** - Unifikasi draw function signatures, fix architectural smells, dan dashboard stats dari data real
- [ ] **Phase 2: Security — Process Monitor** - Scan running processes, flag suspicious, opsi kill
- [ ] **Phase 3: Security — Vulnerability Audit** - LaunchAgents/Daemons audit, SUID/SGID detection, world-writable scan

## Phase Details

### Phase 1: TUI Refactor & Architecture Cleanup
**Goal**: Codebase konsisten dan bersih — semua draw functions punya signature yang seragam, tidak ada logic di dalam render closure, shared utilities tidak duplikat, dan dashboard menampilkan data real.
**Depends on**: Nothing (first phase)
**Requirements**: REFAC-01, REFAC-02, REFAC-03, REFAC-04, REFAC-05, REFAC-06, REFAC-07
**Success Criteria** (what must be TRUE):
  1. Semua 4 draw functions (dashboard, file_manager, app_uninstaller, settings) punya signature pattern yang konsisten
  2. Tidak ada I/O atau side-effect logic di dalam `terminal.draw(...)` closure
  3. `centered_rect` ada satu versi di `src/ui/components.rs`, tidak ada duplikasi
  4. Dashboard menampilkan data real dari persistent state (last clean date, files deleted count, space freed)
  5. `cargo build` clean tanpa warning, semua existing features masih berfungsi
**Plans**: 3 plans

Plans:
- [ ] 01-01: Unify draw function signatures and extract shared UI utilities
- [ ] 01-02: Fix architectural smells (scan trigger, ScanEvent location, progress state)
- [ ] 01-03: Implement dashboard stats persistence and real data display

### Phase 2: Security — Process Monitor
**Goal**: User bisa melihat semua running processes dari dalam Clario, dengan flagging otomatis untuk yang mencurigakan, dan opsi untuk kill process dengan konfirmasi.
**Depends on**: Phase 1
**Requirements**: SEC-01, SEC-02, SEC-03, SEC-04, SEC-05
**Success Criteria** (what must be TRUE):
  1. Screen Process Monitor bisa diakses dari dashboard menu
  2. Running processes ditampilkan dengan nama, PID, CPU%, memory, dan path executable
  3. Suspicious processes diberi visual indicator yang berbeda (warna/ikon)
  4. User bisa kill process dengan confirm modal
  5. Tidak ada panic jika permission denied saat akses process info
**Plans**: TBD

Plans:
- [ ] 02-01: Process scanner using sysinfo with suspicious detection logic
- [ ] 02-02: Process Monitor TUI screen with detail panel and kill flow

### Phase 3: Security — Vulnerability Audit
**Goal**: User bisa audit startup items (LaunchAgents/Daemons), SUID/SGID files, dan world-writable locations yang berpotensi jadi attack surface di macOS.
**Depends on**: Phase 2
**Requirements**: SEC-06, SEC-07, SEC-08, SEC-09, SEC-10, SEC-11, SEC-12
**Success Criteria** (what must be TRUE):
  1. Screen Vulnerability Audit bisa diakses dari dashboard menu
  2. LaunchAgents dan LaunchDaemons ditampilkan dengan flag untuk yang tidak dikenal
  3. SUID/SGID files di luar path standar sistem terdeteksi dan ditampilkan
  4. World-writable files/dirs di lokasi sensitif terdeteksi
  5. User bisa disable/remove LaunchAgent atau LaunchDaemon dengan backup
  6. Audit scan selesai dalam < 5 detik untuk macOS default setup
**Plans**: TBD

Plans:
- [ ] 03-01: Vulnerability scanner (LaunchAgents, SUID/SGID, world-writable)
- [ ] 03-02: Vulnerability Audit TUI screen with findings detail and remediation flow

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. TUI Refactor & Architecture Cleanup | 0/3 | Not started | - |
| 2. Security — Process Monitor | 0/2 | Not started | - |
| 3. Security — Vulnerability Audit | 0/2 | Not started | - |
