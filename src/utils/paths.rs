use std::path::PathBuf;

/// Centralized global path definitions for all scan targets.
/// Constructed once from the user's home directory — no scattered hardcoded strings.
pub struct Paths {
    // Cargo
    pub cargo_registry_cache: PathBuf,
    pub cargo_registry_src: PathBuf,

    // Node
    pub npm_cache: PathBuf,
    pub pnpm_stores: Vec<PathBuf>,

    // Go
    pub go_mod_cache: PathBuf,
    pub go_build_caches: Vec<PathBuf>,

    // Python
    pub pip_caches: Vec<PathBuf>,

    // Java
    pub gradle_cache: PathBuf,
    pub maven_repo: PathBuf,

    // Ruby
    pub gem_dir: PathBuf,

    // System (macOS)
    #[cfg(target_os = "macos")]
    pub user_caches: PathBuf,
    #[cfg(target_os = "macos")]
    #[allow(dead_code)]
    pub user_logs: PathBuf,
    #[cfg(target_os = "macos")]
    pub system_caches: PathBuf,

    // System (Linux)
    #[cfg(target_os = "linux")]
    pub user_cache: PathBuf,
    #[cfg(target_os = "linux")]
    pub system_logs: PathBuf,
}

impl Paths {
    /// Build paths from the current user's home directory.
    /// Returns None if home directory cannot be determined.
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;

        Some(Self {
            cargo_registry_cache: home.join(".cargo/registry/cache"),
            cargo_registry_src: home.join(".cargo/registry/src"),

            npm_cache: home.join(".npm/_cacache"),
            pnpm_stores: vec![
                home.join(".pnpm-store"),
                home.join(".local/share/pnpm/store"),
            ],

            go_mod_cache: home.join("go/pkg/mod"),
            go_build_caches: vec![
                home.join("Library/Caches/go-build"),
                home.join(".cache/go-build"),
            ],

            pip_caches: vec![
                home.join("Library/Caches/pip"),
                home.join(".cache/pip"),
            ],

            gradle_cache: home.join(".gradle/caches"),
            maven_repo: home.join(".m2/repository"),

            gem_dir: home.join(".gem"),

            #[cfg(target_os = "macos")]
            user_caches: home.join("Library/Caches"),
            #[cfg(target_os = "macos")]
            user_logs: home.join("Library/Logs"),
            #[cfg(target_os = "macos")]
            system_caches: PathBuf::from("/Library/Caches"),

            #[cfg(target_os = "linux")]
            user_cache: home.join(".cache"),
            #[cfg(target_os = "linux")]
            system_logs: PathBuf::from("/var/log"),
        })
    }

    /// Returns system-level cache paths for the current OS.
    pub fn system_cache_dirs(&self) -> Vec<&PathBuf> {
        #[cfg(target_os = "macos")]
        return vec![&self.user_caches, &self.system_caches];

        #[cfg(target_os = "linux")]
        return vec![&self.user_cache];

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        return vec![];
    }
}
