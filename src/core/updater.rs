use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::sync::mpsc;

const REPO: &str = "itzmail/clario";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[allow(dead_code)]
    pub prerelease: bool,
    pub body: Option<String>,
    pub published_at: Option<String>,
}

impl Release {
    pub fn version(&self) -> &str {
        self.tag_name.trim_start_matches('v')
    }

    pub fn is_current(&self) -> bool {
        self.version() == CURRENT_VERSION
    }

    pub fn is_newer_than_current(&self) -> bool {
        is_newer(self.version(), CURRENT_VERSION)
    }
}

fn is_newer(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> (u64, u64, u64) {
        let parts: Vec<u64> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(a) > parse(b)
}

fn target_triple() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "aarch64-unknown-linux-musl"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-musl"
    } else {
        "x86_64-apple-darwin"
    }
}

fn download_url(tag: &str) -> String {
    format!(
        "https://github.com/{}/releases/download/{}/clario-{}.tar.gz",
        REPO,
        tag,
        target_triple()
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateState {
    Idle,
    Checking,
    Loaded,
    Downloading,
    Done,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum UpdateEvent {
    ReleasesLoaded(Vec<Release>),
    Progress(String),
    Done,
    Error(String),
}

pub async fn fetch_releases() -> Result<Vec<Release>> {
    let url = format!("https://api.github.com/repos/{}/releases", REPO);
    let client = reqwest::Client::builder()
        .user_agent(format!("clario/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let releases: Vec<Release> = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(releases)
}

pub async fn download_and_install(tag: &str, tx: mpsc::Sender<UpdateEvent>) -> Result<()> {
    let url = download_url(tag);
    let _ = tx.send(UpdateEvent::Progress(format!("Connecting to GitHub...")));

    let client = reqwest::Client::builder()
        .user_agent(format!("clario/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let _ = tx.send(UpdateEvent::Progress(format!("Downloading {}...", tag)));
    let data = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let _ = tx.send(UpdateEvent::Progress("Extracting binary...".to_string()));

    let cursor = std::io::Cursor::new(data.as_ref());
    let gz = flate2::read::GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(gz);

    let install_path = std::env::current_exe()
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".local/bin/clario"));
    let tmp_path = install_path.with_extension("new");

    let mut extracted = false;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.to_path_buf();
        if entry_path
            .file_name()
            .map(|n| n == "clario")
            .unwrap_or(false)
        {
            entry.unpack(&tmp_path)?;
            extracted = true;
            break;
        }
    }

    if !extracted {
        return Err(anyhow!(
            "Binary 'clario' not found in archive. Unsupported platform?"
        ));
    }

    let _ = tx.send(UpdateEvent::Progress("Installing...".to_string()));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    std::fs::rename(&tmp_path, &install_path)?;

    let _ = tx.send(UpdateEvent::Done);
    Ok(())
}
