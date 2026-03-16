//! Output normalization pipeline for differential conformance testing.
//!
//! Handles non-deterministic output fields (timestamps, IDs, paths)
//! so that meaningful differences can be detected while ignoring
//! expected variations.

use regex::Regex;
use std::sync::LazyLock;

// ── Compiled regexes ───────────────────────────────────────────────────────

static TIMESTAMP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[^\s]*"
    ).unwrap()
});

static UNIX_EPOCH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{10,13}\b").unwrap()
});

static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
    ).unwrap()
});

static SNOWFLAKE_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{17,19}\b").unwrap()
});

static HEX_HASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[0-9a-f]{32,64}").unwrap()
});

static PID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"pid[= ]\d+").unwrap()
});

static RELATIVE_TIME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\d+ (?:second|minute|hour|day|week|month|year)s? ago"
    ).unwrap()
});

// ── Normalization functions ────────────────────────────────────────────────

/// Normalize timestamps (ISO 8601 format).
pub fn normalize_timestamps(input: &str) -> String {
    TIMESTAMP_RE.replace_all(input, "TIMESTAMP").to_string()
}

/// Normalize Unix epoch timestamps.
pub fn normalize_epoch(input: &str) -> String {
    UNIX_EPOCH_RE.replace_all(input, "EPOCH").to_string()
}

/// Normalize UUIDs.
pub fn normalize_uuids(input: &str) -> String {
    UUID_RE.replace_all(input, "UUID").to_string()
}

/// Normalize Twitter/X snowflake IDs (17-19 digit numbers).
pub fn normalize_snowflake_ids(input: &str) -> String {
    SNOWFLAKE_ID_RE.replace_all(input, "SNOWFLAKE_ID").to_string()
}

/// Normalize hex hashes (32-64 character hex strings).
pub fn normalize_hex_hashes(input: &str) -> String {
    HEX_HASH_RE.replace_all(input, "HASH").to_string()
}

/// Normalize process IDs.
pub fn normalize_pids(input: &str) -> String {
    PID_RE.replace_all(input, "pid=PID").to_string()
}

/// Normalize relative time expressions ("2 hours ago", etc.).
pub fn normalize_relative_time(input: &str) -> String {
    RELATIVE_TIME_RE.replace_all(input, "TIME_AGO").to_string()
}

/// Normalize path separators (Windows backslashes to forward slashes).
pub fn normalize_paths(input: &str) -> String {
    input.replace('\\', "/")
}

/// Strip carriage returns (\r) for Windows line-ending compatibility.
pub fn normalize_line_endings(input: &str) -> String {
    input.replace('\r', "")
}

static VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"xurl \d+\.\d+\.\d+").unwrap()
});

/// Normalize version strings so "xurl 1.0.3" and "xurl 0.1.0" compare equal.
pub fn normalize_version_string(input: &str) -> String {
    VERSION_RE.replace_all(input, "xurl VERSION").to_string()
}

