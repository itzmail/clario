use crate::core::app_scanner;
use crate::models::app_info::{AppInfo, PackageManager};
use crate::models::file_info::FileInfo;
use crate::utils::size::format_size;
use crate::utils::spinner::spin;
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::io::{self, Write};

pub async fn run_uninstall(name: Option<String>, list: bool, dry_run: bool, force: bool) -> Result<()> {
    let apps = spin("Scanning installed applications", app_scanner::scan_installed_apps);

    if list {
        print_app_list(&apps);
        return Ok(());
    }

    let Some(name) = name else {
        return Err(anyhow!("specify an app name or use --list"));
    };

    let matches = match_apps_by_name(&apps, &name);
    let app = match matches.as_slice() {
        [] => return Err(anyhow!("no application found matching '{}'", name)),
        [single] => single,
        multiple => {
            println!("{}", "Multiple applications match, be more specific:".yellow());
            for app in multiple {
                println!("  {}", app.name);
            }
            return Ok(());
        }
    };

    println!("{}", "Clario Uninstall".bold());
    println!();
    println!("{} {}", "App:".bold(), app.name);

    let action = plan_action(app);
    print_plan(&action);

    if dry_run {
        println!("\n{}", "Dry run — nothing removed.".yellow());
        return Ok(());
    }

    if !force {
        print!("\n{}", "Proceed? [y/N] ".bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }

    execute_action(action)
}

fn print_app_list(apps: &[AppInfo]) {
    if apps.is_empty() {
        println!("{}", "No applications found.".dimmed());
        return;
    }
    for app in apps {
        let source = match &app.package {
            PackageManager::Cask(token) => format!("brew cask: {}", token),
            PackageManager::Dpkg(pkg) => format!("dpkg: {}", pkg),
            PackageManager::Pacman(pkg) => format!("pacman: {}", pkg),
            PackageManager::None => "manual".to_string(),
        };
        println!("{:<40} {}", app.name, source.dimmed());
    }
}

/// Case-insensitive exact match first, falling back to substring match —
/// mirrors Mole's `match_apps_by_name`.
fn match_apps_by_name<'a>(apps: &'a [AppInfo], search: &str) -> Vec<&'a AppInfo> {
    let search_lower = search.to_lowercase();

    let exact: Vec<&AppInfo> = apps.iter().filter(|a| a.name.to_lowercase() == search_lower).collect();
    if !exact.is_empty() {
        return exact;
    }

    apps.iter().filter(|a| a.name.to_lowercase().contains(&search_lower)).collect()
}

enum UninstallAction<'a> {
    Cask { token: String },
    Dpkg { pkg: String },
    Pacman { pkg: String },
    Manual { app: &'a AppInfo, leftovers: Vec<FileInfo> },
}

fn plan_action(app: &AppInfo) -> UninstallAction<'_> {
    match &app.package {
        PackageManager::Cask(token) => UninstallAction::Cask { token: token.clone() },
        PackageManager::Dpkg(pkg) => UninstallAction::Dpkg { pkg: pkg.clone() },
        PackageManager::Pacman(pkg) => UninstallAction::Pacman { pkg: pkg.clone() },
        PackageManager::None => {
            let leftovers = app_scanner::find_leftovers(app);
            UninstallAction::Manual { app, leftovers }
        }
    }
}

fn print_plan(action: &UninstallAction) {
    match action {
        UninstallAction::Cask { token } => {
            println!("{} brew uninstall --cask --zap {}", "Will run:".bold(), token);
        }
        UninstallAction::Dpkg { pkg } => {
            println!("{} sudo apt remove {}", "Will run:".bold(), pkg);
        }
        UninstallAction::Pacman { pkg } => {
            println!("{} sudo pacman -R {}", "Will run:".bold(), pkg);
        }
        UninstallAction::Manual { app, leftovers } => {
            println!("{} {}", "Will remove:".bold(), app.source_path.display());
            if leftovers.is_empty() {
                println!("  {}", "no leftover files found".dimmed());
            } else {
                for item in leftovers {
                    println!("  {} {}", item.path.display(), format_size(item.size_bytes).cyan());
                }
                let total: u64 = leftovers.iter().map(|f| f.size_bytes).sum();
                println!("{} {}", "Total:".bold(), format_size(total).cyan());
            }
        }
    }
}

fn execute_action(action: UninstallAction) -> Result<()> {
    match action {
        UninstallAction::Cask { token } => {
            let status = spin(&format!("Running brew uninstall --cask --zap {}", token), move || {
                std::process::Command::new("brew").args(["uninstall", "--cask", "--zap", &token]).status()
            });
            report_status(status)
        }
        UninstallAction::Dpkg { pkg } => {
            let status = spin(&format!("Running sudo apt remove {}", pkg), move || {
                std::process::Command::new("sudo").args(["apt", "remove", "-y", &pkg]).status()
            });
            report_status(status)
        }
        UninstallAction::Pacman { pkg } => {
            let status = spin(&format!("Running sudo pacman -R {}", pkg), move || {
                std::process::Command::new("sudo").args(["pacman", "-R", "--noconfirm", &pkg]).status()
            });
            report_status(status)
        }
        UninstallAction::Manual { app, leftovers } => {
            let app_path = app.source_path.clone();
            let mut freed: u64 = 0;

            let result = spin(&format!("Removing {}", app_path.display()), move || trash::delete(&app_path));
            if result.is_ok() {
                println!("{}", "done".green());
            } else {
                println!("{}", "failed".red());
            }

            for item in &leftovers {
                let path = item.path.clone();
                let result = spin(&format!("Removing {}", path.display()), move || trash::delete(&path));
                match result {
                    Ok(_) => {
                        freed += item.size_bytes;
                        println!("{}", "done".green());
                    }
                    Err(e) => println!("{} ({})", "failed".red(), e),
                }
            }

            println!("\n{} {}", "Freed:".bold(), format_size(freed).green().bold());
            Ok(())
        }
    }
}

fn report_status(status: std::io::Result<std::process::ExitStatus>) -> Result<()> {
    match status {
        Ok(s) if s.success() => {
            println!("{}", "done".green());
            Ok(())
        }
        Ok(s) => {
            println!("{}", "failed".red());
            Err(anyhow!("command exited with status {}", s))
        }
        Err(e) => {
            println!("{}", "failed".red());
            Err(e.into())
        }
    }
}
