use crate::models::file_info::{FileCategory, FileInfo, SafetyLevel};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

/// Project build artifact directory names, purge candidates across any language.
const PURGE_TARGETS: &[&str] = &[
    "node_modules",
    "target", // Rust, Maven
    "dist",   // JS builds
    ".venv",
    "venv",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".gradle",
    ".next",
    ".nuxt",
    "vendor", // PHP Composer
    ".turbo",
    ".parcel-cache",
];

/// Files/dirs whose presence marks a directory as a project root.
const PROJECT_INDICATORS: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "requirements.txt",
    "pom.xml",
    "build.gradle",
    "Gemfile",
    "composer.json",
    ".git",
];

/// A project directory below can be excluded from deletion because it was touched recently.
pub struct PurgeCandidate {
    pub artifact: FileInfo,
    pub project_name: String,
    pub is_recent: bool,
}

fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("clario/purge_paths"))
}

/// Default directories to scan for projects when no config exists.
fn default_search_paths() -> Vec<PathBuf> {
    let Some(home) = dirs::home_dir() else { return vec![] };
    [
        "dev",
        "Projects",
        "GitHub",
        "Code",
        "Workspace",
        "Repos",
        "Development",
        "www",
    ]
    .iter()
    .map(|d| home.join(d))
    .filter(|p| p.exists())
    .collect()
}

/// Load search paths from `~/.config/clario/purge_paths`, one per line.
/// Falls back to (and saves) auto-discovered default directories on first run.
pub fn load_search_paths() -> Vec<PathBuf> {
    if let Some(config) = config_path() {
        if let Ok(content) = std::fs::read_to_string(&config) {
            let paths: Vec<PathBuf> = content
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(PathBuf::from)
                .collect();
            if !paths.is_empty() {
                return paths;
            }
        }

        let discovered = default_search_paths();
        if !discovered.is_empty() {
            let _ = save_search_paths(&config, &discovered);
        }
        return discovered;
    }

    default_search_paths()
}

fn save_search_paths(config: &Path, paths: &[PathBuf]) -> std::io::Result<()> {
    if let Some(parent) = config.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut content = String::from(
        "# Clario Purge Paths - Auto-discovered project directories\n# Edit this file to customize, or run: clario purge --paths\n",
    );
    for path in paths {
        content.push_str(&path.to_string_lossy());
        content.push('\n');
    }
    std::fs::write(config, content)
}

fn is_project_root(dir: &Path) -> bool {
    PROJECT_INDICATORS.iter().any(|marker| dir.join(marker).exists())
}

/// Recently modified projects are excluded from deletion by default (matches Mole's
/// "unselected if recent" default) — someone actively working in a project shouldn't
/// have their node_modules/target vanish mid-session.
fn is_recent(dir: &Path, threshold_days: u32) -> bool {
    let Ok(meta) = std::fs::metadata(dir) else { return false };
    let Ok(modified) = meta.modified() else { return false };
    let Ok(age) = SystemTime::now().duration_since(modified) else { return false };
    age.as_secs() < threshold_days as u64 * 86400
}

fn dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Walk each search path up to a bounded depth, find project roots, and within each
/// project find purge-target artifact directories (node_modules, target, etc).
pub fn scan(search_paths: &[PathBuf], recent_threshold_days: u32) -> Vec<PurgeCandidate> {
    let mut results = Vec::new();

    for root in search_paths {
        if !root.exists() {
            continue;
        }

        // Depth 4 below a search root is enough to find most project layouts
        // (e.g. ~/Projects/org/repo) without walking into node_modules internals.
        let walker = WalkDir::new(root)
            .max_depth(4)
            .into_iter()
            .filter_entry(|e| e.path() == root || !is_purge_target(e.file_name().to_string_lossy().as_ref()));

        for entry in walker.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() || !is_project_root(path) {
                continue;
            }

            let project_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let recent = is_recent(path, recent_threshold_days);

            for target in PURGE_TARGETS {
                let candidate_path = path.join(target);
                if !candidate_path.exists() {
                    continue;
                }
                let size = dir_size(&candidate_path);
                if size == 0 {
                    continue;
                }
                let mut info = FileInfo::new(target.to_string(), candidate_path, size, true);
                info.category = FileCategory::Other;
                info.safety = SafetyLevel::SafeToDelete;
                results.push(PurgeCandidate {
                    artifact: info,
                    project_name: project_name.clone(),
                    is_recent: recent,
                });
            }
        }
    }

    results.sort_by(|a, b| b.artifact.size_bytes.cmp(&a.artifact.size_bytes));
    results
}

fn is_purge_target(name: &str) -> bool {
    PURGE_TARGETS.contains(&name)
}
