use crate::models::file_info::{FileCategory, FileInfo, SafetyLevel};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Returns true if the current working directory contains any of the given marker files.
fn cwd_has_marker(markers: &[&str]) -> bool {
    let Ok(cwd) = std::env::current_dir() else { return false };
    markers.iter().any(|m| cwd.join(m).exists())
}

/// Returns the current working directory, or None if unavailable.
fn cwd() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

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

/// Scan Cargo cache and project target/ directories.
/// Returns a flat list of FileInfo entries, each representing a directory.
pub fn scan_cargo() -> Vec<FileInfo> {
    let mut results = Vec::new();

    // Global: ~/.cargo/registry/cache and src
    if let Some(home) = dirs::home_dir() {
        for subdir in &["registry/cache", "registry/src"] {
            let path = home.join(".cargo").join(subdir);
            if path.exists() {
                if let Some(info) = dir_info(&path, FileCategory::CargoCache, SafetyLevel::SafeToDelete) {
                    results.push(info);
                }
            }
        }
    }

    // Local: if CWD is a Cargo project, scan ./target/
    if cwd_has_marker(&["Cargo.toml"]) {
        if let Some(cwd) = cwd() {
            let target = cwd.join("target");
            if target.exists() {
                if let Some(info) = dir_info(&target, FileCategory::CargoBuild, SafetyLevel::SafeToDelete) {
                    results.push(info);
                }
            }
        }
    }

    results
}

