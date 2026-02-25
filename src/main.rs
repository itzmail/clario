mod app;
mod core;
mod models;
mod ui;
mod utils;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new();
    if let Err(err) = app.run().await {
        eprintln!("Application error: {:?}", err);
        return Err(err);
    }
    Ok(())
}
