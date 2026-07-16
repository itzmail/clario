use crate::models::file_info::{FileCategory, FileInfo, SafetyLevel};
use crate::utils::paths::Paths;
use std::path::Path;
use walkdir::WalkDir;

/// Aggregated info about Docker disk usage (from `docker system df`)
pub struct DockerInfo {
    pub images_size: u64,
    pub containers_size: u64,
    pub volumes_size: u64,
    pub build_cache_size: u64,
}

impl DockerInfo {
    pub fn total(&self) -> u64 {
        self.images_size + self.containers_size + self.volumes_size + self.build_cache_size
    }
}

/// Scan the global Cargo registry cache (not project target/ dirs — those are
/// covered globally across all projects by `purge_scanner`).
pub fn scan_cargo() -> Vec<FileInfo> {
    let mut results = Vec::new();
    let Some(paths) = Paths::new() else { return results };

    for path in &[&paths.cargo_registry_cache, &paths.cargo_registry_src] {
        if path.exists() {
            if let Some(info) = dir_info(path, FileCategory::CargoCache, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }
    }

    results
}

/// Scan the global npm/pnpm cache (not project node_modules/ — those are
/// covered globally across all projects by `purge_scanner`).
pub fn scan_node() -> Vec<FileInfo> {
    let mut results = Vec::new();
    let Some(paths) = Paths::new() else { return results };

    if paths.npm_cache.exists() {
        if let Some(info) = dir_info(&paths.npm_cache, FileCategory::NodeCache, SafetyLevel::SafeToDelete) {
            results.push(info);
        }
    }

    for pnpm_path in &paths.pnpm_stores {
        if pnpm_path.exists() {
            if let Some(info) = dir_info(pnpm_path, FileCategory::NodeCache, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }
    }

    results
}

/// Query Docker disk usage via `docker system df`.
/// Returns None if Docker is unavailable or the daemon isn't running.
pub fn scan_docker() -> Option<DockerInfo> {
    let output = std::process::Command::new("docker")
        .args(["system", "df", "--format", "{{.Type}}\t{{.Size}}\t{{.Reclaimable}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut info = DockerInfo {
        images_size: 0,
        containers_size: 0,
        volumes_size: 0,
        build_cache_size: 0,
    };

    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let size = parse_docker_size(parts[1]);
        match parts[0] {
            "Images" => info.images_size = size,
            "Containers" => info.containers_size = size,
            "Local Volumes" => info.volumes_size = size,
            "Build Cache" => info.build_cache_size = size,
            _ => {}
        }
    }

    Some(info)
}

/// Scan Go module cache and build cache.
pub fn scan_go() -> Vec<FileInfo> {
    let mut results = Vec::new();
    let Some(paths) = Paths::new() else { return results };

    if paths.go_mod_cache.exists() {
        if let Some(info) = dir_info(&paths.go_mod_cache, FileCategory::GoCache, SafetyLevel::SafeToDelete) {
            results.push(info);
        }
    }

    for go_build in &paths.go_build_caches {
        if go_build.exists() {
            if let Some(info) = dir_info(go_build, FileCategory::GoBuild, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }
    }

    results
}

/// Scan the global Python pip cache (not project __pycache__/venv — those are
/// covered globally across all projects by `purge_scanner`).
pub fn scan_python() -> Vec<FileInfo> {
    let mut results = Vec::new();
    let Some(paths) = Paths::new() else { return results };

    for pip_cache in &paths.pip_caches {
        if pip_cache.exists() {
            if let Some(info) = dir_info(pip_cache, FileCategory::PythonCache, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }
    }

    results
}

/// Scan the global Gradle and Maven caches (not project-local .gradle — that's
/// covered globally across all projects by `purge_scanner`).
pub fn scan_java() -> Vec<FileInfo> {
    let mut results = Vec::new();
    let Some(paths) = Paths::new() else { return results };

    if paths.gradle_cache.exists() {
        if let Some(info) = dir_info(&paths.gradle_cache, FileCategory::JavaGradle, SafetyLevel::SafeToDelete) {
            results.push(info);
        }
    }

    if paths.maven_repo.exists() {
        if let Some(info) = dir_info(&paths.maven_repo, FileCategory::JavaMaven, SafetyLevel::SafeToDelete) {
            results.push(info);
        }
    }

    results
}

/// Scan Ruby gems cache.
pub fn scan_ruby() -> Vec<FileInfo> {
    let mut results = Vec::new();
    let Some(paths) = Paths::new() else { return results };

    if paths.gem_dir.exists() {
        if let Some(info) = dir_info(&paths.gem_dir, FileCategory::RubyGems, SafetyLevel::SafeToDelete) {
            results.push(info);
        }
    }

    results
}

/// Scan system cache directories, broken down per app/subfolder (e.g. `~/.cache/google-chrome`,
/// `~/.cache/spotify`) instead of one lump sum, so the user can see which app is the biggest offender.
pub fn scan_cache() -> Vec<FileInfo> {
    let Some(paths) = Paths::new() else { return vec![] };

    let mut results = Vec::new();
    for base in paths.system_cache_dirs() {
        if !base.exists() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(base) else { continue };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(info) = dir_info(&path, FileCategory::AppCache, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }
    }
    results
}

/// Scan the user's Trash (already-deleted files awaiting permanent removal).
/// Each direct child of the Trash directory is its own item — emptying the Trash
/// means permanently deleting these paths directly, NOT re-trashing the Trash
/// folder itself. Linux: `~/.local/share/Trash/files`. macOS: `~/.Trash`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn scan_trash() -> Vec<FileInfo> {
    let Some(paths) = Paths::new() else { return vec![] };
    if !paths.trash_files.exists() {
        return vec![];
    }
    let Ok(entries) = std::fs::read_dir(&paths.trash_files) else { return vec![] };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_dir = path.is_dir();
            let size = if is_dir { dir_size(&path) } else { entry.metadata().ok()?.len() };
            let name = entry.file_name().to_string_lossy().to_string();
            let mut info = FileInfo::new(name, path, size, is_dir);
            info.category = FileCategory::Trash;
            info.safety = SafetyLevel::SafeToDelete;
            Some(info)
        })
        .collect()
}

// --- Helpers ---

/// Build a FileInfo for a directory with its total recursive size.
fn dir_info(path: &Path, category: FileCategory, safety: SafetyLevel) -> Option<FileInfo> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let size = dir_size(path);
    let mut info = FileInfo::new(name, path.to_path_buf(), size, true);
    info.category = category;
    info.safety = safety;
    Some(info)
}

/// Recursively sum file sizes under a directory.
fn dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Parse Docker's human-readable size strings like "1.2GB", "345MB".
fn parse_docker_size(s: &str) -> u64 {
    crate::utils::size::parse_size(s).unwrap_or(0)
}
