use crate::core::dev_scanner;
use crate::models::file_info::{FileCategory, FileInfo, SafetyLevel};
use crate::utils::size::{format_size, parse_size};
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

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
    /// Clean system log files
    Logs,
    /// Clean system cache directories
    Cache,
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
    println!("{}", "Scanning...".dimmed());

    // Gather items based on category
    let (file_items, docker_info) = gather_items(&category);

    // Filter by min_size and exclude SystemCritical
    let filtered: Vec<FileInfo> = file_items
        .into_iter()
        .filter(|f| f.safety != SafetyLevel::SystemCritical)
        .filter(|f| f.size_bytes >= min_bytes)
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

    // Confirm
    if !force {
        print!("\n{}", "Proceed with cleanup? [y/N] ".bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }

    // Delete files
    let mut freed: u64 = 0;
    for item in &filtered {
        print!("  Removing {}... ", item.path.display());
        io::stdout().flush()?;
        match trash::delete(&item.path) {
            Ok(_) => {
                freed += item.size_bytes;
                println!("{}", "done".green());
            }
            Err(e) => println!("{} ({})", "failed".red(), e),
        }
    }

    // Docker cleanup
    if docker_info.is_some() {
        print!("  Running docker system prune... ");
        io::stdout().flush()?;
        let status = std::process::Command::new("docker")
            .args(["system", "prune", "-f"])
            .status();
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

fn gather_items(
    category: &Option<CleanCategory>,
) -> (Vec<FileInfo>, Option<dev_scanner::DockerInfo>) {
    let mut items = Vec::new();
    let mut docker = None;

    match category {
        Some(CleanCategory::Cargo) => {
            items.extend(dev_scanner::scan_cargo());
        }
        Some(CleanCategory::Node) => {
            items.extend(dev_scanner::scan_node());
        }
        Some(CleanCategory::Docker) => {
            docker = dev_scanner::scan_docker();
            if docker.is_none() {
                eprintln!("{}", "Docker daemon not available, skipping.".yellow());
            }
        }
        Some(CleanCategory::Logs) => {
            items.extend(dev_scanner::scan_logs());
        }
        Some(CleanCategory::Go) => {
            items.extend(dev_scanner::scan_go());
        }
        Some(CleanCategory::Python) => {
            items.extend(dev_scanner::scan_python());
        }
        Some(CleanCategory::Java) => {
            items.extend(dev_scanner::scan_java());
        }
        Some(CleanCategory::Ruby) => {
            items.extend(dev_scanner::scan_ruby());
        }
        Some(CleanCategory::Cache) => {
            items.extend(dev_scanner::scan_cache());
        }
        None => {
            // Scan all categories
            items.extend(dev_scanner::scan_cargo());
            items.extend(dev_scanner::scan_node());
            items.extend(dev_scanner::scan_go());
            items.extend(dev_scanner::scan_python());
            items.extend(dev_scanner::scan_java());
            items.extend(dev_scanner::scan_ruby());
            items.extend(dev_scanner::scan_logs());
            items.extend(dev_scanner::scan_cache());
            docker = dev_scanner::scan_docker();
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
        ("System cache", FileCategory::Cache),
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
