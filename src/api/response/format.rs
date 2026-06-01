/// Response formatting — JSON pretty-printing with syntax highlighting.
///
/// Mirrors the Go `utils.FormatAndPrintResponse` function, producing
/// colorized JSON output to a caller-supplied writer.
use std::io::{self, Write};

use colored::Colorize;

/// Formats a JSON value with syntax highlighting and writes it to `out`.
///
/// # Errors
///
/// Returns any I/O error from writing to `out`. The typical caller
/// (`OutputConfig::print_response`) treats this best-effort and ignores
/// the error so a closed pipe doesn't abort the program.
pub fn format_response(out: &mut dyn Write, value: &serde_json::Value) -> io::Result<()> {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    colorize_json(out, &pretty)
}

/// Writes a JSON string to `out` with line-by-line syntax highlighting.
fn colorize_json(out: &mut dyn Write, json_str: &str) -> io::Result<()> {
    for line in json_str.lines() {
        let trimmed = line.trim();

        // Structure characters
        if matches!(trimmed, "{" | "}" | "[" | "]" | "," | "}," | "],") {
            writeln!(out, "{}", line.white().bold())?;
            continue;
        }

        if let Some(colon_pos) = line.find(':') {
            let key = &line[..colon_pos];
            let value = line[colon_pos + 1..].trim();

            // Key
            write!(out, "{}", key.cyan().bold())?;
            write!(out, ":")?;

            // Value starts a nested structure
            if value.ends_with('{') || value.ends_with('[') {
                let without_bracket = value.trim_end_matches(['{', '[']);
                if !without_bracket.is_empty() {
                    write!(out, "{without_bracket}")?;
                }
                let bracket = &value[without_bracket.len()..];
                writeln!(out, "{}", bracket.white().bold())?;
                continue;
            }

            // Value ends with a closing bracket
            if value.ends_with('}')
                || value.ends_with(']')
                || value.ends_with("},")
                || value.ends_with("],")
            {
                // Find the last bracket position
                let last_bracket = value.rfind(['}', ']']).unwrap_or(value.len());

                if last_bracket > 0 {
                    let before = &value[..last_bracket];
                    let bracket_part = &value[last_bracket..];
                    write_colorized_value(out, before)?;
                    write!(out, "{}", bracket_part.white().bold())?;
                    writeln!(out)?;
                } else {
                    writeln!(out, "{}", value.white().bold())?;
                }
                continue;
            }

            write_colorized_value_ln(out, value)?;
        } else {
            write_colorized_value_ln(out, line)?;
        }
    }
    Ok(())
}

/// Writes a value with color based on its JSON type, with newline.
fn write_colorized_value_ln(out: &mut dyn Write, value: &str) -> io::Result<()> {
    write_colorized_value(out, value)?;
    writeln!(out)
}

/// Writes a value with color based on its JSON type.
fn write_colorized_value(out: &mut dyn Write, value: &str) -> io::Result<()> {
    let trimmed = value.trim();

    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('"') && trimmed.ends_with("\","))
    {
        // String value
        write!(out, "{}", value.green())
    } else if matches!(trimmed, "true" | "false" | "true," | "false,") {
        // Boolean value
        write!(out, "{}", value.magenta())
    } else if matches!(trimmed, "null" | "null,") {
        // Null value
        write!(out, "{}", value.red())
    } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
        // Structure
        write!(out, "{}", value.white().bold())
    } else {
        // Number or other
        write!(out, "{}", value.yellow())
    }
}
