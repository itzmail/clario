use crate::models::app_info::{AppInfo, PackageManager};
use crate::models::file_info::{FileCategory, FileInfo, SafetyLevel};
use crate::utils::platform::get_app_directories;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

fn dir_leftover(path: &Path) -> Option<FileInfo> {
    if !path.exists() {
        return None;
    }
    let name = path.file_name()?.to_string_lossy().to_string();
    let size = dir_size(path);
    let mut info = FileInfo::new(name, path.to_path_buf(), size, true);
    info.category = FileCategory::AppCache;
    info.safety = SafetyLevel::SafeToDelete;
    Some(info)
}

// ---------------------------------------------------------------------
// Linux: .desktop entries + dpkg/pacman detection
// ---------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::collections::HashMap;

    /// Parse the `[Desktop Entry]` section of a `.desktop` file for the
    /// fields we need. Ignores every other section (e.g. `[Desktop Action ...]`).
    fn parse_desktop_entry(path: &Path) -> Option<HashMap<String, String>> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut fields = HashMap::new();
        let mut in_main_section = false;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_main_section = line == "[Desktop Entry]";
                continue;
            }
            if !in_main_section || line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                fields.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Some(fields)
    }

    pub fn scan_installed_apps() -> Vec<AppInfo> {
        let mut apps = Vec::new();

        for dir in get_app_directories() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }
                let Some(fields) = parse_desktop_entry(&path) else { continue };

                // Skip entries not meant to be shown as launchable apps.
                if fields.get("NoDisplay").map(|v| v == "true").unwrap_or(false)
                    || fields.get("Hidden").map(|v| v == "true").unwrap_or(false)
                {
                    continue;
                }

                let Some(id) = path.file_stem().map(|s| s.to_string_lossy().to_string()) else { continue };
                let name = fields.get("Name").cloned().unwrap_or_else(|| id.clone());
                let package = detect_package_manager(&path);

                apps.push(AppInfo { id, name, source_path: path, package });
            }
        }

        apps
    }

    /// Ask dpkg, then pacman, which package (if any) owns this `.desktop` file.
    /// A `.desktop` file not owned by either is presumed manually installed.
    fn detect_package_manager(desktop_path: &Path) -> PackageManager {
        if let Some(pkg) = dpkg_owner(desktop_path) {
            return PackageManager::Dpkg(pkg);
        }
        if let Some(pkg) = pacman_owner(desktop_path) {
            return PackageManager::Pacman(pkg);
        }
        PackageManager::None
    }

    fn dpkg_owner(path: &Path) -> Option<String> {
        let output = std::process::Command::new("dpkg").arg("-S").arg(path).output().ok()?;
        if !output.status.success() {
            return None;
        }
        // Format: "<package>[,<package>...]: <path>"
        let text = String::from_utf8_lossy(&output.stdout);
        let (pkg, _) = text.split_once(':')?;
        let pkg = pkg.split(',').next()?.trim();
        if pkg.is_empty() { None } else { Some(pkg.to_string()) }
    }

    fn pacman_owner(path: &Path) -> Option<String> {
        let output = std::process::Command::new("pacman").arg("-Qo").arg(path).output().ok()?;
        if !output.status.success() {
            return None;
        }
        // Format: "<path> is owned by <package> <version>"
        let text = String::from_utf8_lossy(&output.stdout);
        let pkg = text.split("is owned by").nth(1)?.trim().split_whitespace().next()?;
        if pkg.is_empty() { None } else { Some(pkg.to_string()) }
    }

    /// Config/cache/data an app may have left behind under XDG dirs, keyed by
    /// its `.desktop` id. Matching is exact-name only — no vendor-prefix or
    /// fuzzy variants, to avoid sweeping up an unrelated app's directory.
    pub fn find_leftovers(app: &AppInfo) -> Vec<FileInfo> {
        let Some(home) = dirs::home_dir() else { return vec![] };
        let candidates = [
            home.join(".config").join(&app.id),
            home.join(".cache").join(&app.id),
            home.join(".local/share").join(&app.id),
        ];

        candidates.iter().filter_map(|p| dir_leftover(p)).collect()
    }
}

