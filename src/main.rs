mod app;
mod cli;
mod core;
mod handlers;
mod models;
mod ui;
mod utils;

use anyhow::Result;
use app::App;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Update { version }) => cli::update::run_update(version).await,
        Some(Command::Clean {
            category,
            min_size,
            force,
            dry_run,
        }) => cli::clean::run_clean(category, min_size, force, dry_run).await,
        None => {
            let mut app = App::new();
            if let Err(err) = app.run().await {
                eprintln!("Application error: {:?}", err);
                return Err(err);
            }
            Ok(())
        }
    }
}
