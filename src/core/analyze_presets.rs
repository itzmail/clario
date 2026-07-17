use crate::utils::paths::Paths;
use std::path::PathBuf;

/// One entry on the Analyze category screen (Mole-style preset location).
pub struct Preset {
    pub label: &'static str,
    pub path: PathBuf,
}

/// Preset locations for the Analyze category screen. Only paths that exist
/// on this machine are included (e.g. skip JetBrains cache if no JetBrains
/// IDE was ever installed).
pub fn presets() -> Vec<Preset> {
    let Some(home) = dirs::home_dir() else { return Vec::new() };
    let paths = Paths::new();

    let mut candidates: Vec<Preset> = vec![
        Preset { label: "Home", path: home.clone() },
        #[cfg(target_os = "macos")]
        Preset { label: "User Library", path: home.join("Library") },
        #[cfg(target_os = "macos")]
        Preset { label: "Applications", path: PathBuf::from("/Applications") },
        #[cfg(target_os = "macos")]
        Preset { label: "System Library", path: PathBuf::from("/Library") },
        #[cfg(target_os = "macos")]
        Preset { label: "System Logs", path: home.join("Library/Logs") },
        #[cfg(target_os = "macos")]
        Preset { label: "Homebrew Cache", path: home.join("Library/Caches/Homebrew") },
        #[cfg(target_os = "macos")]
        Preset { label: "Xcode Simulators", path: home.join("Library/Developer/CoreSimulator") },
        #[cfg(target_os = "macos")]
        Preset { label: "Xcode Archives", path: home.join("Library/Developer/Xcode/Archives") },
        #[cfg(target_os = "macos")]
        Preset { label: "JetBrains Cache", path: home.join("Library/Caches/JetBrains") },
        #[cfg(target_os = "macos")]
        Preset { label: "Docker Data", path: home.join("Library/Containers/com.docker.docker") },
        #[cfg(target_os = "linux")]
        Preset { label: "User Cache", path: home.join(".cache") },
        #[cfg(target_os = "linux")]
        Preset { label: "Config", path: home.join(".config") },
        #[cfg(target_os = "linux")]
        Preset { label: "Local Share", path: home.join(".local/share") },
        #[cfg(target_os = "linux")]
        Preset { label: "Trash", path: home.join(".local/share/Trash") },
    ];

    if let Some(paths) = paths {
        candidates.push(Preset { label: "Gradle Cache", path: paths.gradle_cache });
    }

    candidates.retain(|p| p.path.exists());
    candidates
}
