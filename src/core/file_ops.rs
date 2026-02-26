use crate::models::file_info::FileInfo;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::{ExtendedFileOptions, FileOptions};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ToggleAction {
    Expand,
    Select,
}

pub struct FileOps;

impl FileOps {
    /// Helper untuk mencari baris layar ke-N pada Flattened View lalu membalik state `is_expanded`-nya
    pub fn toggle_recursive(
        files: &mut [FileInfo],
        target_index: usize,
        current_index: &mut usize,
        action: ToggleAction,
    ) -> bool {
        for file in files.iter_mut() {
            if *current_index == target_index {
                match action {
                    ToggleAction::Expand => file.is_expanded = !file.is_expanded,
                    ToggleAction::Select => {
                        file.is_selected = !file.is_selected;
                        // Jika parent dipilih, anak-anaknya semua ikut dipilih
                        if file.is_dir {
                            Self::set_selection_all(&mut file.children, file.is_selected);
                        }
                    }
                }
                return true;
            }
            *current_index += 1;

            // Jika folder dan sedang dibuka (expanded), anak-anaknya otomatis dihitung indexnya!
            if file.is_expanded
                && file.is_dir
                && Self::toggle_recursive(&mut file.children, target_index, current_index, action)
            {
                return true;
            }
        }
        false
    }

    pub fn has_any_selected(files: &[FileInfo]) -> bool {
        for f in files {
            if f.is_selected {
                return true;
            }
            if f.is_expanded && f.is_dir && Self::has_any_selected(&f.children) {
                return true;
            }
        }
        false
    }

    pub fn set_selection_all(files: &mut [FileInfo], checked: bool) {
        for f in files.iter_mut() {
            f.is_selected = checked;
            if f.is_dir {
                Self::set_selection_all(&mut f.children, checked);
            }
        }
    }

    pub fn execute_deletion(
        scanned_files: &[FileInfo],
        tx: std::sync::mpsc::Sender<Option<String>>,
    ) {
        // Collect all paths to delete first, so we don't have to borrow `scanned_files` across the thread boundary
        let mut paths_to_delete = Vec::new();
        Self::collect_selected_paths(scanned_files, &mut paths_to_delete);

        // Spawn tokio background task thread
        tokio::task::spawn_blocking(move || {
            for path in paths_to_delete {
                // Beri tahu UI file/folder apa yang sedang dihapus
                let msg = path.to_string_lossy().to_string();
                let _ = tx.send(Some(msg));

                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
            // Kirim sinyal (None) bahwa I/O hapus selesai total!
            let _ = tx.send(None);
        });
    }

    fn collect_selected_paths(files: &[FileInfo], paths: &mut Vec<PathBuf>) {
        for f in files.iter() {
            if f.is_selected {
                paths.push(f.path.clone());
            } else if f.is_dir {
                Self::collect_selected_paths(&f.children, paths);
            }
        }
    }

    pub fn execute_archiving(
        scanned_files: &[FileInfo],
        tx: std::sync::mpsc::Sender<Option<String>>,
    ) {
        let mut paths_to_archive = Vec::new();
        Self::collect_selected_paths(scanned_files, &mut paths_to_archive);

        if paths_to_archive.is_empty() {
            let _ = tx.send(None);
            return;
        }

        tokio::task::spawn_blocking(move || {
            let user_dirs = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
            let archive_dir = user_dirs.join("Clario_Archives");

            if !archive_dir.exists() {
                let _ = std::fs::create_dir_all(&archive_dir);
            }

            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let archive_name = format!("Clario_Archive_{}.zip", timestamp);
            let archive_path = archive_dir.join(archive_name);

            let file = std::fs::File::create(&archive_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);

            let mut zip_success = true;

            for path in &paths_to_archive {
                // Stream active progress info!
                let _ = tx.send(Some(path.to_string_lossy().to_string()));

                if path.is_file() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    let options: FileOptions<'_, ExtendedFileOptions> = FileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated)
                        .unix_permissions(0o755);

                    if zip.start_file(name.into_owned(), options).is_err() {
                        zip_success = false;
                        break;
                    }

                    if let Ok(mut f) = std::fs::File::open(path) {
                        let mut buffer = Vec::new();
                        if f.read_to_end(&mut buffer).is_ok() {
                            if zip.write_all(&buffer).is_err() {
                                zip_success = false;
                                break;
                            }
                        }
                    }
                } else if path.is_dir() {
                    for entry in walkdir::WalkDir::new(path)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        let inner_path = entry.path();
                        if inner_path.is_file() {
                            let relative = inner_path.strip_prefix(path).unwrap_or(inner_path);
                            let parent_folder_name = path.file_name().unwrap_or_default();
                            let archived_name = Path::new(parent_folder_name)
                                .join(relative)
                                .to_string_lossy()
                                .to_string();

                            let options: FileOptions<'_, ExtendedFileOptions> =
                                FileOptions::default()
                                    .compression_method(zip::CompressionMethod::Deflated)
                                    .unix_permissions(0o755);
                            if zip.start_file(archived_name, options).is_err() {
                                zip_success = false;
                                break;
                            }

                            if let Ok(mut f) = std::fs::File::open(inner_path) {
                                let mut buffer = Vec::new();
                                if f.read_to_end(&mut buffer).is_ok() {
                                    if zip.write_all(&buffer).is_err() {
                                        zip_success = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            match zip.finish() {
                Ok(_) if zip_success => {
                    for path in &paths_to_archive {
                        if path.is_dir() {
                            let _ = std::fs::remove_dir_all(path);
                        } else {
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
                _ => {
                    let _ = std::fs::remove_file(&archive_path);
                }
            }

            // Kirim sinyal operasi IO sudah aman dan tutup UI loader
            let _ = tx.send(None);
        });
    }

    pub fn retain_unselected(files: &mut Vec<FileInfo>) {
        files.retain(|f| !f.is_selected);
        for f in files.iter_mut() {
            if f.is_dir {
                Self::retain_unselected(&mut f.children);
            }
        }
    }
}
