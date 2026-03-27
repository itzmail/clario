pub mod clean;
pub mod update;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clario", about = "System cleaning utility")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Check for updates and install the latest version
    Update {
        /// Specific version to install (e.g., v0.2.0)
        version: Option<String>,
    },
    /// Clean developer caches and build artifacts
    Clean {
        #[command(subcommand)]
        category: Option<clean::CleanCategory>,

        /// Only show items larger than this threshold (e.g., 100MB, 1GB)
        #[arg(long, global = true)]
        min_size: Option<String>,

        /// Skip confirmation prompt
        #[arg(long, short, global = true)]
        force: bool,

        /// Show what would be cleaned without deleting
        #[arg(long, global = true)]
        dry_run: bool,
    },
}
