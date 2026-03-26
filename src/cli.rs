use crate::core::updater::{self, UpdateEvent, CURRENT_VERSION};
use anyhow::Result;
use std::sync::mpsc;

pub enum Command {
    Update { version: Option<String> },
    Tui,
}

pub fn parse_command(args: &[String]) -> Command {
    match args.get(1).map(String::as_str) {
        Some("update") => Command::Update {
            version: args.get(2).cloned(),
        },
        _ => Command::Tui,
    }
}

pub async fn run_update(version: Option<String>) -> Result<()> {
    println!("clario v{}\n", CURRENT_VERSION);

    // Normalize to ensure "v" prefix (accept both "v0.1.0" and "0.1.0")
    let requested_tag = version.map(|v| {
        if v.starts_with('v') {
            v
        } else {
            format!("v{}", v)
        }
    });

    println!("Checking for updates...");
    let releases = updater::fetch_releases().await?;

    if releases.is_empty() {
        eprintln!("No releases found on GitHub.");
        std::process::exit(1);
    }
    println!("  → Found {} release(s)\n", releases.len());

    let target = if let Some(ref tag) = requested_tag {
        match releases.iter().find(|r| &r.tag_name == tag) {
            Some(r) => {
                println!("Installing specific version: {}\n", r.tag_name);
                r.clone()
            }
            None => {
                eprintln!("Error: version '{}' not found in GitHub releases.", tag);
                eprintln!("Available versions:");
                for r in releases.iter().take(10) {
                    eprintln!("  {}", r.tag_name);
                }
                std::process::exit(1);
            }
        }
    } else {
        // No version specified — install latest
        let latest = releases.first().unwrap(); // safe: checked non-empty above
        let date = latest
            .published_at
            .as_deref()
            .and_then(|d| d.get(..10))
            .unwrap_or("unknown");
        println!("Latest: {} (released {})", latest.tag_name, date);

        if latest.is_current() {
            println!("Already on the latest version (v{}).", CURRENT_VERSION);
            return Ok(());
        }
        if !latest.is_newer_than_current() {
            println!(
                "Current build v{} is newer than the latest release. Nothing to do.",
                CURRENT_VERSION
            );
            return Ok(());
        }

        println!("  Current version is older. Installing...\n");
        latest.clone()
    };

    let (tx, rx) = mpsc::channel::<UpdateEvent>();
    let tag = target.tag_name.clone();

    tokio::spawn(async move {
        if let Err(e) = updater::download_and_install(&tag, tx.clone()).await {
            let _ = tx.send(UpdateEvent::Error(e.to_string()));
        }
    });

    loop {
        match rx.recv() {
            Ok(UpdateEvent::Progress(msg)) => println!("  {}", msg),
            Ok(UpdateEvent::Done) => {
                println!("\nDone! Restart clario to use {}.", target.tag_name);
                return Ok(());
            }
            Ok(UpdateEvent::Error(err)) => {
                eprintln!("\nError: {}", err);
                std::process::exit(1);
            }
            Ok(UpdateEvent::ReleasesLoaded(_)) => {}
            Err(_) => {
                eprintln!("Update process ended unexpectedly.");
                std::process::exit(1);
            }
        }
    }
}
