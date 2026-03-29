#![cfg(target_os = "macos")]

use crate::core::events::ScanEvent;
use crate::models::{app_info::AppInfo, file_info::FileInfo};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct AppScanner;

impl AppScanner {
    /// Mencari seluruh `.app` bundle di direktori Mac yang paling umum
    pub fn scan_applications(tx: std::sync::mpsc::Sender<ScanEvent>) -> Vec<AppInfo> {
        let mut results = Vec::new();

        let targets = vec![
            PathBuf::from("/Applications"),
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/"))
                .join("Applications"),
        ];

        for target in targets {
            if !target.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&target) {
                for entry_res in entries.filter_map(Result::ok) {
                    let path = entry_res.path();

                    // Di Mac, Aplikasi itu berakhiran *.app dan aslinya adalah sebuah Folder Directory
                    if path.is_dir() && path.extension().and_then(|s| s.to_str()) == Some("app") {
                        let name = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        let _ = tx.send(ScanEvent::Progress(name.clone()));

                        let mut app_info = AppInfo::new(name.clone(), path.clone(), 0);

                        // Baca Info.plist untuk mendapatkan Bundle ID "com.xxx.yy"
                        let plist_path = path.join("Contents").join("Info.plist");
                        if let Ok(dict) = plist::Value::from_file(&plist_path) {
                            if let Some(dict) = dict.as_dictionary() {
                                if let Some(bundle_id) =
                                    dict.get("CFBundleIdentifier").and_then(|v| v.as_string())
                                {
                                    app_info.bundle_id = Some(bundle_id.to_string());
                                }
                            }
                        }

                        // Hitung Ukuran Aplikasi (Karena *.app itu folder, kita muter isinya)
                        let mut app_size = 0;
                        let mut last_accessed: Option<chrono::DateTime<chrono::Local>> = None;
                        for inner in WalkDir::new(&path).into_iter().filter_map(Result::ok) {
                            if let Ok(meta) = inner.metadata() {
                                app_size += meta.len();
                                // Track MAX access time — kapan app terakhir benar-benar dipakai
                                let file_time = meta
                                    .accessed()
                                    .ok()
                                    .or_else(|| meta.modified().ok())
                                    .map(chrono::DateTime::<chrono::Local>::from);
                                if let Some(t) = file_time {
                                    last_accessed = Some(match last_accessed {
                                        None => t,
                                        Some(prev) => prev.max(t),
                                    });
                                }
                            }
                        }
                        app_info.app_size_bytes = app_size;
                        app_info.total_size_bytes = app_size;
                        app_info.last_accessed = last_accessed;

                        // Cari file Library/Caches yang bersangkut paut
                        Self::find_related_files(&mut app_info);

                        results.push(app_info);
                    }
                }
            }
        }

        // Urutkan aplikasi dari yang terlama diakses (unused paling lama di atas)
        results.sort_by(|a, b| {
            let time_a = a.last_accessed.unwrap_or_else(|| chrono::Local::now());
            let time_b = b.last_accessed.unwrap_or_else(|| chrono::Local::now());
            time_a.cmp(&time_b)
        });
        results
    }

    /// Mencari jejak file sampah aplikasi (Caches, Preferences, dll.) di Library OS macOS
    fn find_related_files(app: &mut AppInfo) {
        let home = match dirs::home_dir() {
            Some(dir) => dir,
            None => return,
        };

        let library = home.join("Library");
        if !library.exists() {
            return;
        }

        let search_dirs = vec![
            library.join("Caches"),
            library.join("Preferences"),
            library.join("Application Support"),
            library.join("Containers"),
            library.join("Logs"),
            library.join("Saved Application State"),
        ];

        // Pakai Bundle ID terlebih dahulu, jika gagal coba nama asli (lowercase strictness)
        let search_terms: Vec<String> = match &app.bundle_id {
            Some(id) => vec![id.to_lowercase(), app.name.to_lowercase()],
            None => vec![app.name.to_lowercase()],
        };

        for target_dir in search_dirs {
            if !target_dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&target_dir) {
                for entry in entries.filter_map(Result::ok) {
                    let file_name = entry.file_name().to_string_lossy().to_lowercase();

                    // Cek apakah file/folder di library ini punya nama yang mengandung ID aplikasi kita
                    let mut matched = false;
                    for term in &search_terms {
                        // Agar tidak salah hapus (misal `app` vs `apple`), kita perlu exact match atau prefix yang kuat
                        if file_name.starts_with(term) || file_name == *term {
                            matched = true;
                            break;
                        }
                    }

                    if matched {
                        let path = entry.path();
                        let mut size = 0;

                        if path.is_dir() {
                            for inner in WalkDir::new(&path).into_iter().filter_map(Result::ok) {
                                if let Ok(meta) = inner.metadata() {
                                    size += meta.len();
                                }
                            }
                        } else if let Ok(meta) = entry.metadata() {
                            size = meta.len();
                        }

                        if size > 0 {
                            let f_name = entry.file_name().to_string_lossy().to_string();
                            let mut related_node =
                                FileInfo::new(f_name, path, size, entry.path().is_dir());
                            // Default to Other, user can still see it's linked
                            related_node.category = crate::models::file_info::FileCategory::Other;
                            related_node.safety =
                                crate::models::file_info::SafetyLevel::SafeToDelete; // Usually library cache/prefs are safe

                            app.related_files.push(related_node);
                            app.total_size_bytes += size;
                        }
                    }
                }
            }
        }
    }
}
