use anyhow::{anyhow, Result};

/// Parse a human-readable size string into bytes.
/// Accepts: "100MB", "1GB", "500KB", "1.5GB" (case-insensitive)
pub fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim().to_uppercase();
    let (num_part, unit) = if s.ends_with("GB") {
        (&s[..s.len() - 2], 1024u64 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1024u64 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1024u64)
    } else if s.ends_with('B') {
        (&s[..s.len() - 1], 1u64)
    } else {
        // Assume bytes if no unit
        return s.parse::<u64>().map_err(|_| anyhow!("Invalid size: {}", s));
    };

    let num: f64 = num_part
        .trim()
        .parse()
        .map_err(|_| anyhow!("Invalid size number: {}", num_part))?;
    Ok((num * unit as f64) as u64)
}

/// Format bytes into a human-readable string (e.g., "1.2 GB", "345 MB", "12 KB")
pub fn format_size(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    const KB: u64 = 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("100MB").unwrap(), 100 * 1024 * 1024);
        assert_eq!(parse_size("500KB").unwrap(), 500 * 1024);
        assert_eq!(parse_size("1gb").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size(100 * 1024 * 1024), "100.0 MB");
        assert_eq!(format_size(500), "500 B");
    }
}
