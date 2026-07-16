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
    // Developer toolchain categories
    CargoBuild,
    CargoCache,
    NodeModules,
    NodeCache,
    Docker,
    GoBuild,
    GoCache,
    PythonCache,
    PythonVenv,
    JavaGradle,
    JavaMaven,
    RubyGems,
    AppCache,
    Trash,
}

/// DTO (Data Transfer Object) untuk setiap file yang kita scan.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,                   // Nama file
    pub path: PathBuf,                  // Path lengkap (kayak os.Path error safe)
    pub size_bytes: u64,                // Ukuran file
    pub last_modified: DateTime<Local>, // Terakhir dimodifikasi
    pub is_dir: bool,                   // True kalau ini folder
    pub category: FileCategory,         // Kategori file
    pub safety: SafetyLevel,            // Keamanan hapus file
}

impl FileInfo {
    // Ini mirip constructor di Java / func NewFileInfo() di Go
    pub fn new(name: String, path: PathBuf, size: u64, is_dir: bool) -> Self {
        Self {
            name,
            path,
            size_bytes: size,
            last_modified: Local::now(),
            is_dir,
            category: FileCategory::Other,     // Default
            safety: SafetyLevel::SafeToDelete, // Default
        }
    }
}