/// Scan Node.js caches and project node_modules/ directories.
pub fn scan_node() -> Vec<FileInfo> {
    let mut results = Vec::new();

    // Global: npm and pnpm caches
    if let Some(home) = dirs::home_dir() {
        let npm_cache = home.join(".npm/_cacache");
        if npm_cache.exists() {
            if let Some(info) = dir_info(&npm_cache, FileCategory::NodeCache, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }

        for pnpm_path in &[
            home.join(".pnpm-store"),
            home.join(".local/share/pnpm/store"),
        ] {
            if pnpm_path.exists() {
                if let Some(info) = dir_info(pnpm_path, FileCategory::NodeCache, SafetyLevel::SafeToDelete) {
                    results.push(info);
                }
            }
        }
    }

    // Local: if CWD is a Node project, scan ./node_modules/
    if cwd_has_marker(&["package.json"]) {
        if let Some(cwd) = cwd() {
            let node_modules = cwd.join("node_modules");
            if node_modules.exists() {
                if let Some(info) = dir_info(&node_modules, FileCategory::NodeModules, SafetyLevel::SafeToDelete) {
                    results.push(info);
                }
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

    if let Some(home) = dirs::home_dir() {
        // Go module cache
        let mod_cache = home.join("go/pkg/mod");
        if mod_cache.exists() {
            if let Some(info) = dir_info(&mod_cache, FileCategory::GoCache, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }

        // Go build cache (~/.cache/go-build on Linux, ~/Library/Caches/go-build on macOS)
        for go_build in &[
            home.join("Library/Caches/go-build"),
            home.join(".cache/go-build"),
        ] {
            if go_build.exists() {
                if let Some(info) = dir_info(go_build, FileCategory::GoBuild, SafetyLevel::SafeToDelete) {
                    results.push(info);
                }
            }
        }
    }

    results
}

/// Scan Python pip cache and project __pycache__ / venv directories.
pub fn scan_python() -> Vec<FileInfo> {
    let mut results = Vec::new();

    // Global: pip cache
    if let Some(home) = dirs::home_dir() {
        for pip_cache in &[
            home.join("Library/Caches/pip"),
            home.join(".cache/pip"),
        ] {
            if pip_cache.exists() {
                if let Some(info) = dir_info(pip_cache, FileCategory::PythonCache, SafetyLevel::SafeToDelete) {
                    results.push(info);
                }
            }
        }
    }

    // Local: if CWD is a Python project, scan __pycache__ and venv dirs
    if cwd_has_marker(&["requirements.txt", "pyproject.toml", "setup.py", "setup.cfg"]) {
        if let Some(cwd) = cwd() {
            find_named_dirs(&cwd.clone().into(), "__pycache__", 5, FileCategory::PythonCache, SafetyLevel::SafeToDelete, &mut results);
            for venv_name in &[".venv", "venv", "env"] {
                let venv_path = cwd.join(venv_name);
                if venv_path.exists() {
                    if let Some(info) = dir_info(&venv_path, FileCategory::PythonVenv, SafetyLevel::ProceedWithCaution) {
                        results.push(info);
                    }
                }
            }
        }
    }

    results
}

/// Scan Java Gradle and Maven caches.
pub fn scan_java() -> Vec<FileInfo> {
    let mut results = Vec::new();

    // Global: ~/.gradle/caches and ~/.m2/repository
    if let Some(home) = dirs::home_dir() {
        let gradle_cache = home.join(".gradle/caches");
        if gradle_cache.exists() {
            if let Some(info) = dir_info(&gradle_cache, FileCategory::JavaGradle, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }

        let maven_repo = home.join(".m2/repository");
        if maven_repo.exists() {
            if let Some(info) = dir_info(&maven_repo, FileCategory::JavaMaven, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }
    }

    // Local: if CWD is a Gradle project, scan local .gradle/
    if cwd_has_marker(&["build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts"]) {
        if let Some(cwd) = cwd() {
            let local_gradle = cwd.join(".gradle");
            if local_gradle.exists() {
                if let Some(info) = dir_info(&local_gradle, FileCategory::JavaGradle, SafetyLevel::SafeToDelete) {
                    results.push(info);
                }
            }
        }
    }

    results
}

/// Scan Ruby gems cache.
pub fn scan_ruby() -> Vec<FileInfo> {
    let mut results = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let gem_dir = home.join(".gem");
        if gem_dir.exists() {
            if let Some(info) = dir_info(&gem_dir, FileCategory::RubyGems, SafetyLevel::SafeToDelete) {
                results.push(info);
            }
        }
    }

    results
}

/// Scan system log directories (delegates to platform scan targets filtered to log paths).
pub fn scan_logs() -> Vec<FileInfo> {
    use crate::utils::platform::get_scan_targets;
    get_scan_targets()
        .into_iter()
        .filter(|p| {
            p.to_string_lossy().to_lowercase().contains("log")
        })
        .filter(|p| p.exists())
        .filter_map(|p| dir_info(&p, FileCategory::Log, SafetyLevel::SafeToDelete))
        .collect()
}

/// Scan system cache directories (delegates to platform scan targets filtered to cache paths).
pub fn scan_cache() -> Vec<FileInfo> {
    use crate::utils::platform::get_scan_targets;
    get_scan_targets()
        .into_iter()
        .filter(|p| {
            p.to_string_lossy().to_lowercase().contains("cache")
        })
        .filter(|p| p.exists())
        .filter_map(|p| dir_info(&p, FileCategory::Cache, SafetyLevel::SafeToDelete))
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

/// Walk `root` up to `max_depth` levels and collect directories named `target_name`.
/// Skips descending into matched directories (avoids scanning inside node_modules/node_modules).
fn find_named_dirs(
    root: &PathBuf,
    target_name: &str,
    max_depth: usize,
    category: FileCategory,
    safety: SafetyLevel,
    results: &mut Vec<FileInfo>,
) {
    // filter_entry skips descending into matched dirs (avoids scanning inside node_modules/node_modules)
    let walker = WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0 || e.file_name().to_string_lossy() != target_name
        });

    for entry in walker.filter_map(Result::ok) {
        if entry.depth() == 0 {
            continue;
        }
        if entry.file_name().to_string_lossy() == target_name && entry.path().is_dir() {
            if let Some(info) = dir_info(entry.path(), category.clone(), safety.clone()) {
                results.push(info);
            }
        }
    }
}

/// Parse Docker's human-readable size strings like "1.2GB", "345MB".
fn parse_docker_size(s: &str) -> u64 {
    // Docker uses formats like "1.234GB", "345MB", "0B"
    crate::utils::size::parse_size(s).unwrap_or(0)
}
