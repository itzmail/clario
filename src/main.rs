mod app;
mod cli;
mod core;
mod handlers;
mod models;
mod ui;
mod utils;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match cli::parse_command(&args) {
        cli::Command::Update { version } => cli::run_update(version).await,
        cli::Command::Tui => {
            let mut app = App::new();
            if let Err(err) = app.run().await {
                eprintln!("Application error: {:?}", err);
                return Err(err);
            }
            Ok(())
        }
    }
}
