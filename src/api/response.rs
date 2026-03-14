/// Response formatting — JSON pretty-printing with syntax highlighting.
///
/// Mirrors the Go `utils.FormatAndPrintResponse` function, producing
/// colorized JSON output to stdout.
use colored::Colorize;

/// Formats and prints a JSON value with syntax highlighting.
pub fn format_and_print_response(value: &serde_json::Value) {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    colorize_and_print_json(&pretty);
}

/// Prints JSON string with line-by-line syntax highlighting.
fn colorize_and_print_json(json_str: &str) {
    for line in json_str.lines() {
        let trimmed = line.trim();

        // Structure characters
        if matches!(trimmed, "{" | "}" | "[" | "]" | "," | "}," | "],") {
            println!("{}", line.white().bold());
            continue;
        }

        if let Some(colon_pos) = line.find(':') {
            let key = &line[..colon_pos];
            let value = line[colon_pos + 1..].trim();

            // Key
            print!("{}", key.cyan().bold());
            print!(":");

            // Value starts a nested structure
            if value.ends_with('{') || value.ends_with('[') {
                let without_bracket = value.trim_end_matches(['{', '[']);
                if !without_bracket.is_empty() {
                    print!("{without_bracket}");
                }
                let bracket = &value[without_bracket.len()..];
                println!("{}", bracket.white().bold());
                continue;
            }

            // Value ends with a closing bracket
            if value.ends_with('}')
                || value.ends_with(']')
                || value.ends_with("},")
                || value.ends_with("],")
            {
                // Find the last bracket position
                let last_bracket = value
                    .rfind(['}', ']'])
                    .unwrap_or(value.len());

                if last_bracket > 0 {
                    let before = &value[..last_bracket];
                    let bracket_part = &value[last_bracket..];
                    print_colorized_value(before);
                    print!("{}", bracket_part.white().bold());
                    println!();
                } else {
                    println!("{}", value.white().bold());
                }
                continue;
            }

            print_colorized_value_ln(value);
        } else {
            print_colorized_value_ln(line);
        }
    }
}

/// Prints a value with color based on its JSON type, with newline.
fn print_colorized_value_ln(value: &str) {
    print_colorized_value(value);
    println!();
}

/// Prints a value with color based on its JSON type.
fn print_colorized_value(value: &str) {
    let trimmed = value.trim();

    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('"') && trimmed.ends_with("\","))
    {
        // String value
        print!("{}", value.green());
    } else if matches!(
        trimmed,
        "true" | "false" | "true," | "false,"
    ) {
        // Boolean value
        print!("{}", value.magenta());
    } else if matches!(trimmed, "null" | "null,") {
        // Null value
        print!("{}", value.red());
    } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
        // Structure
        print!("{}", value.white().bold());
    } else {
        // Number or other
        print!("{}", value.yellow());
    }
}
