pub mod analyze;
pub mod clean;
pub mod menu;
pub mod purge;
pub mod uninstall;
pub mod update;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "clario",
    about = "System cleaning utility",
    long_about = "Clario - Developer cache & system cleaner\n\nSubcommands:\n clean Clean developer caches, build artifacts, logs, and more\n  update   Check and install the latest version\n\nTips:\n  Run 'clario clean --help' for full list of clean targets\n  Use '--dry-run' to preview what would be deleted\n Use '--force' to skip confirmation prompt"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
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
    /// Clean build artifacts (node_modules, target, dist, etc.) across projects
    Purge {
        /// Only show items larger than this threshold (e.g., 100MB, 1GB)
        #[arg(long)]
        min_size: Option<String>,

        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,

        /// Show what would be purged without deleting
        #[arg(long)]
        dry_run: bool,

        /// Also purge artifacts belonging to recently modified projects
        #[arg(long)]
        include_recent: bool,

        /// Show configured search paths and exit
        #[arg(long)]
        paths: bool,
    },
    /// Uninstall an application and its leftover files
    Uninstall {
        /// Name of the application to uninstall
        name: Option<String>,

        /// List installed applications and exit
        #[arg(long)]
        list: bool,

        /// Show what would be removed without deleting
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,
    },
    /// Show a size breakdown of a directory (defaults to home)
    Analyze {
        /// Directory to analyze (defaults to home directory)
        path: Option<PathBuf>,
    },
}
