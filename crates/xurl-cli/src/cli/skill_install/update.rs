//! `xr skill update` orchestration. Owns the remove-then-reinstall verb for
//! one host and for every host, plus the skip rule that keeps `--all` from
//! creating installations.

use std::fs;
use std::io::Write;

use crate::cli::output::OutputConfig;

use super::{
    ACTION_UPDATE, DestinationStatus, InstallEnvelope, InstallError, InstallMultiEnvelope,
    REASON_NOT_INSTALLED, STATUS_DRY_RUN, STATUS_ERROR, STATUS_OK, STATUS_SKIPPED, SkillEnv,
    SkillHost, compute_install_envelope, emit_envelope, emit_missing_host_envelope,
    format_clone_command, host_envelope_str, render_envelope, render_multi, resolve_host,
};

/// Resolve one host's update outcome without emitting it. Removes the existing
/// destination and re-runs the install unless `dry_run` is set, so the returned
/// envelope reports what actually happened.
///
/// The `action` field is `"skill-update"` so a consumer can tell update from
/// install at the JSON layer. Every other field mirrors the install envelope,
/// which is what lets [`run_update_multi`] aggregate hosts through the same
/// [`InstallMultiEnvelope`] that `skill install --all` uses.
fn compute_update_envelope(
    host: SkillHost,
    dry_run: bool,
    skill_env: &SkillEnv,
) -> InstallEnvelope {
    let (url, dest_template) = resolve_host(host);
    let host_str = host_envelope_str(host);

    let dest = match skill_env.destination(host) {
        Ok(p) => p,
        Err(InstallError::MissingHome) => {
            // Without a home the destination cannot be resolved, so the template
            // and its literal `~` are the most specific thing to report.
            return InstallEnvelope {
                action: ACTION_UPDATE,
                host: host_str,
                install_dir: dest_template.to_string(),
                command_preview: format_clone_command(url, dest_template),
                destination_status: DestinationStatus::Absent.as_envelope_str(),
                status: STATUS_ERROR,
                would_succeed: if dry_run { Some(false) } else { None },
                exit_code: Some(1),
                reason: Some(InstallError::MissingHome.reason()),
            };
        }
        Err(_) => unreachable!("expand_tilde_with only emits MissingHome"),
    };

    let dest_display = dest.display().to_string();
    let command_preview = format_clone_command(url, &dest_display);
    let dest_exists = dest.exists();
    let dest_label = if dest_exists {
        DestinationStatus::NonEmptyDir.as_envelope_str()
    } else {
        DestinationStatus::Absent.as_envelope_str()
    };

    if dry_run {
        return InstallEnvelope {
            action: ACTION_UPDATE,
            host: host_str,
            install_dir: dest_display,
            command_preview,
            destination_status: dest_label,
            status: STATUS_DRY_RUN,
            // Update clears the destination first, so an occupied directory is
            // not the blocker it is for a plain install.
            would_succeed: Some(true),
            exit_code: Some(0),
            reason: None,
        };
    }

    if dest_exists && let Err(e) = fs::remove_dir_all(&dest) {
        let _ = e;
        return InstallEnvelope {
            action: ACTION_UPDATE,
            host: host_str,
            install_dir: dest_display,
            command_preview,
            destination_status: dest_label,
            status: STATUS_ERROR,
            would_succeed: None,
            exit_code: Some(1),
            reason: Some("remove-failed"),
        };
    }

    let mut env = compute_install_envelope(host, false, skill_env);
    env.action = ACTION_UPDATE;
    env
}

/// The envelope for a host `--all` passes over because nothing is installed
/// there. Carries exit code 0: a host with no installation is not a failure.
fn skipped_envelope(host: SkillHost, skill_env: &SkillEnv) -> InstallEnvelope {
    let (url, dest_template) = resolve_host(host);
    let install_dir = skill_env
        .destination(host)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| dest_template.to_string());

    InstallEnvelope {
        action: ACTION_UPDATE,
        host: host_envelope_str(host),
        command_preview: format_clone_command(url, &install_dir),
        install_dir,
        destination_status: DestinationStatus::Absent.as_envelope_str(),
        status: STATUS_SKIPPED,
        would_succeed: None,
        exit_code: Some(0),
        reason: Some(REASON_NOT_INSTALLED),
    }
}

