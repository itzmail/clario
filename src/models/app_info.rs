use std::path::PathBuf;

/// How this app is tracked by the system, if at all. Determines the uninstall
/// path: a package-managed app must go through its manager (brew/apt/pacman)
/// so hooks like Homebrew's cask `zap` stanza run; anything else falls back
/// to deleting the app entry plus best-effort leftover files.
#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    /// Homebrew Cask, value is the cask token (e.g. "spotify").
    Cask(String),
    /// Debian/Ubuntu dpkg, value is the package name.
    Dpkg(String),
    /// Arch pacman, value is the package name.
    Pacman(String),
    /// Not tracked by any package manager (manual install, AppImage, etc).
    None,
}

/// A discovered installed application, macOS `.app` bundle or Linux `.desktop` entry.
#[derive(Debug, Clone)]
pub struct AppInfo {
    /// Stable identifier: `CFBundleIdentifier` on macOS, `.desktop` filename
    /// (without extension) on Linux. Used to search for leftover files.
    pub id: String,
    /// Human-readable name shown in previews and matched against user input.
    pub name: String,
    /// Path to the `.app` bundle or `.desktop` file.
    pub source_path: PathBuf,
    pub package: PackageManager,
}
