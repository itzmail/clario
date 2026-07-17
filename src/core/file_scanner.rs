use crate::models::file_info::{FileCategory, FileInfo, SafetyLevel};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use walkdir::WalkDir;

pub struct FileScanner;

impl FileScanner {
    /// Melakukan scan (baca) secara dasar terhadap daftar folder target.
    /// Mengembalikan list flat berisi setiap file di dalam target (bukan tree).
    #[allow(dead_code)] // Reserved: general-purpose flat scanner, no current caller
    pub fn scan_targets(targets: &[PathBuf], safety_threshold_days: u32) -> Vec<FileInfo> {
        let mut results = Vec::new();
        let now = std::time::SystemTime::now();

        for target in targets {
            if !target.exists() {
                continue; // Lewati jika foldernya tidak ada
            }

            for entry in WalkDir::new(target).into_iter().filter_map(Result::ok) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let Ok(meta) = entry.metadata() else { continue };
                let name = entry.file_name().to_string_lossy().to_string();

                let mut info = FileInfo::new(name, path.to_path_buf(), meta.len(), false);
                info.category = Self::guess_category(path);
                info.safety = Self::guess_safety(path, now, safety_threshold_days);
                results.push(info);
            }
        }

        results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        results
    }

    /// Scan satu direktori root, kembalikan satu `FileInfo` per entry langsung di
    /// bawahnya (depth 1) dengan `size_bytes` = total ukuran rekursif entry itu
    /// (kalau folder) — breakdown ala `du -d1`. Dipakai oleh TUI `clario analyze`.
    ///
    /// Tiap entry top-level di-scan di thread terpisah (folder besar seperti
    /// `~/.cache` atau `~/Library` bisa berisi jutaan file dan mendominasi waktu
    /// total kalau dijalankan sekuensial). Berhenti lebih awal kalau `cancel`
    /// `true` di tengah scan (dipakai TUI analyze saat user pindah direktori
    /// sebelum scan folder besar sebelumnya selesai). `on_entry` dipanggil tiap
    /// entry selesai dengan `FileInfo`-nya — dipakai untuk streaming list live
    /// di TUI browser, bukan cuma progress counter.
    pub fn scan_depth1(
        root: &Path,
        cancel: &AtomicBool,
        on_entry: impl Fn(FileInfo) + Sync,
    ) -> Vec<FileInfo> {
        let Ok(entries) = std::fs::read_dir(root) else {
            return Vec::new();
        };

        let entries: Vec<_> = entries.filter_map(Result::ok).collect();

        let mut results: Vec<FileInfo> = std::thread::scope(|scope| {
            let handles: Vec<_> = entries
                .into_iter()
                .map(|entry| {
                    let on_entry = &on_entry;
                    scope.spawn(move || {
                        if cancel.load(Ordering::Relaxed) {
                            return None;
                        }
                        let path = entry.path();
                        let meta = entry.metadata().ok()?;
                        let is_dir = meta.is_dir();
                        let name = entry.file_name().to_string_lossy().to_string();
                        let size = if is_dir { dir_size(&path, cancel) } else { meta.len() };
                        if cancel.load(Ordering::Relaxed) {
                            return None;
                        }
                        let info = FileInfo::new(name, path, size, is_dir);
                        on_entry(info.clone());
                        Some(info)
                    })
                })
                .collect();

            handles.into_iter().filter_map(|h| h.join().ok().flatten()).collect()
        });

        results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        results
    }

    /// Implementasi awal untuk menebak kategori file berdasarkan path
    fn guess_category(path: &Path) -> FileCategory {
        // to_string_lossy() memastikan tidak crash meskipun nama OS file menggunakan karakter unicode aneh (misal nama folder bahasa Jepang)
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.contains("cache") || path_str.ends_with(".cache") {
            FileCategory::Cache
        } else if path_str.contains("log") || path_str.ends_with(".log") {
            FileCategory::Log
        } else {
            FileCategory::Other
        }
    }

    /// Implementasi Heuristic untuk menentukan keamanan file berdasarkan lokasinya
    fn guess_safety(path: &Path, now: std::time::SystemTime, threshold_days: u32) -> SafetyLevel {
        let path_str = path.to_string_lossy().to_lowercase();

        // Cek umur file (hari ini - modified date)
        let is_recently_modified = if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified_time) = metadata.modified() {
                if let Ok(duration) = now.duration_since(modified_time) {
                    duration.as_secs() < (threshold_days as u64 * 86400) // Konversi ke detik
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        // Aturan Heuristic Clario:
        // 1. Kalau path memuat kata 'system' atau 'root' atau 'windows' -> Bisa bikin OS crash!
        if path_str.contains("system") || path_str.contains("root") || path_str.contains("windows")
        {
            SafetyLevel::SystemCritical
        }
        // 2. Jika file ini masih aktif dipakai (dimodifikasi dalam ambang batas threshold hari yang disetel user di settings)
        else if is_recently_modified {
            SafetyLevel::ProceedWithCaution
        }
        // 2. Library Sistem (macOS) - Boleh dihapus tapi mungkin aplikasi minta re-login atau sedikit lag pas dibuka awal
        else if path_str.starts_with("/library/") {
            SafetyLevel::ProceedWithCaution
        }
        // 3. User Cache dan User Logs biasanya 99% aman untuk di-'sapu' habis
        else if path_str.contains("temp")
            || path_str.contains("cache")
            || path_str.contains("log")
        {
            SafetyLevel::SafeToDelete
        }
        // Default
        else {
            SafetyLevel::ProceedWithCaution
        }
    }
}

/// Total ukuran rekursif sebuah direktori. Entry yang tak bisa dibaca (permission
/// denied, symlink putus, dll) dilewati, bukan menggagalkan seluruh scan. Berhenti
/// (dan mengembalikan hitungan parsial) begitu `cancel` di-set — dicek per entry
/// karena `WalkDir` sendiri tidak punya hook cancellation bawaan.
fn dir_size(path: &Path, cancel: &AtomicBool) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .take_while(|_| !cancel.load(Ordering::Relaxed))
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
