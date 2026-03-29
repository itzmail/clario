use crate::models::process_info::{ProcessInfo, SuspicionFlag};
use std::path::Path;
use sysinfo::System;

/// Trusted root paths for D-01: executables in these directories are not flagged
#[cfg(target_os = "macos")]
const TRUSTED_PATHS: &[&str] = &[
    "/usr/",
    "/bin/",
    "/sbin/",
    "/System/",
    "/Applications/",
    "/Library/",
];

#[cfg(target_os = "linux")]
const TRUSTED_PATHS: &[&str] = &[
    "/usr/",
    "/bin/",
    "/sbin/",
    "/opt/",
    "/snap/",
    "/flatpak/",
    "/nix/",
];

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
const TRUSTED_PATHS: &[&str] = &["/usr/", "/bin/", "/sbin/"];

/// System paths for D-03: root processes in these directories are not flagged
#[cfg(target_os = "macos")]
const ROOT_SYSTEM_PATHS: &[&str] = &["/System/", "/usr/sbin/", "/sbin/"];

#[cfg(target_os = "linux")]
const ROOT_SYSTEM_PATHS: &[&str] = &["/usr/sbin/", "/sbin/", "/usr/lib/systemd/"];

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
const ROOT_SYSTEM_PATHS: &[&str] = &["/usr/sbin/", "/sbin/"];

/// Known bad name patterns for D-05 (lowercase substrings)
const KNOWN_BAD_PATTERNS: &[&str] = &["xmrig", "cryptonight", "miner"];

/// Stateless scanner for running processes
pub struct ProcessScanner;

impl ProcessScanner {
    /// Scans all running processes via sysinfo::System and returns them sorted suspicious-first
    pub fn scan(sys: &System) -> Vec<ProcessInfo> {
        let mut results: Vec<ProcessInfo> = sys
            .processes()
            .values()
            .map(|process| {
                let name = process.name().to_string_lossy().to_string();
                let exe_path = process.exe().map(|e| e.to_path_buf());
                let cpu_usage = process.cpu_usage();
                let memory_bytes = process.memory();
                let user_id = process.user_id().map(|u| **u);
                let parent_pid = process.parent();
                let run_time_secs = process.run_time();

                let exe_str = exe_path.as_deref().and_then(|p| p.to_str()).map(String::from);

                let suspicion_flags = apply_rules(&name, exe_str.as_deref(), user_id, cpu_usage);

                ProcessInfo {
                    pid: process.pid(),
                    name,
                    exe_path,
                    cpu_usage,
                    memory_bytes,
                    user_id,
                    parent_pid,
                    run_time_secs,
                    suspicion_flags,
                    is_selected: false,
                }
            })
            .collect();

        // Sort suspicious-first (more flags first), then alphabetical by name
        results.sort_by(|a, b| {
            b.suspicion_flags
                .len()
                .cmp(&a.suspicion_flags.len())
                .then(a.name.cmp(&b.name))
        });

        results
    }
}

/// Applies all 5 suspicion rules to a process and returns the list of triggered flags.
/// Extracted as a standalone function for testability without needing a real sysinfo::System.
pub fn apply_rules(
    name: &str,
    exe_path: Option<&str>,
    uid: Option<u32>,
    cpu: f32,
) -> Vec<SuspicionFlag> {
    let mut flags = Vec::new();

    if let Some(exe_str) = exe_path {
        // D-01: SuspiciousPath — exe outside all trusted paths
        let in_trusted = TRUSTED_PATHS.iter().any(|prefix| exe_str.starts_with(prefix));
        if !in_trusted {
            flags.push(SuspicionFlag::SuspiciousPath(exe_str.to_string()));
        }

        // D-02: NameExeMismatch — process name != exe basename
        let exe_basename = Path::new(exe_str)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        if !exe_basename.is_empty() && name != exe_basename {
            flags.push(SuspicionFlag::NameExeMismatch(format!(
                "name='{}' exe='{}'",
                name, exe_basename
            )));
        }

        // D-03: RootOutsideSystemPath — UID 0 and exe not in system paths
        if uid == Some(0) {
            let in_root_system = ROOT_SYSTEM_PATHS
                .iter()
                .any(|prefix| exe_str.starts_with(prefix));
            if !in_root_system {
                flags.push(SuspicionFlag::RootOutsideSystemPath(exe_str.to_string()));
            }
        }
    }

    // D-04: HighCpu — no exe requirement
    if cpu > 80.0 {
        flags.push(SuspicionFlag::HighCpu(cpu));
    }

    // D-05: KnownBadName — check lowercase name for known bad patterns
    let name_lower = name.to_lowercase();
    for pattern in KNOWN_BAD_PATTERNS {
        if name_lower.contains(pattern) {
            flags.push(SuspicionFlag::KnownBadName(pattern.to_string()));
            break;
        }
    }

    flags
}

/// Formats memory bytes into a human-readable string (G/M/K)
pub fn format_memory(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    const KB: u64 = 1024;

    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{}M", bytes / MB)
    } else if bytes >= KB {
        format!("{}K", bytes / KB)
    } else {
        format!("{}B", bytes)
    }
}

