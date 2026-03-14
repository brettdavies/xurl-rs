use serde_json;
fn key_color_init() -> Box<Color> {
    todo!("package-level var init")
}
fn string_color_init() -> Box<Color> {
    todo!("package-level var init")
}
fn number_color_init() -> Box<Color> {
    todo!("package-level var init")
}
fn bool_color_init() -> Box<Color> {
    todo!("package-level var init")
}
fn null_color_init() -> Box<Color> {
    todo!("package-level var init")
}
fn structure_color_init() -> Box<Color> {
    todo!("package-level var init")
}
/// colorizeAndPrintJSON prints JSON with syntax highlighting
fn colorize_and_print_json(json_str: &str) {
    let mut lines = json_str.split("\n").collect::<Vec<&str>>();
    for line in lines.iter() {
        let mut trimmed_line = line.trim();
        if trimmed_line == "{" || trimmed_line == "}" || trimmed_line == "["
            || trimmed_line == "]" || trimmed_line == "," || trimmed_line == "},"
            || trimmed_line == "],"
        {
            structure_color.println(line);
            continue;
        }
        if line.contains(":") {
            let mut parts = strings.split_n(line, ":", 2);
            let mut key = parts[0];
            let mut value = parts[1].trim();
            key_color.print(key);
            fmt.print(":");
            if value.ends_with("{") || value.ends_with("[") {
                let mut value_without_bracket = value
                    .strip_suffix("{")
                    .unwrap_or(value)
                    .strip_suffix("[")
                    .unwrap_or(value.strip_suffix("{").unwrap_or(value));
                if value_without_bracket != "" {
                    fmt.print(value_without_bracket);
                }
                structure_color.println(value[value_without_bracket.len()..]);
                continue;
            }
            if value.ends_with("}") || value.ends_with("]") || value.ends_with("},")
                || value.ends_with("],")
            {
                let mut last_bracket_pos = -1;
                let mut i = value.len() - 1;
                while i >= 0 {
                    if value[i] == '}' || value[i] == ']' {
                        last_bracket_pos = i;
                        break;
                    }
                    i -= 1;
                }
                if last_bracket_pos > 0 {
                    let mut value_before_bracket = value[..last_bracket_pos];
                    let mut bracket_part = value[last_bracket_pos..];
                    colorize_value(value_before_bracket);
                    structure_color.println(bracket_part);
                } else {
                    structure_color.println(value);
                }
                continue;
            }
            colorize_value(value);
        } else {
            colorize_value(line);
        }
    }
}
/// Helper function to colorize values based on their type
fn colorize_value(value: &str) {
    let mut trimmed_value = value.trim();
    if trimmed_value.starts_with("\"")
        && (trimmed_value.ends_with("\"") || trimmed_value.ends_with("\","))
    {
        string_color.println(value);
    } else if trimmed_value == "true" || trimmed_value == "false"
        || trimmed_value.ends_with("true,") || trimmed_value.ends_with("false,")
    {
        bool_color.println(value);
    } else if trimmed_value == "null" || trimmed_value.ends_with("null,") {
        null_color.println(value);
    } else if trimmed_value.starts_with("{") || trimmed_value.starts_with("[") {
        structure_color.println(value);
    } else {
        number_color.println(value);
    }
}
pub fn format_and_print_response(
    response: Box<dyn std::any::Any>,
) -> anyhow::Result<()> {
    let mut pretty_json = serde_json::to_string_pretty(&response)?;
    colorize_and_print_json(String::from(pretty_json));
    Ok(())
}
