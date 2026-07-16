use crate::core::purge_scanner::{self, PurgeCandidate};
use crate::utils::size::{format_size, parse_size};
use crate::utils::spinner::spin;
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

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
    let filtered: Vec<PurgeCandidate> = candidates
        .into_iter()
        .filter(|c| c.artifact.size_bytes >= min_bytes)
        .collect();

    if filtered.is_empty() {
        println!("\n{}", "Nothing to purge.".green());
        return Ok(());
    }

    println!();
    print_summary(&filtered);

    let to_delete: Vec<&PurgeCandidate> = filtered
        .iter()
        .filter(|c| include_recent || !c.is_recent)
        .collect();

    let total_bytes: u64 = to_delete.iter().map(|c| c.artifact.size_bytes).sum();

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

    if !force {
        print!(
            "\n{} {}",
            format!("Delete {} item(s), freeing {}?", to_delete.len(), format_size(total_bytes)).bold(),
            "[y/N] ".bold()
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }

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
