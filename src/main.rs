mod cli;
mod core;
mod models;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Update { version } => cli::update::run_update(version).await,
        Command::Clean {
            category,
            min_size,
            force,
            dry_run,
        } => cli::clean::run_clean(category, min_size, force, dry_run).await,
        Command::Purge {
            min_size,
            force,
            dry_run,
            include_recent,
            paths,
        } => cli::purge::run_purge(min_size, force, dry_run, include_recent, paths).await,
    }
}