// ---------------------------------------------------------------------
// macOS: .app bundles + Homebrew Cask detection
// ---------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    fn plutil_extract(plist: &Path, key: &str) -> Option<String> {
        let output = std::process::Command::new("plutil")
            .args(["-extract", key, "raw", "-o", "-"])
            .arg(plist)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    }

    pub fn scan_installed_apps() -> Vec<AppInfo> {
        let mut apps = Vec::new();

        for dir in get_app_directories() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("app") {
                    continue;
                }

                let file_stem = path.file_stem().map(|s| s.to_string_lossy().to_string());
                let Some(fallback_name) = file_stem else { continue };

                let plist = path.join("Contents/Info.plist");
                let bundle_id = plutil_extract(&plist, "CFBundleIdentifier");
                let display_name = plutil_extract(&plist, "CFBundleDisplayName")
                    .or_else(|| plutil_extract(&plist, "CFBundleName"))
                    .unwrap_or_else(|| fallback_name.clone());

                let id = bundle_id.unwrap_or_else(|| fallback_name.clone());
                let package = get_brew_cask_name(&path)
                    .map(PackageManager::Cask)
                    .unwrap_or(PackageManager::None);

                apps.push(AppInfo { id, name: display_name, source_path: path, package });
            }
        }

        apps
    }

    fn is_homebrew_available() -> bool {
        std::process::Command::new("brew").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
    }

    /// Resolve an app's Homebrew Cask token, if brew-managed.
    ///
    /// ponytail: only the two most reliable Mole stages (resolved-path-in-Caskroom,
    /// then `brew list --cask` name fallback) — skips the Caskroom-`find`-search and
    /// direct-symlink stages Mole also has, since those exist to catch edge cases
    /// (indirect symlinks, ambiguous bundle names) that haven't shown up here yet.
    /// Add them back if a real cask install goes undetected.
    fn get_brew_cask_name(app_path: &Path) -> Option<String> {
        if !is_homebrew_available() {
            return None;
        }

        if let Ok(resolved) = std::fs::canonicalize(app_path) {
            let resolved_str = resolved.to_string_lossy();
            for prefix in ["/opt/homebrew/Caskroom/", "/usr/local/Caskroom/"] {
                if let Some(rest) = resolved_str.strip_prefix(prefix) {
                    if let Some(token) = rest.split('/').next() {
                        if !token.is_empty() {
                            return Some(token.to_string());
                        }
                    }
                }
            }
        }

        let app_name_lower = app_path.file_stem()?.to_string_lossy().to_lowercase();
        let output = std::process::Command::new("brew")
            .env("HOMEBREW_NO_ENV_HINTS", "1")
            .args(["list", "--cask"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|line| line.trim().to_lowercase() == app_name_lower)
            .map(|s| s.trim().to_string())
    }

    /// Config/cache/data an app may have left behind under `~/Library`, keyed
    /// by bundle id and display name. Matching is exact-name only.
    pub fn find_leftovers(app: &AppInfo) -> Vec<FileInfo> {
        let Some(home) = dirs::home_dir() else { return vec![] };
        let library = home.join("Library");
        let candidates: Vec<PathBuf> = vec![
            library.join("Application Support").join(&app.name),
            library.join("Caches").join(&app.id),
            library.join("Logs").join(&app.name),
        ];

        let mut results: Vec<FileInfo> = candidates.iter().filter_map(|p| dir_leftover(p)).collect();

        let prefs = library.join("Preferences").join(format!("{}.plist", app.id));
        if let Ok(meta) = std::fs::metadata(&prefs) {
            let mut info = FileInfo::new(
                prefs.file_name().unwrap().to_string_lossy().to_string(),
                prefs.clone(),
                meta.len(),
                false,
            );
            info.category = FileCategory::AppCache;
            info.safety = SafetyLevel::SafeToDelete;
            results.push(info);
        }

        results
    }
}

#[cfg(target_os = "linux")]
pub use linux::{find_leftovers, scan_installed_apps};

#[cfg(target_os = "macos")]
pub use macos::{find_leftovers, scan_installed_apps};
