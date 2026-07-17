use crate::core::{dev_scanner, purge_scanner};
use crate::models::file_info::{FileCategory, FileInfo, SafetyLevel};
use crate::utils::size::{format_size, parse_size};
use crate::utils::spinner::spin;
use anyhow::Result;
use colored::Colorize;
use dialoguer::MultiSelect;
use std::io::{self, IsTerminal, Write};

/// Recently modified projects are excluded from `clean`'s project-artifact scan by
/// default, matching `purge`'s safety guard (someone actively working in a project
/// shouldn't have node_modules/target vanish mid-session).
const RECENT_THRESHOLD_DAYS: u32 = 7;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum CleanCategory {
    /// Clean Cargo build artifacts and registry cache
    Cargo,
    /// Clean Node.js node_modules and package manager caches
    Node,
    /// Clean Go module cache and build cache
    Go,
    /// Clean Python pip cache, __pycache__, and virtualenvs
    Python,
    /// Clean Java Gradle caches and Maven local repository
    Java,
    /// Clean Ruby gem cache
    Ruby,
    /// Clean Docker unused images, containers, volumes, and build cache
    Docker,
    /// Clean per-app cache directories (browsers, chat apps, editors, etc.)
    Cache,
    /// Empty the Trash
    Trash,
}

pub async fn run_clean(
    category: Option<CleanCategory>,
    min_size: Option<String>,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    let min_bytes = match min_size {
        Some(ref s) => parse_size(s)?,
        None => 0,
    };

    println!("{}", "Clario Clean".bold());
    println!();

    // Gather items based on category, printing progress as each scanner finishes
    let (file_items, docker_info) = gather_items(&category);

    // Filter by min_size and exclude SystemCritical
    let whitelist = crate::core::protection::load_whitelist();

    let filtered: Vec<FileInfo> = file_items
        .into_iter()
        .filter(|f| f.safety != SafetyLevel::SystemCritical)
        .filter(|f| f.size_bytes >= min_bytes)
        .filter(|f| {
            use crate::core::protection;
            protection::is_safe_to_delete(&f.path) && !protection::is_path_whitelisted(&f.path, &whitelist)
        })
        .collect();

    // Display summary table
    println!();
    print_summary(&filtered, docker_info.as_ref());

    let total_bytes: u64 = filtered.iter().map(|f| f.size_bytes).sum::<u64>()
        + docker_info.as_ref().map(|d| d.total()).unwrap_or(0);

    if total_bytes == 0 {
        println!("\n{}", "Nothing to clean.".green());
        return Ok(());
    }

    if dry_run {
        println!("\n{}", "Dry run — no files deleted.".yellow());
        return Ok(());
    }

    // Confirm / select
    let to_delete: Vec<&FileInfo> = if force {
        filtered.iter().collect()
    } else if io::stdout().is_terminal() {
        let labels: Vec<String> = filtered
            .iter()
            .map(|f| format!("{} ({})", f.name, format_size(f.size_bytes)))
            .collect();
        let defaults = vec![true; filtered.len()];
        println!();
        let chosen = MultiSelect::new()
            .with_prompt("Select items to delete (space to toggle, enter to confirm)")
            .items(&labels)
            .defaults(&defaults)
            .interact_opt()?;
        match chosen {
            Some(indices) if !indices.is_empty() => {
                indices.into_iter().map(|i| &filtered[i]).collect()
            }
            _ => {
                println!("{}", "Aborted.".yellow());
                return Ok(());
            }
        }
    } else {
        print!("\n{}", "Proceed with cleanup? [y/N] ".bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
        filtered.iter().collect()
    };

    // Delete files
    let mut freed: u64 = 0;
    for item in &to_delete {
        let path = item.path.clone();
        let is_trash_item = item.category == FileCategory::Trash;
        let is_dir = item.is_dir;
        // Trash items are already in the Trash — delete them permanently instead of
        // re-trashing (trash::delete on a Trash entry would just nest it deeper).
        let result = spin(&format!("Removing {}", path.display()), move || {
            if is_trash_item {
                if is_dir {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                }
            } else {
                trash::delete(&path).map_err(|e| io::Error::other(e.to_string()))
            }
        });
        match result {
            Ok(_) => {
                freed += item.size_bytes;
                println!("{}", "done".green());
            }
            Err(e) => println!("{} ({})", "failed".red(), e),
        }
    }

    // Docker cleanup
    if docker_info.is_some() {
        let status = spin("Running docker system prune", || {
            std::process::Command::new("docker").args(["system", "prune", "-f"]).status()
        });
        match status {
            Ok(s) if s.success() => {
                freed += docker_info.map(|d| d.total()).unwrap_or(0);
                println!("{}", "done".green());
            }
            _ => println!("{}", "failed".red()),
        }
    }

    println!("\n{} {}", "Freed:".bold(), format_size(freed).green().bold());
    Ok(())
}

/// Run one scanner behind a spinner, print the result on the same line once it finishes.
fn scan_step(label: &str, scan: impl FnOnce() -> Vec<FileInfo> + Send + 'static) -> Vec<FileInfo> {
    let items = spin(label, scan);
    let size: u64 = items.iter().map(|f| f.size_bytes).sum();
    if items.is_empty() {
        println!("{}", "nothing found".dimmed());
    } else {
        println!("{}", format_size(size).cyan());
    }
    items
}

/// Project build artifacts (node_modules, target, .venv, etc.) live across every project
/// under the configured purge search paths — same source `purge` uses, filtered down to
/// the artifact names relevant to the requested language category.
fn scan_project_artifacts(target_names: &[&str]) -> Vec<FileInfo> {
    let search_paths = purge_scanner::load_search_paths();
    purge_scanner::scan(&search_paths, RECENT_THRESHOLD_DAYS)
        .into_iter()
        .filter(|c| !c.is_recent && target_names.contains(&c.artifact.name.as_str()))
        .map(|c| {
            let mut info = c.artifact;
            info.category = match info.name.as_str() {
                "target" => FileCategory::CargoBuild,
                "node_modules" => FileCategory::NodeModules,
                ".venv" | "venv" => FileCategory::PythonVenv,
                "__pycache__" => FileCategory::PythonCache,
                ".gradle" => FileCategory::JavaGradle,
                _ => FileCategory::Other,
            };
            info
        })
        .collect()
}

fn gather_items(
    category: &Option<CleanCategory>,
) -> (Vec<FileInfo>, Option<dev_scanner::DockerInfo>) {
    let mut items = Vec::new();
    let mut docker = None;

    match category {
        Some(CleanCategory::Cargo) => {
            items.extend(scan_step("Cargo cache", dev_scanner::scan_cargo));
            items.extend(scan_step("Cargo target/ (all projects)", || scan_project_artifacts(&["target"])));
        }
        Some(CleanCategory::Node) => {
            items.extend(scan_step("Node cache", dev_scanner::scan_node));
            items.extend(scan_step("node_modules (all projects)", || scan_project_artifacts(&["node_modules"])));
        }
        Some(CleanCategory::Docker) => {
            docker = spin("Docker", dev_scanner::scan_docker);
            match &docker {
                Some(d) => println!("{}", format_size(d.total()).cyan()),
                None => {
                    println!("{}", "unavailable".dimmed());
                    eprintln!("{}", "Docker daemon not available, skipping.".yellow());
                }
            }
        }
        Some(CleanCategory::Go) => {
            items.extend(scan_step("Go cache", dev_scanner::scan_go));
        }
        Some(CleanCategory::Python) => {
            items.extend(scan_step("Python cache", dev_scanner::scan_python));
            items.extend(scan_step("Python venv/__pycache__ (all projects)", || {
                scan_project_artifacts(&[".venv", "venv", "__pycache__"])
            }));
        }
        Some(CleanCategory::Java) => {
            items.extend(scan_step("Gradle/Maven cache", dev_scanner::scan_java));
            items.extend(scan_step(".gradle (all projects)", || scan_project_artifacts(&[".gradle"])));
        }
        Some(CleanCategory::Ruby) => {
            items.extend(scan_step("Ruby gems", dev_scanner::scan_ruby));
        }
        Some(CleanCategory::Cache) => {
            items.extend(scan_step("App caches", dev_scanner::scan_cache));
        }
        Some(CleanCategory::Trash) => {
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            items.extend(scan_step("Trash", dev_scanner::scan_trash));
        }
        None => {
            items.extend(scan_step("Cargo cache", dev_scanner::scan_cargo));
            items.extend(scan_step("Node cache", dev_scanner::scan_node));
            items.extend(scan_step("Go cache", dev_scanner::scan_go));
            items.extend(scan_step("Python cache", dev_scanner::scan_python));
            items.extend(scan_step("Gradle/Maven cache", dev_scanner::scan_java));
            items.extend(scan_step("Ruby gems", dev_scanner::scan_ruby));
            items.extend(scan_step("Project artifacts (all projects)", || {
                scan_project_artifacts(&["target", "node_modules", ".venv", "venv", "__pycache__", ".gradle"])
            }));
            items.extend(scan_step("App caches", dev_scanner::scan_cache));
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            items.extend(scan_step("Trash", dev_scanner::scan_trash));

            docker = spin("Docker", dev_scanner::scan_docker);
            match &docker {
                Some(d) if d.total() > 0 => println!("{}", format_size(d.total()).cyan()),
                _ => println!("{}", "nothing found".dimmed()),
            }
        }
    }

    (items, docker)
}

fn print_summary(items: &[FileInfo], docker: Option<&dev_scanner::DockerInfo>) {
    let col_w = 24;
    println!(
        "{:<col_w$} {:>8}  {}",
        "Category".bold(),
        "Items".bold(),
        "Size".bold(),
    );
    println!("{}", "─".repeat(46));

    // Group by category label
    let groups: &[(&str, FileCategory)] = &[
        ("Cargo cache", FileCategory::CargoCache),
        ("Cargo target/", FileCategory::CargoBuild),
        ("node_modules", FileCategory::NodeModules),
        ("Node cache", FileCategory::NodeCache),
        ("Go module cache", FileCategory::GoCache),
        ("Go build cache", FileCategory::GoBuild),
        ("Python cache", FileCategory::PythonCache),
        ("Python venv", FileCategory::PythonVenv),
        ("Gradle cache", FileCategory::JavaGradle),
        ("Maven repository", FileCategory::JavaMaven),
        ("Ruby gems", FileCategory::RubyGems),
        ("Logs", FileCategory::Log),
    ];

    let mut total_items = 0usize;
    let mut total_bytes = 0u64;

    for (label, cat) in groups {
        let matched: Vec<&FileInfo> = items.iter().filter(|f| &f.category == cat).collect();
        if matched.is_empty() {
            continue;
        }
        let count = matched.len();
        let size: u64 = matched.iter().map(|f| f.size_bytes).sum();
        total_items += count;
        total_bytes += size;
        println!(
            "{:<col_w$} {:>8}  {}",
            label,
            count,
            format_size(size).cyan()
        );
    }

    // App caches are shown per-app (not summed into one line) so the user can
    // see which app is the biggest offender, e.g. "google-chrome  2.1 GB".
    let mut app_caches: Vec<&FileInfo> = items
        .iter()
        .filter(|f| f.category == FileCategory::AppCache)
        .collect();
    if !app_caches.is_empty() {
        app_caches.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        for item in &app_caches {
            total_items += 1;
            total_bytes += item.size_bytes;
            println!(
                "{:<col_w$} {:>8}  {}",
                format!("App cache: {}", item.name),
                "—",
                format_size(item.size_bytes).cyan()
            );
        }
    }

    let trash_items: Vec<&FileInfo> = items
        .iter()
        .filter(|f| f.category == FileCategory::Trash)
        .collect();
    if !trash_items.is_empty() {
        let count = trash_items.len();
        let size: u64 = trash_items.iter().map(|f| f.size_bytes).sum();
        total_items += count;
        total_bytes += size;
        println!("{:<col_w$} {:>8}  {}", "Trash", count, format_size(size).cyan());
    }

    if let Some(d) = docker {
        let docker_total = d.total();
        if docker_total > 0 {
            total_bytes += docker_total;
            println!(
                "{:<col_w$} {:>8}  {}",
                "Docker",
                "—",
                format_size(docker_total).cyan()
            );
        }
    }

    println!("{}", "─".repeat(46));
    println!(
        "{:<col_w$} {:>8}  {}",
        "Total".bold(),
        total_items,
        format_size(total_bytes).green().bold()
    );
}