/// Whether a host currently has something to update.
fn is_installed(host: SkillHost, skill_env: &SkillEnv) -> bool {
    skill_env.destination(host).is_ok_and(|dest| dest.exists())
}

/// Run the update pipeline for a single host and emit its envelope.
pub fn run_update(
    host: SkillHost,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    skill_env: &SkillEnv,
) -> i32 {
    let env = compute_update_envelope(host, dry_run, skill_env);
    let rendered = render_envelope(&env, &out.format);
    emit_envelope(stdout, &rendered, &out.format);
    if env.status == STATUS_ERROR {
        env.exit_code.unwrap_or(1)
    } else {
        0
    }
}

/// Multi-host wrapper for update. When `all` is set, refreshes every entry in
/// [`KNOWN_HOSTS`](super::KNOWN_HOSTS) that has an existing destination and
/// reports the rest as skipped, aggregating all of them into one
/// [`InstallMultiEnvelope`] exactly as `skill install --all` does. Otherwise
/// dispatches to [`run_update`] for the single host.
pub fn run_update_multi(
    host: Option<SkillHost>,
    all: bool,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    skill_env: &SkillEnv,
) -> i32 {
    if all {
        let mut installations = Vec::with_capacity(SkillHost::ALL.len());
        let mut worst: i32 = 0;
        for h in SkillHost::ALL {
            let env = if is_installed(*h, skill_env) {
                compute_update_envelope(*h, dry_run, skill_env)
            } else {
                skipped_envelope(*h, skill_env)
            };
            if env.status == STATUS_ERROR {
                worst = worst.max(env.exit_code.unwrap_or(1));
            }
            installations.push(env);
        }
        let multi = InstallMultiEnvelope {
            action: ACTION_UPDATE,
            status: if worst == 0 {
                if dry_run { STATUS_DRY_RUN } else { STATUS_OK }
            } else {
                STATUS_ERROR
            },
            installations,
            exit_code: worst,
        };
        let rendered = render_multi(&multi, &out.format);
        let _ = writeln!(stdout, "{rendered}");
        return worst;
    }
    let Some(host) = host else {
        return emit_missing_host_envelope(out, stdout);
    };
    run_update(host, dry_run, out, stdout, skill_env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::output::OutputFormat;
    use crate::cli::skill_install::expand_tilde_with;
    use tempfile::TempDir;

    fn first_host() -> SkillHost {
        SkillHost::ALL[0]
    }

    /// A skill environment with only `HOME`, at `home`.
    fn at(home: &TempDir) -> SkillEnv {
        SkillEnv {
            home: home.path().to_str().map(String::from),
            ..SkillEnv::default()
        }
    }

    #[test]
    fn update_dry_run_reports_the_update_action_and_would_succeed() {
        let home = TempDir::new().expect("tempdir");
        let env = compute_update_envelope(first_host(), true, &at(&home));

        assert_eq!(env.action, "skill-update");
        assert_eq!(env.status, "dry_run");
        assert_eq!(env.would_succeed, Some(true));
        assert_eq!(env.exit_code, Some(0));
        assert_eq!(env.reason, None);
    }

    #[test]
    fn update_dry_run_marks_an_absent_destination_absent() {
        let home = TempDir::new().expect("tempdir");
        let env = compute_update_envelope(first_host(), true, &at(&home));

        assert_eq!(env.destination_status, "absent");
    }

    #[test]
    fn update_without_home_is_an_error_naming_the_reason() {
        let env = compute_update_envelope(first_host(), true, &SkillEnv::default());

        assert_eq!(env.action, "skill-update");
        assert_eq!(env.status, "error");
        assert_eq!(env.reason, Some("home-not-set"));
        assert_eq!(env.exit_code, Some(1));
        assert_eq!(env.would_succeed, Some(false));
    }

    #[test]
    fn a_host_with_no_destination_is_not_installed() {
        let home = TempDir::new().expect("tempdir");
        assert!(!is_installed(first_host(), &at(&home)));
    }

    #[test]
    fn a_host_whose_destination_exists_is_installed() {
        let home = TempDir::new().expect("tempdir");
        let host = first_host();
        let (_, dest_template) = resolve_host(host);
        let dest = expand_tilde_with(dest_template, home.path().to_str())
            .expect("home is set in the test");
        std::fs::create_dir_all(&dest).expect("create destination");

        assert!(is_installed(host, &at(&home)));
    }

    #[test]
    fn skipped_envelope_carries_the_not_installed_reason_and_a_zero_exit() {
        let home = TempDir::new().expect("tempdir");
        let env = skipped_envelope(first_host(), &at(&home));

        assert_eq!(env.action, "skill-update");
        assert_eq!(env.status, "skipped");
        assert_eq!(env.reason, Some("not-installed"));
        assert_eq!(env.exit_code, Some(0));
        assert_eq!(env.destination_status, "absent");
    }

    #[test]
    fn update_all_skips_every_uninstalled_host_and_still_succeeds() {
        let home = TempDir::new().expect("tempdir");
        let out = OutputConfig::new(
            OutputFormat::Json,
            false,
            false,
            crate::cli::ColorChoice::Never,
        );
        let mut stdout = Vec::new();

        let code = run_update_multi(None, true, true, &out, &mut stdout, &at(&home));
        assert_eq!(code, 0);

        let rendered = String::from_utf8(stdout).expect("utf8");
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("one json object");

        assert_eq!(value["action"], "skill-update");
        assert_eq!(value["exit_code"], 0);
        let installations = value["installations"]
            .as_array()
            .expect("installations array");
        assert_eq!(installations.len(), SkillHost::ALL.len());
        for entry in installations {
            assert_eq!(entry["status"], "skipped");
            assert_eq!(entry["reason"], "not-installed");
        }
    }

    #[test]
    fn update_all_refreshes_an_installed_host_and_skips_the_others() {
        let home = TempDir::new().expect("tempdir");
        let installed = first_host();
        let (_, dest_template) = resolve_host(installed);
        let dest = expand_tilde_with(dest_template, home.path().to_str())
            .expect("home is set in the test");
        std::fs::create_dir_all(&dest).expect("create destination");

        let out = OutputConfig::new(
            OutputFormat::Json,
            false,
            false,
            crate::cli::ColorChoice::Never,
        );
        let mut stdout = Vec::new();
        let code = run_update_multi(None, true, true, &out, &mut stdout, &at(&home));
        assert_eq!(code, 0);

        let rendered = String::from_utf8(stdout).expect("utf8");
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("one json object");
        let installations = value["installations"]
            .as_array()
            .expect("installations array");

        let installed_str = host_envelope_str(installed);
        let refreshed = installations
            .iter()
            .find(|e| e["host"] == installed_str)
            .expect("the installed host appears");
        assert_eq!(refreshed["status"], "dry_run");
        assert_eq!(refreshed["reason"], serde_json::Value::Null);

        let skipped = installations.len() - 1;
        assert_eq!(
            installations
                .iter()
                .filter(|e| e["status"] == "skipped")
                .count(),
            skipped
        );
    }

    #[test]
    fn update_all_renders_through_every_structured_format() {
        let home = TempDir::new().expect("tempdir");
        for format in [
            OutputFormat::Json,
            OutputFormat::Jsonl,
            OutputFormat::Yaml,
            OutputFormat::Csv,
            OutputFormat::Tsv,
        ] {
            let out = OutputConfig::new(format, false, false, crate::cli::ColorChoice::Never);
            let mut stdout = Vec::new();
            let code = run_update_multi(None, true, true, &out, &mut stdout, &at(&home));

            assert_eq!(code, 0, "exit code for {:?}", out.format);
            let rendered = String::from_utf8(stdout).expect("utf8");
            assert!(
                rendered.contains("skill-update"),
                "{:?} output names the action: {rendered}",
                out.format
            );
        }
    }
}
