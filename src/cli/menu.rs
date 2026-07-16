use crate::cli::{clean, purge, uninstall, update};
use anyhow::Result;
use colored::Colorize;
use dialoguer::Select;

const TAGLINE: &str = "Fast dev cache & artifact cleaner.";

fn show_banner() {
    println!(
        "{}",
        r"
          ________           _
         / ____/ /___ ______(_)___
        / /   / / __ `/ ___/ / __ \
       / /___/ / /_/ / /  / / /_/ /
       \____/_/\__,_/_/  /_/\____/
                            "
            .green()
    );
    println!("{}", "https://github.com/itzmail/clario".blue());
    println!("{}", TAGLINE.green());
    println!();
}

/// Interactive main menu shown when Clario is run with no arguments (Mole-style entry point).
/// Returns Ok(()) after the chosen action completes, or immediately if the user quits.
pub async fn run_main_menu() -> Result<()> {
    show_banner();

    let options = ["Clean        Clean developer caches and build artifacts", "Purge        Clean build artifacts across all projects", "Uninstall    Remove an application and its leftover files", "Update       Check and install the latest version", "Quit"];

    let choice = Select::new()
        .with_prompt("Select an action")
        .items(&options)
        .default(0)
        .interact_opt()?;

    match choice {
        Some(0) => clean::run_clean(None, None, false, false).await,
        Some(1) => purge::run_purge(None, false, false, false, false).await,
        Some(2) => uninstall::run_uninstall(None, false, false, false).await,
        Some(3) => update::run_update(None).await,
        _ => Ok(()),
    }
}
