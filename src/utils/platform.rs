use std::path::PathBuf;

/// Mendapatkan daftar target direktori yang wajib di-scan oleh Clario.
/// Implementasi berbeda-beda tergantung OS (macOS / Linux / Windows).
pub fn get_scan_targets() -> Vec<PathBuf> {
    let mut targets = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // --- 🍎 MACOS (Darwin) SPECIFIC ---
        #[cfg(target_os = "macos")]
        {
            targets.push(home.join("Library/Caches"));
            targets.push(home.join("Library/Logs"));
            targets.push(PathBuf::from("/Library/Caches"));
        }

        // --- 🐧 LINUX SPECIFIC ---
        #[cfg(target_os = "linux")]
        {
            targets.push(home.join(".cache"));
            targets.push(home.join(".local/share/Trash"));
            targets.push(PathBuf::from("/var/log"));
        }

        // --- 🪟 WINDOWS SPECIFIC ---
        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = dirs::data_local_dir() {
                targets.push(appdata.join("Temp"));
            }
            targets.push(PathBuf::from("C:\\Windows\\Temp"));
        }
    }

    targets
}

/// Mendapatkan folder di mana aplikasi biasanya terinstall
#[allow(dead_code)] // Reserved: akan dipakai Phase 3 untuk LaunchAgents/app dirs audit
pub fn get_app_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/Applications"));
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Applications"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        dirs.push(PathBuf::from("/usr/share/applications"));
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/share/applications"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        dirs.push(PathBuf::from("C:\\Program Files"));
        dirs.push(PathBuf::from("C:\\Program Files (x86)"));
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("AppData\\Local\\Programs"));
        }
    }

    dirs
}
