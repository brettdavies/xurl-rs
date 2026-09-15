//! `xr skill update` orchestration. Owns the remove-then-reinstall verb for
//! one host and for every host, plus its dry-run and error envelopes.

use std::fs;
use std::io::Write;

use crate::output::OutputConfig;

use super::{
    ACTION_INSTALL, ACTION_UPDATE, InstallError, STATUS_DRY_RUN, STATUS_ERROR, STATUS_OK,
    SkillHost, compute_install_envelope, emit_envelope, emit_missing_host_envelope,
    expand_tilde_with, host_envelope_str, render_envelope, resolve_host,
};

/// Orchestrate the update pipeline for a single host. Semantics: remove the
/// existing destination (if present) and re-run the install. When `dry_run`
/// is set, emit a `skill-update` envelope describing the resolved operation
/// without touching the filesystem or spawning `git`.
///
/// The envelope `action` field is `"skill-update"` so consumers can tell
/// update from install at the JSON layer. All other fields mirror the
/// install envelope shape.
pub fn run_update(
    host: SkillHost,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    home: Option<&str>,
) -> i32 {
    let (_, dest_template) = resolve_host(host);
    let host_str = host_envelope_str(host);

    let dest = match expand_tilde_with(dest_template, home) {
        Ok(p) => p,
        Err(InstallError::MissingHome) => {
            let rendered = render_update_error(host_str, dest_template, dry_run, "home-not-set");
            emit_envelope(stdout, &rendered, &out.format);
            return 1;
        }
        Err(_) => unreachable!("expand_tilde only emits MissingHome"),
    };

    if dry_run {
        let rendered = render_update_dry_run(host_str, &dest.display().to_string(), dest.exists());
        emit_envelope(stdout, &rendered, &out.format);
        return 0;
    }

    if dest.exists()
        && let Err(e) = fs::remove_dir_all(&dest)
    {
        let rendered = render_update_error(
            host_str,
            &dest.display().to_string(),
            false,
            &format!("remove-failed: {e}"),
        );
        emit_envelope(stdout, &rendered, &out.format);
        return 1;
    }

    let env = compute_install_envelope(host, false, home);
    let mut rendered = render_envelope(&env, &out.format);
    if env.status == STATUS_OK {
        rendered = rendered.replace(ACTION_INSTALL, ACTION_UPDATE);
    }
    let _ = writeln!(stdout, "{rendered}");
    if env.status == STATUS_ERROR {
        env.exit_code.unwrap_or(1)
    } else {
        0
    }
}

/// Multi-host wrapper for update. When `all` is set, updates every entry in
/// [`KNOWN_HOSTS`](super::KNOWN_HOSTS) whose destination currently exists,
/// skipping hosts that aren't installed. Otherwise dispatches to
/// [`run_update`] for the single host.
pub fn run_update_multi(
    host: Option<SkillHost>,
    all: bool,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    home: Option<&str>,
) -> i32 {
    if all {
        use clap::ValueEnum as _;
        let mut worst: i32 = 0;
        for h in SkillHost::value_variants() {
            let code = run_update(*h, dry_run, out, stdout, home);
            if code != 0 {
                worst = worst.max(code);
            }
        }
        return worst;
    }
    let Some(host) = host else {
        return emit_missing_host_envelope(out, stdout);
    };
    run_update(host, dry_run, out, stdout, home)
}

fn render_update_dry_run(host: &str, install_dir: &str, dest_exists: bool) -> String {
    let json = serde_json::json!({
        "action": ACTION_UPDATE,
        "host": host,
        "install_dir": install_dir,
        "destination_status": if dest_exists { "non-empty-dir" } else { "absent" },
        "status": STATUS_DRY_RUN,
        "would_succeed": true,
        "exit_code": 0,
    });
    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{\"status\":\"error\"}".to_string())
}

fn render_update_error(host: &str, install_dir: &str, dry_run: bool, reason: &str) -> String {
    let json = serde_json::json!({
        "action": ACTION_UPDATE,
        "host": host,
        "install_dir": install_dir,
        "status": STATUS_ERROR,
        "would_succeed": if dry_run { Some(false) } else { None },
        "exit_code": 1,
        "reason": reason,
    });
    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{\"status\":\"error\"}".to_string())
}
