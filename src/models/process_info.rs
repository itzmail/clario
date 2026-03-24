use sysinfo::Pid;
use std::path::PathBuf;

/// Severity level of suspicious activity for a process
#[derive(Debug, Clone, PartialEq)]
pub enum SuspicionSeverity {
    Clean,
    Warning,
    Danger,
}

/// Flags encoding which suspicion rules (D-01 through D-05) a process triggered
#[derive(Debug, Clone)]
pub enum SuspicionFlag {
    /// D-01: Executable located outside trusted system directories
    SuspiciousPath(String),
    /// D-02: Process name does not match the executable filename
    NameExeMismatch(String),
    /// D-03: Running as root from a non-system path
    RootOutsideSystemPath(String),
    /// D-04: Sustained high CPU usage above threshold
    HighCpu(f32),
    /// D-05: Process name matches a known malicious pattern
    KnownBadName(String),
}

impl SuspicionFlag {
    /// Returns a human-readable explanation of why this flag was raised
    pub fn display_reason(&self) -> String {
        match self {
            SuspicionFlag::SuspiciousPath(path) => {
                format!("Executable path outside trusted system locations: {}", path)
            }
            SuspicionFlag::NameExeMismatch(detail) => {
                format!("Process name does not match executable filename: {}", detail)
            }
            SuspicionFlag::RootOutsideSystemPath(path) => {
                format!("Running as root from non-system path: {}", path)
            }
            SuspicionFlag::HighCpu(cpu) => {
                format!("Sustained high CPU usage: {:.1}%", cpu)
            }
            SuspicionFlag::KnownBadName(name) => {
                format!("Process name matches known malicious pattern: {}", name)
            }
        }
    }
}

/// Data model representing a running process with suspicion analysis
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub exe_path: Option<PathBuf>,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub user_id: Option<u32>,
    pub parent_pid: Option<Pid>,
    pub run_time_secs: u64,
    pub suspicion_flags: Vec<SuspicionFlag>,
    pub is_selected: bool,
}

impl ProcessInfo {
    /// Returns the suspicion severity based on the number of flags raised
    pub fn severity(&self) -> SuspicionSeverity {
        match self.suspicion_flags.len() {
            0 => SuspicionSeverity::Clean,
            1 => SuspicionSeverity::Warning,
            _ => SuspicionSeverity::Danger,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::Pid;

    fn make_process_info(flags: Vec<SuspicionFlag>) -> ProcessInfo {
        ProcessInfo {
            pid: Pid::from(1usize),
            name: "test-process".to_string(),
            exe_path: None,
            cpu_usage: 0.0,
            memory_bytes: 0,
            user_id: None,
            parent_pid: None,
            run_time_secs: 0,
            suspicion_flags: flags,
            is_selected: false,
        }
    }

    #[test]
    fn test_severity_clean() {
        let p = make_process_info(vec![]);
        assert_eq!(p.severity(), SuspicionSeverity::Clean);
    }

    #[test]
    fn test_severity_warning() {
        let p = make_process_info(vec![SuspicionFlag::HighCpu(85.0)]);
        assert_eq!(p.severity(), SuspicionSeverity::Warning);
    }

    #[test]
    fn test_severity_danger() {
        let p = make_process_info(vec![
            SuspicionFlag::HighCpu(85.0),
            SuspicionFlag::SuspiciousPath("/tmp/evil".to_string()),
        ]);
        assert_eq!(p.severity(), SuspicionSeverity::Danger);
    }

    #[test]
    fn test_display_reason_variants() {
        let flags = vec![
            SuspicionFlag::SuspiciousPath("/tmp/evil".to_string()),
            SuspicionFlag::NameExeMismatch("name='Safari' exe='malware'".to_string()),
            SuspicionFlag::RootOutsideSystemPath("/tmp/rootevil".to_string()),
            SuspicionFlag::HighCpu(92.5),
            SuspicionFlag::KnownBadName("xmrig".to_string()),
        ];

        let reasons: Vec<String> = flags.iter().map(|f| f.display_reason()).collect();

        // Each reason should be non-empty and contain the key detail
        assert!(reasons[0].contains("/tmp/evil"), "SuspiciousPath reason should contain path");
        assert!(reasons[1].contains("Safari"), "NameExeMismatch reason should contain detail");
        assert!(reasons[2].contains("/tmp/rootevil"), "RootOutsideSystemPath reason should contain path");
        assert!(reasons[3].contains("92.5"), "HighCpu reason should contain percentage");
        assert!(reasons[4].contains("xmrig"), "KnownBadName reason should contain name");

        for reason in &reasons {
            assert!(!reason.is_empty(), "display_reason() must not return empty string");
        }
    }
}
