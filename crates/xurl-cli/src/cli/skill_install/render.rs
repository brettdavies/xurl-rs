//! Envelope rendering for the install and update verbs. Owns the per-format
//! serialization of the single-host and multi-host envelopes and the writer.

use std::io::Write;

use crate::cli::output::OutputFormat;

use super::{
    ACTION_UPDATE, DestinationStatus, InstallEnvelope, InstallMultiEnvelope, STATUS_DRY_RUN,
    STATUS_OK,
};

/// Render a single install envelope per the active output format.
///
/// Json pretty-prints JSON. Jsonl and Ndjson emit one compact JSON line. Yaml
/// emits a YAML document. Csv/Tsv fall back to one compact JSON line because
/// the envelope is nested by construction. Text mode emits a human-readable
/// summary, then the refusal's sentence when one was refused, a note line when
/// a dry run would replace the bundle at the destination, and a note line when
/// the envelope names a copy at the host's legacy location that update would
/// remove.
pub(super) fn render_envelope(env: &InstallEnvelope, format: &OutputFormat) -> String {
    if format.is_structured() {
        return render_structured(env, format);
    }
    let summary = match env.status {
        STATUS_DRY_RUN => env.command_preview.clone(),
        STATUS_OK => format!("Installed xurl-rs skill bundle into {}", env.install_dir),
        _ => {
            let reason = env.reason.unwrap_or("unknown");
            let dir = env.refused_dir.as_deref().unwrap_or(&env.install_dir);
            format!("error: {reason}: {dir}")
        }
    };
    let mut lines = vec![summary];
    lines.extend(env.message.clone());
    if env.action == ACTION_UPDATE
        && env.status == STATUS_DRY_RUN
        && env.destination_status == DestinationStatus::NonEmptyDir.as_envelope_str()
    {
        lines.push(format!(
            "note: replaces the bundle at {}, along with any file added to it",
            env.install_dir
        ));
    }
    if let Some(legacy) = env
        .legacy_install_dir
        .as_ref()
        .filter(|_| env.refused_dir.is_none())
    {
        lines.push(if env.action == ACTION_UPDATE {
            format!(
                "note: {legacy} holds another copy; update removes it along with the destination"
            )
        } else {
            format!(
                "note: {legacy} holds another copy; `xr skill update {}` removes it",
                env.host
            )
        });
    }
    lines.join("\n")
}

/// The sentence for a skill verb's failure, from the facts its envelope holds.
pub(super) fn failure_message(env: &InstallEnvelope) -> Option<String> {
    let dir = env.refused_dir.as_deref().unwrap_or(&env.install_dir);
    let host = env.host;
    Some(match env.reason? {
        "home-not-set" => format!(
            "{dir} starts at the home directory, and neither HOME nor XURL_SKILL_HOME is set. \
             Set one, or the host's own config-directory variable; `xr --help` lists them."
        ),
        "destination-not-empty" => format!(
            "{dir} already holds files, so nothing was cloned. `xr skill update {host}` replaces \
             a clone of the bundle there and refuses anything else."
        ),
        "destination-is-file" => {
            format!(
                "{dir} is a file, so nothing was cloned. Move it elsewhere, then run this again."
            )
        }
        "git-not-found" => {
            "`git` was not found on PATH, so nothing was cloned. Install it, then run this again."
                .to_string()
        }
        "git-clone-failed" => format!(
            "`git clone` exited with code {}, so {dir} is as it was. Its own error is on stderr.",
            env.exit_code.unwrap_or(1)
        ),
        "remove-failed" => format!(
            "A fresh clone could not be swapped in at {dir}, or the copy it replaced could not be \
             removed. Check the permissions on its parent directory, then run this again."
        ),
        _ => return None,
    })
}

/// Which of `total` hosts failed in an `--all` run, and why.
pub(super) fn hosts_failed_message(failed: &[&InstallEnvelope], total: usize) -> String {
    let hosts: Vec<String> = failed
        .iter()
        .map(|env| format!("{} ({})", env.host, env.reason.unwrap_or("unknown")))
        .collect();
    format!(
        "{} of {total} hosts failed: {}. Each entry in `installations` says what to do about it.",
        failed.len(),
        hosts.join(", ")
    )
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
        OutputFormat::Json => serde_json::to_string_pretty(env)
            .unwrap_or_else(|_| "{\"status\":\"error\"}".to_string()),
        OutputFormat::Jsonl | OutputFormat::Ndjson | OutputFormat::Csv | OutputFormat::Tsv => {
            serde_json::to_string(env).unwrap_or_else(|_| "{\"status\":\"error\"}".to_string())
        }
        OutputFormat::Yaml => xdk::yaml::to_string(env)
            .unwrap_or_else(|_| "status: error\n".to_string())
            .trim_end()
            .to_string(),
        OutputFormat::Text => unreachable!("guarded by is_structured()"),
    }
}

pub(super) fn emit_envelope(stdout: &mut dyn Write, rendered: &str, _format: &OutputFormat) {
    let _ = writeln!(stdout, "{rendered}");
}

#[cfg(test)]
mod tests {
    use super::super::{ACTION_INSTALL, ACTION_UPDATE};
    use super::*;

    fn with_legacy(action: &'static str, status: &'static str) -> InstallEnvelope {
        InstallEnvelope {
            action,
            host: "codex",
            install_dir: "/new/xurl-rs".to_string(),
            command_preview: "git clone --depth 1 URL /new/xurl-rs".to_string(),
            destination_status: "absent",
            status,
            would_succeed: None,
            exit_code: Some(0),
            reason: None,
            legacy_install_dir: Some("/old/xurl-rs".to_string()),
            refused_dir: None,
            message: None,
            next_step: None,
        }
    }

    #[test]
    fn text_names_a_legacy_copy_for_each_verb() {
        let update = render_envelope(&with_legacy(ACTION_UPDATE, STATUS_OK), &OutputFormat::Text);
        assert!(update.contains("/old/xurl-rs"), "{update}");

        let install = render_envelope(
            &with_legacy(ACTION_INSTALL, STATUS_DRY_RUN),
            &OutputFormat::Text,
        );
        assert!(install.contains("/old/xurl-rs"), "{install}");
        assert!(install.contains("xr skill update codex"), "{install}");
    }
}
