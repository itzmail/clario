use crate::models::file_info::{FileCategory, FileInfo, SafetyLevel, ScanEvent};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use walkdir::WalkDir;

pub struct FileScanner;

impl FileScanner {
    /// Melakukan scan (baca) secara dasar terhadap daftar folder target
    pub fn scan_targets(targets: &[PathBuf], safety_threshold_days: u32, tx: Sender<ScanEvent>) {
        let mut results = Vec::new();
        let now = std::time::SystemTime::now();

        for target in targets {
            if !target.exists() {
                continue; // Lewati jika foldernya tidak ada di Mac kita
            }

            let target_name = target
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let mut target_node = FileInfo::new(target_name, target.to_path_buf(), 0, true);
            target_node.category = Self::guess_category(target);
            target_node.safety = Self::guess_safety(target, now, safety_threshold_days);

            // Baca isi target (misal ~/Library/Caches) -> isinya adalah App/Folder
            if let Ok(entries) = std::fs::read_dir(target) {
                for entry_res in entries.filter_map(Result::ok) {
                    let path = entry_res.path();
                    let name = entry_res.file_name().to_string_lossy().to_string();

                    // Kirim stream notifikasi ke UI tentang progress file manager
                    let _ = tx.send(ScanEvent::Progress(name.clone()));

                    let mut child_node = FileInfo::new(name, path.clone(), 0, path.is_dir());
                    child_node.category = target_node.category.clone();
                    child_node.safety = Self::guess_safety(&path, now, safety_threshold_days);

                    if path.is_dir() {
                        let mut dir_size = 0;
                        // Kita gabungkan file di dalamnya tanpa peduli hirarki anak-cucunya (Flatten files dari folder aplikasi tsb)
                        for inner in WalkDir::new(&path).into_iter().filter_map(Result::ok) {
                            let inner_path = inner.path();
                            if inner_path.is_file() {
                                if let Ok(meta) = inner.metadata() {
                                    dir_size += meta.len();

                                    // Limit RAM usage: Hanya record maskimal 150 file sample untuk di-render
                                    if child_node.children.len() < 150 {
                                        let mut f = FileInfo::new(
                                            inner.file_name().to_string_lossy().to_string(),
                                            inner_path.to_path_buf(),
                                            meta.len(),
                                            false,
                                        );
                                        f.category = child_node.category.clone();
                                        f.safety = Self::guess_safety(
                                            inner_path,
                                            now,
                                            safety_threshold_days,
                                        );
                                        child_node.children.push(f);
                                    }
                                }
                            }
                        }

                        // SORTING 3: Sort Anak level terdalam berdasarkan besar file (Descending)
                        child_node
                            .children
                            .sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

                        child_node.size_bytes = dir_size;
                        target_node.size_bytes += dir_size;
                    } else if let Ok(meta) = entry_res.metadata() {
                        child_node.size_bytes = meta.len();
                        target_node.size_bytes += meta.len();
                    }

                    target_node.children.push(child_node);
                }
            }
            // SORTING 1: Sort Root Level items (e.g. log folder, cache folder) by Size Descending
            target_node
                .children
                .sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

            results.push(target_node);
        }

        // SORTING 2: Sort Target items (Root Targets from OS) by total size Descending
        results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

        let _ = tx.send(ScanEvent::Finished(results));
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
