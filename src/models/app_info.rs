use crate::models::file_info::FileInfo;
use std::path::PathBuf;

/// DTO untuk Aplikasi dan relasi sampahnya
#[derive(Debug, Clone)]
pub struct AppInfo {
    #[allow(dead_code)] // Reserved: akan dipakai sebagai identifier di Phase 3
    pub id: String,                   // UUID
    pub name: String,                 // "Google Chrome"
    pub bundle_id: Option<String>,    // "com.google.Chrome"
    pub path: PathBuf,                // "/Applications/Google Chrome.app"
    pub app_size_bytes: u64,          // Ukuran aplikasi utama
    pub total_size_bytes: u64,        // Ukuran Aplikasi + Sampah relasi
    pub related_files: Vec<FileInfo>, // Daftar cache/preferences/log app ini
    pub last_accessed: Option<chrono::DateTime<chrono::Local>>,
    pub is_selected: bool,
}

impl AppInfo {
    pub fn new(name: String, path: PathBuf, app_size: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            bundle_id: None,
            path,
            app_size_bytes: app_size,
            total_size_bytes: app_size,
            related_files: Vec::new(),
            last_accessed: None,
            is_selected: false,
        }
    }
}
