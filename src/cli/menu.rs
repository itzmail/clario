use crate::cli::{analyze, clean, purge, uninstall, update};
use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};

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
    dialoguer::console::Term::stdout().clear_screen().ok();
    show_banner();

    let footer = format!("\n{}", "↑↓  |  Enter  |  Q Quit".dimmed());
    let options = [
        "1. Clean        Clean developer caches and build artifacts".to_string(),
        "2. Purge        Clean build artifacts across all projects".to_string(),
        "3. Uninstall    Remove an application and its leftover files".to_string(),
        "4. Analyze      Explore disk usage".to_string(),
        "5. Update       Check and install the latest version".to_string(),
        format!("6. Quit{footer}"),
    ];

    let theme = ColorfulTheme {
        active_item_prefix: dialoguer::console::Style::new().green().apply_to("➤ ".to_string()),
        ..ColorfulTheme::default()
    };

    let choice = Select::with_theme(&theme)
        .items(&options)
        .default(0)
        .interact_opt()?;

    match choice {
        Some(0) => clean::run_clean(None, None, false, false).await,
        Some(1) => purge::run_purge(None, false, false, false, false).await,
        Some(2) => uninstall::run_uninstall(None, false, false, false).await,
        Some(3) => analyze::run_analyze(None).await,
        Some(4) => update::run_update(None).await,
        _ => Ok(()),
    }
}
