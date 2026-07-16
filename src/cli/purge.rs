use crate::core::purge_scanner::{self, PurgeCandidate};
use crate::utils::size::{format_size, parse_size};
use crate::utils::spinner::spin;
use anyhow::Result;
use colored::Colorize;
use dialoguer::MultiSelect;
use std::io::{self, IsTerminal, Write};

const RECENT_THRESHOLD_DAYS: u32 = 7;

pub async fn run_purge(
    min_size: Option<String>,
    force: bool,
    dry_run: bool,
    include_recent: bool,
    show_paths: bool,
) -> Result<()> {
    let search_paths = purge_scanner::load_search_paths();

    if show_paths {
        println!("{}", "Purge search paths".bold());
        for path in &search_paths {
            println!("  {}", path.display());
        }
        println!("\nEdit ~/.config/clario/purge_paths to customize.");
        return Ok(());
    }

    if search_paths.is_empty() {
        println!(
            "No project directories found. Create one of ~/Projects, ~/dev, ~/Code, ~/GitHub, \
             or configure custom paths in ~/.config/clario/purge_paths."
        );
        return Ok(());
    }

    let min_bytes = match min_size {
        Some(ref s) => parse_size(s)?,
        None => 0,
    };

    println!("{}", "Clario Purge".bold());
    println!();

    let candidates = spin("Scanning projects", move || purge_scanner::scan(&search_paths, RECENT_THRESHOLD_DAYS));
    println!("{}", format_size(candidates.iter().map(|c| c.artifact.size_bytes).sum::<u64>()).cyan());

    #[cfg(target_os = "macos")]
    let whitelist = crate::core::protection::load_whitelist();

    let filtered: Vec<PurgeCandidate> = candidates
        .into_iter()
        .filter(|c| c.artifact.size_bytes >= min_bytes)
        .filter(|c| {
            #[cfg(target_os = "macos")]
            {
                use crate::core::protection;
                return protection::is_safe_to_delete(&c.artifact.path) && !protection::is_path_whitelisted(&c.artifact.path, &whitelist);
            }
            #[cfg(not(target_os = "macos"))]
            true
        })
        .collect();

    if filtered.is_empty() {
        println!("\n{}", "Nothing to purge.".green());
        return Ok(());
    }

    println!();
    print_summary(&filtered);

    let candidates: Vec<&PurgeCandidate> = filtered
        .iter()
        .filter(|c| include_recent || !c.is_recent)
        .collect();

    let total_bytes: u64 = candidates.iter().map(|c| c.artifact.size_bytes).sum();

    if total_bytes == 0 {
        println!(
            "\n{}",
            "All candidates are from recently modified projects, nothing to delete. \
             Use --include-recent to purge them anyway."
                .yellow()
        );
        return Ok(());
    }

    if dry_run {
        println!("\n{}", "Dry run — no files deleted.".yellow());
        return Ok(());
    }

    let to_delete: Vec<&PurgeCandidate> = if force {
        candidates
    } else if io::stdout().is_terminal() {
        let labels: Vec<String> = candidates
            .iter()
            .map(|c| format!("{} — {} ({})", c.project_name, c.artifact.name, format_size(c.artifact.size_bytes)))
            .collect();
        let defaults = vec![true; candidates.len()];
        println!();
        let chosen = MultiSelect::new()
            .with_prompt("Select items to delete (space to toggle, enter to confirm)")
            .items(&labels)
            .defaults(&defaults)
            .interact_opt()?;
        match chosen {
            Some(indices) if !indices.is_empty() => {
                indices.into_iter().map(|i| candidates[i]).collect()
            }
            _ => {
                println!("{}", "Aborted.".yellow());
                return Ok(());
            }
        }
    } else {
        print!(
            "\n{} {}",
            format!("Delete {} item(s), freeing {}?", candidates.len(), format_size(total_bytes)).bold(),
            "[y/N] ".bold()
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
        candidates
    };

    let mut freed: u64 = 0;
    for candidate in &to_delete {
        let path = candidate.artifact.path.clone();
        let result = spin(&format!("Removing {}", path.display()), move || std::fs::remove_dir_all(&path));
        match result {
            Ok(_) => {
                freed += candidate.artifact.size_bytes;
                println!("{}", "done".green());
            }
            Err(e) => println!("{} ({})", "failed".red(), e),
        }
    }

    println!("\n{} {}", "Freed:".bold(), format_size(freed).green().bold());
    Ok(())
}

fn print_summary(candidates: &[PurgeCandidate]) {
    let col_w = 20;
    let art_w = 16;
    println!(
        "{:<col_w$} {:<art_w$} {}",
        "Project".bold(),
        "Artifact".bold(),
        "Size".bold(),
    );
    println!("{}", "─".repeat(50));

    for c in candidates {
        let name = if c.is_recent {
            format!("{} [Recent]", c.project_name)
        } else {
            c.project_name.clone()
        };
        println!(
            "{:<col_w$} {:<art_w$} {}",
            name,
            c.artifact.name,
            format_size(c.artifact.size_bytes).cyan()
        );
    }

    println!("{}", "─".repeat(50));
    let total: u64 = candidates.iter().map(|c| c.artifact.size_bytes).sum();
    println!(
        "{:<col_w$} {:<art_w$} {}",
        "Total".bold(),
        candidates.len(),
        format_size(total).green().bold()
    );
}
