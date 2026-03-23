use chrono::{DateTime, Local};
use std::path::PathBuf;

/// Menandakan apakah file ini berbahaya untuk dihapus atau tidak.
/// Mirip Enum di Java, tapi ini versi Rust.
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyLevel {
    SafeToDelete,
    ProceedWithCaution,
    SystemCritical,
}

/// Menandakan kategori dari sebuah file sampah.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Document/Application/Archive reserved untuk Phase 3 vulnerability scan
pub enum FileCategory {
    Cache,
    Log,
    Document,
    Application,
    Archive,
    Other,
}

/// DTO (Data Transfer Object) untuk setiap file yang kita scan.
#[derive(Debug, Clone)]
pub struct FileInfo {
    #[allow(dead_code)] // Reserved: akan dipakai sebagai identifier di Phase 3
    pub id: String,                     // UUID string
    pub name: String,                   // Nama file
    pub path: PathBuf,                  // Path lengkap (kayak os.Path error safe)
    pub size_bytes: u64,                // Ukuran file
    pub last_modified: DateTime<Local>, // Terakhir dimodifikasi
    pub is_dir: bool,                   // True kalau ini folder
    pub category: FileCategory,         // Kategori file
    pub safety: SafetyLevel,            // Keamanan hapus file
    pub children: Vec<FileInfo>,        // Anak-anak (jika ini directory)
    pub is_expanded: bool,              // Status toggle UI (terbuka/tertutup)
    pub is_selected: bool,              // Menandai jika user mem-pilih (centang) file ini
}

impl FileInfo {
    // Ini mirip constructor di Java / func NewFileInfo() di Go
    pub fn new(name: String, path: PathBuf, size: u64, is_dir: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(), // Generate UUID v4
            name,
            path,
            size_bytes: size,
            last_modified: Local::now(),
            is_dir,
            category: FileCategory::Other,     // Default
            safety: SafetyLevel::SafeToDelete, // Default
            children: Vec::new(),
            is_expanded: false,
            is_selected: false,
        }
    }
}
