//! Envelope rendering for the install and update verbs. Owns the per-format
//! serialization of the single-host and multi-host envelopes and the writer.

use std::io::Write;

use crate::output::OutputFormat;

use super::{InstallEnvelope, InstallMultiEnvelope, STATUS_DRY_RUN, STATUS_OK};

/// Render a single install envelope per the active output format.
///
/// Json/Jsonl pretty-print JSON. Ndjson emits one compact JSON line. Yaml
/// emits a YAML document. Csv/Tsv fall back to one compact JSON line because
/// the envelope is nested by construction. Text mode emits the legacy
/// human-readable summary.
pub(super) fn render_envelope(env: &InstallEnvelope, format: &OutputFormat) -> String {
    if format.is_structured() {
        return render_structured(env, format);
    }
    match env.status {
        STATUS_DRY_RUN => env.command_preview.clone(),
        STATUS_OK => format!("Installed xurl-rs skill bundle into {}", env.install_dir),
        _ => {
            let reason = env.reason.unwrap_or("unknown");
            format!("error: {reason}: {}", env.install_dir)
        }
    }
}

/// Render a multi-host envelope per the active output format.
pub(super) fn render_multi(env: &InstallMultiEnvelope, format: &OutputFormat) -> String {
    if format.is_structured() {
        return render_structured(env, format);
    }
    let mut out = String::new();
    for inst in &env.installations {
        out.push_str(&render_envelope(inst, format));
        out.push('\n');
    }
    // Trim trailing newline — the writer adds its own.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Best-effort serializer for any of the structured formats.
pub(super) fn render_structured<T: serde::Serialize>(env: &T, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json | OutputFormat::Jsonl => serde_json::to_string_pretty(env)
            .unwrap_or_else(|_| "{\"status\":\"error\"}".to_string()),
        OutputFormat::Ndjson | OutputFormat::Csv | OutputFormat::Tsv => {
            serde_json::to_string(env).unwrap_or_else(|_| "{\"status\":\"error\"}".to_string())
        }
        OutputFormat::Yaml => serde_yaml::to_string(env)
            .unwrap_or_else(|_| "status: error\n".to_string())
            .trim_end()
            .to_string(),
        OutputFormat::Text => unreachable!("guarded by is_structured()"),
    }
}

pub(super) fn emit_envelope(stdout: &mut dyn Write, rendered: &str, _format: &OutputFormat) {
    let _ = writeln!(stdout, "{rendered}");
}