/// Formats uptime in seconds into a human-readable string
pub fn format_uptime(secs: u64) -> String {
    const HOUR: u64 = 3600;
    const MINUTE: u64 = 60;

    if secs >= HOUR {
        let h = secs / HOUR;
        let m = (secs % HOUR) / MINUTE;
        format!("{}h {}m", h, m)
    } else if secs >= MINUTE {
        let m = secs / MINUTE;
        let s = secs % MINUTE;
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- D-01 tests ---

    #[test]
    fn test_d01_suspicious_path() {
        let flags = apply_rules("evil", Some("/tmp/evil"), None, 0.0);
        assert!(
            flags.iter().any(|f| matches!(f, SuspicionFlag::SuspiciousPath(_))),
            "Expected SuspiciousPath flag for /tmp/evil"
        );
    }

    #[test]
    fn test_d01_trusted_path() {
        let flags = apply_rules("ls", Some("/usr/bin/ls"), None, 0.0);
        assert!(
            !flags.iter().any(|f| matches!(f, SuspicionFlag::SuspiciousPath(_))),
            "Should not flag /usr/bin/ls as suspicious path"
        );
    }

    // --- D-02 tests ---

    #[test]
    fn test_d02_name_mismatch() {
        let flags = apply_rules("Safari", Some("/tmp/malware"), None, 0.0);
        assert!(
            flags.iter().any(|f| matches!(f, SuspicionFlag::NameExeMismatch(_))),
            "Expected NameExeMismatch when name='Safari' and exe basename='malware'"
        );
    }

    #[test]
    fn test_d02_name_match() {
        let flags = apply_rules("ls", Some("/usr/bin/ls"), None, 0.0);
        assert!(
            !flags.iter().any(|f| matches!(f, SuspicionFlag::NameExeMismatch(_))),
            "Should not flag name='ls' with exe='ls' as mismatch"
        );
    }

    // --- D-03 tests ---

    #[test]
    fn test_d03_root_outside_system() {
        let flags = apply_rules("evil", Some("/tmp/evil"), Some(0), 0.0);
        assert!(
            flags.iter().any(|f| matches!(f, SuspicionFlag::RootOutsideSystemPath(_))),
            "Expected RootOutsideSystemPath for uid=0 at /tmp/evil"
        );
    }

    #[test]
    fn test_d03_root_in_system() {
        let flags = apply_rules("httpd", Some("/usr/sbin/httpd"), Some(0), 0.0);
        assert!(
            !flags.iter().any(|f| matches!(f, SuspicionFlag::RootOutsideSystemPath(_))),
            "Should not flag uid=0 at /usr/sbin/httpd as outside system path"
        );
    }

    // --- D-04 tests ---

    #[test]
    fn test_d04_high_cpu() {
        let flags = apply_rules("heavy", None, None, 85.0);
        assert!(
            flags.iter().any(|f| matches!(f, SuspicionFlag::HighCpu(_))),
            "Expected HighCpu flag for cpu=85.0"
        );
    }

    #[test]
    fn test_d04_normal_cpu() {
        let flags = apply_rules("normal", None, None, 50.0);
        assert!(
            !flags.iter().any(|f| matches!(f, SuspicionFlag::HighCpu(_))),
            "Should not flag cpu=50.0 as high"
        );
    }

    // --- D-05 tests ---

    #[test]
    fn test_d05_known_bad() {
        let flags = apply_rules("xmrig-worker", None, None, 0.0);
        assert!(
            flags.iter().any(|f| matches!(f, SuspicionFlag::KnownBadName(_))),
            "Expected KnownBadName for 'xmrig-worker'"
        );
    }

    #[test]
    fn test_d05_clean_name() {
        let flags = apply_rules("safari", None, None, 0.0);
        assert!(
            !flags.iter().any(|f| matches!(f, SuspicionFlag::KnownBadName(_))),
            "Should not flag 'safari' as known bad name"
        );
    }

    // --- exe=None safety test ---

    #[test]
    fn test_no_exe_no_path_rules() {
        let flags = apply_rules("mystery", None, Some(0), 0.0);
        assert!(
            !flags.iter().any(|f| matches!(f, SuspicionFlag::SuspiciousPath(_))),
            "exe=None should not trigger SuspiciousPath"
        );
        assert!(
            !flags.iter().any(|f| matches!(f, SuspicionFlag::NameExeMismatch(_))),
            "exe=None should not trigger NameExeMismatch"
        );
        assert!(
            !flags.iter().any(|f| matches!(f, SuspicionFlag::RootOutsideSystemPath(_))),
            "exe=None should not trigger RootOutsideSystemPath even for root uid"
        );
    }

    // --- format_memory tests ---

    #[test]
    fn test_format_memory() {
        assert_eq!(format_memory(1073741824), "1.0G", "1 GiB should format as 1.0G");
        assert_eq!(format_memory(1048576), "1M", "1 MiB should format as 1M");
        assert_eq!(format_memory(1024), "1K", "1 KiB should format as 1K");
    }

    // --- format_uptime tests ---

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "30s", "30 seconds should format as 30s");
        assert_eq!(format_uptime(90), "1m 30s", "90 seconds should format as 1m 30s");
        assert_eq!(format_uptime(3661), "1h 1m", "3661 seconds should format as 1h 1m");
    }
}