/// Normalize help text — collapse whitespace and remove framework-specific
/// formatting differences between Go cobra and Rust clap.
pub fn normalize_help_text(input: &str) -> String {
    // Strip ANSI escape codes
    let ansi_re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let s = ansi_re.replace_all(input, "").to_string();
    // Collapse runs of whitespace to single space per line, trim lines
    s.lines()
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Apply a named normalization to the input.
fn apply_normalization(input: &str, name: &str) -> String {
    match name {
        "timestamps" => normalize_timestamps(input),
        "epoch" => normalize_epoch(input),
        "uuids" => normalize_uuids(input),
        "snowflake_ids" => normalize_snowflake_ids(input),
        "hex_hashes" => normalize_hex_hashes(input),
        "pids" => normalize_pids(input),
        "relative_time" => normalize_relative_time(input),
        "paths" => normalize_paths(input),
        "line_endings" => normalize_line_endings(input),
        "version_string" => normalize_version_string(input),
        "help_text" => normalize_help_text(input),
        "all" => {
            let s = normalize_line_endings(input);
            let s = normalize_paths(&s);
            let s = normalize_timestamps(&s);
            let s = normalize_uuids(&s);
            let s = normalize_snowflake_ids(&s);
            let s = normalize_pids(&s);
            normalize_relative_time(&s)
        }
        "ids" => {
            let s = normalize_uuids(input);
            normalize_snowflake_ids(&s)
        }
        _ => input.to_string(),
    }
}

/// Apply a pipeline of normalizations to the input string.
pub fn normalize_output(input: &str, normalizations: &[String]) -> String {
    let mut result = input.to_string();
    for norm in normalizations {
        result = apply_normalization(&result, norm);
    }
    result
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_timestamps() {
        let input = "Created at 2024-01-15T10:30:45Z by user";
        let result = normalize_timestamps(input);
        assert_eq!(result, "Created at TIMESTAMP by user");
    }

    #[test]
    fn test_normalize_timestamps_with_offset() {
        let input = "time: 2024-01-15T10:30:45+05:00";
        let result = normalize_timestamps(input);
        assert_eq!(result, "time: TIMESTAMP");
    }

    #[test]
    fn test_normalize_uuids() {
        let input = "id: 550e8400-e29b-41d4-a716-446655440000";
        let result = normalize_uuids(input);
        assert_eq!(result, "id: UUID");
    }

    #[test]
    fn test_normalize_snowflake_ids() {
        let input = r#"{"id": "1234567890123456789"}"#;
        let result = normalize_snowflake_ids(input);
        assert_eq!(result, r#"{"id": "SNOWFLAKE_ID"}"#);
    }

    #[test]
    fn test_normalize_pids() {
        let input = "Process pid=12345 started";
        let result = normalize_pids(input);
        assert_eq!(result, "Process pid=PID started");
    }

    #[test]
    fn test_normalize_relative_time() {
        let input = "Posted 2 hours ago";
        let result = normalize_relative_time(input);
        assert_eq!(result, "Posted TIME_AGO");
    }

    #[test]
    fn test_normalize_paths() {
        let input = r"C:\Users\test\.xurl";
        let result = normalize_paths(input);
        assert_eq!(result, "C:/Users/test/.xurl");
    }

    #[test]
    fn test_normalize_line_endings() {
        let input = "line1\r\nline2\r\n";
        let result = normalize_line_endings(input);
        assert_eq!(result, "line1\nline2\n");
    }

    #[test]
    fn test_normalize_output_pipeline() {
        let input = "Created at 2024-01-15T10:30:45Z id=550e8400-e29b-41d4-a716-446655440000";
        let result = normalize_output(
            input,
            &["timestamps".to_string(), "uuids".to_string()],
        );
        assert_eq!(result, "Created at TIMESTAMP id=UUID");
    }

    #[test]
    fn test_normalize_all() {
        let input = "Created 2024-01-15T10:30:45Z pid=123 550e8400-e29b-41d4-a716-446655440000 3 days ago";
        let result = normalize_output(input, &["all".to_string()]);
        assert!(result.contains("TIMESTAMP"));
        assert!(result.contains("pid=PID"));
        assert!(result.contains("UUID"));
        assert!(result.contains("TIME_AGO"));
    }

    #[test]
    fn test_normalize_no_match() {
        let input = "simple text with no special patterns";
        let result = normalize_output(input, &["timestamps".to_string()]);
        assert_eq!(result, input);
    }

    #[test]
    fn test_normalize_empty_pipeline() {
        let input = "unchanged";
        let result = normalize_output(input, &[]);
        assert_eq!(result, "unchanged");
    }

    #[test]
    fn test_normalize_ids_combined() {
        let input = "uuid=550e8400-e29b-41d4-a716-446655440000 tweet=1234567890123456789";
        let result = normalize_output(input, &["ids".to_string()]);
        assert!(result.contains("UUID"));
        assert!(result.contains("SNOWFLAKE_ID"));
    }
}
