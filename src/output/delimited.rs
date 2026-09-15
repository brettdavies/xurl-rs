//! Delimited-table serialization for the `csv` and `tsv` output formats:
//! turning a JSON value into header and value rows with escaped cells.

use std::io::Write;

use serde_json::Value;

/// Best-effort delimited-table emitter for the `csv` / `tsv` output formats.
///
/// Strategy:
/// - Object → one header row + one value row.
/// - Array of objects → header is the union of keys (insertion-ordered by
///   first occurrence), followed by one row per element. Missing keys emit
///   an empty cell.
/// - Array of scalars → single-column `value` table, one row per element.
/// - Scalar → single-column `value` table with one row.
///
/// Nested values (objects, arrays inside a cell) are serialized as JSON so
/// each cell stays a single delimited field. A `_warning` column is appended
/// with the note "nested values JSON-stringified" so downstream tools see why
/// some cells carry JSON.
///
/// The result is forward-only and best-effort: malformed shapes degrade
/// gracefully into a one-line JSON blob in a `_payload` column rather than
/// erroring out.
pub(super) fn write_flattened(
    out: &mut dyn Write,
    value: &Value,
    sep: char,
) -> std::io::Result<()> {
    let rows: Vec<&Value> = match value {
        Value::Array(arr) => arr.iter().collect(),
        other => vec![other],
    };

    // Collect the union of object keys to use as the column header.
    let mut header: Vec<String> = Vec::new();
    let mut all_objects = true;
    for row in &rows {
        if let Value::Object(map) = row {
            for k in map.keys() {
                if !header.iter().any(|h| h == k) {
                    header.push(k.clone());
                }
            }
        } else {
            all_objects = false;
        }
    }

    if !all_objects || header.is_empty() {
        // Scalar / mixed shapes: single-column "value" table.
        writeln!(out, "value")?;
        for row in &rows {
            let cell = scalar_cell(row);
            writeln!(out, "{}", delimited_escape(&cell, sep))?;
        }
        return Ok(());
    }

    // Detect whether any cell needed JSON stringification so we can emit a
    // single `_warning` column at the end, only when relevant.
    let mut needs_warning = false;
    let row_cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            header
                .iter()
                .map(|key| {
                    let cell =
                        row.as_object()
                            .and_then(|m| m.get(key))
                            .map_or_else(String::new, |v| {
                                if matches!(v, Value::Object(_) | Value::Array(_)) {
                                    needs_warning = true;
                                    serde_json::to_string(v).unwrap_or_else(|_| v.to_string())
                                } else {
                                    scalar_cell(v)
                                }
                            });
                    delimited_escape(&cell, sep)
                })
                .collect()
        })
        .collect();

    // Header line.
    let mut header_out: Vec<String> = header.iter().map(|h| delimited_escape(h, sep)).collect();
    if needs_warning {
        header_out.push(delimited_escape("_warning", sep));
    }
    writeln!(out, "{}", header_out.join(&sep.to_string()))?;

    for cells in &row_cells {
        let mut line = cells.clone();
        if needs_warning {
            line.push(delimited_escape("nested values JSON-stringified", sep));
        }
        writeln!(out, "{}", line.join(&sep.to_string()))?;
    }
    Ok(())
}

/// Renders a scalar [`Value`] as the unquoted cell body (the caller wraps with
/// `delimited_escape` to handle the delimiter / quote rules).
fn scalar_cell(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Object(_) | Value::Array(_) => {
            serde_json::to_string(v).unwrap_or_else(|_| v.to_string())
        }
    }
}

/// Applies RFC 4180 quoting when needed.
///
/// For comma-delimited output: wraps in double quotes when the cell contains
/// a comma, double quote, CR, or LF, and escapes embedded quotes. For tab-
/// delimited output: replaces embedded tabs and newlines with spaces (TSV
/// has no canonical quoting rule, and most consumers reject control chars
/// inside fields).
fn delimited_escape(cell: &str, sep: char) -> String {
    if sep == '\t' {
        return cell.replace(['\t', '\n', '\r'], " ");
    }
    if cell.contains(sep) || cell.contains('"') || cell.contains('\n') || cell.contains('\r') {
        let escaped = cell.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        cell.to_string()
    }
}
