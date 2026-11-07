//! `xr skill update` orchestration. Owns the replace-with-a-fresh-clone verb
//! for one host and for every host, plus the skip rule that keeps `--all` from
//! creating installations.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::cli::output::OutputConfig;

use super::bundle::{Occupant, replace_with_clone};
use super::git::spawn_git_clone;
use super::{
    ACTION_UPDATE, DestinationStatus, InstallEnvelope, InstallError, InstallMultiEnvelope,
    REASON_NOT_INSTALLED, STATUS_DRY_RUN, STATUS_ERROR, STATUS_OK, STATUS_SKIPPED, SkillEnv,
    SkillHost, build_clone_command, emit_envelope, emit_missing_host_envelope,
    format_clone_command, host_envelope_str, refused_status, render_envelope, render_multi,
    resolve_host,
};

/// Resolve one host's update outcome without emitting it. Replaces the bundle
/// at the destination with a fresh clone, and removes any copy at the host's
/// legacy location, unless `dry_run` is set, so the returned envelope reports
/// what actually happened. Anything at either path other than a bundle is
/// refused before a write, in a dry run and a real run alike.
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
    update_envelope(host, dry_run, skill_env)
        .with_next_step()
        .with_message()
}

fn update_envelope(host: SkillHost, dry_run: bool, skill_env: &SkillEnv) -> InstallEnvelope {
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
                status: refused_status(dry_run),
                would_succeed: if dry_run { Some(false) } else { None },
                exit_code: Some(1),
                reason: Some(InstallError::MissingHome.reason()),
                legacy_install_dir: None,
                refused_dir: None,
                message: None,
                next_step: None,
            };
        }
        Err(_) => unreachable!("expand_tilde_with only emits MissingHome"),
    };

    let dest_display = dest.display().to_string();
    let occupant = Occupant::of(&dest, url);
    let legacy = skill_env.legacy_copy(host);
    let base = InstallEnvelope {
        action: ACTION_UPDATE,
        host: host_str,
        command_preview: format_clone_command(url, &dest_display),
        install_dir: dest_display,
        destination_status: occupant.status().as_envelope_str(),
        status: STATUS_OK,
        would_succeed: None,
        exit_code: Some(0),
        reason: None,
        legacy_install_dir: legacy.as_ref().map(|path| path.display().to_string()),
        refused_dir: None,
        message: None,
        next_step: None,
    };

    let refusal = occupant
        .refusal()
        .map(|error| (error, dest.as_path()))
        .or_else(|| {
            let legacy = legacy.as_deref()?;
            let error = Occupant::of(legacy, url).refusal()?;
            Some((error, legacy))
        });
    if let Some((error, dir)) = refusal {
        return InstallEnvelope {
            status: refused_status(dry_run),
            would_succeed: dry_run.then_some(false),
            exit_code: Some(error.exit_code()),
            reason: Some(error.reason()),
            refused_dir: Some(dir.display().to_string()),
            message: Some(refusal_message(dir, host_str)),
            ..base
        };
    }

    if dry_run {
        return InstallEnvelope {
            status: STATUS_DRY_RUN,
            would_succeed: Some(true),
            ..base
        };
    }

    let replaced = if occupant == Occupant::Bundle {
        replace_with_clone(url, &dest)
    } else {
        spawn_git_clone(&mut build_clone_command(url, &dest))
    };
    match replaced.and_then(|()| remove_legacy_copy(legacy.as_deref())) {
        Ok(()) => base,
        Err(error) => InstallEnvelope {
            status: STATUS_ERROR,
            exit_code: Some(error.exit_code()),
            reason: Some(error.reason()),
            ..base
        },
    }
}

/// The sentence a refusal carries, the same for a dry run and a real run.
fn refusal_message(dir: &Path, host: &str) -> String {
    format!(
        "{} holds something other than the xurl-rs skill bundle, so nothing was changed. \
         Move it elsewhere, then run `xr skill update {host}` again.",
        dir.display()
    )
}

/// Removes the copy at the host's legacy location, once the clone at its
/// destination is in place.
fn remove_legacy_copy(legacy: Option<&Path>) -> Result<(), InstallError> {
    legacy.map_or(Ok(()), |dir| {
        fs::remove_dir_all(dir).map_err(|_| InstallError::RemoveFailed)
    })
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
        legacy_install_dir: None,
        refused_dir: None,
        message: None,
        next_step: None,
    }
    .with_next_step()
}

/// Whether a host currently has something to update, at its destination or
/// its legacy location.
fn is_installed(host: SkillHost, skill_env: &SkillEnv) -> bool {
    skill_env.destination(host).is_ok_and(|dest| dest.exists())
        || skill_env.legacy_copy(host).is_some()
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
        let installations = SkillHost::ALL
            .iter()
            .map(|h| {
                if is_installed(*h, skill_env) {
                    compute_update_envelope(*h, dry_run, skill_env)
                } else {
                    skipped_envelope(*h, skill_env)
                }
            })
            .collect();
        let multi = InstallMultiEnvelope::of(ACTION_UPDATE, dry_run, installations);
        let rendered = render_multi(&multi, &out.format);
        let _ = writeln!(stdout, "{rendered}");
        return multi.process_exit_code();
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
    use crate::cli::skill_install::{expand_tilde_with, legacy_destination};
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

    /// The first host with a legacy location, with a copy created there under
    /// `home`.
    fn host_with_legacy_copy(home: &TempDir) -> (SkillHost, std::path::PathBuf) {
        let host = SkillHost::ALL
            .iter()
            .copied()
            .find(|&host| legacy_destination(host).is_some())
            .expect("a host with a legacy location");
        let legacy = expand_tilde_with(
            legacy_destination(host).expect("found above"),
            home.path().to_str(),
        )
        .expect("home is set in the test");
        std::fs::create_dir_all(&legacy).expect("create the legacy copy");
        (host, legacy)
    }

    #[test]
    fn a_legacy_copy_counts_as_installed_and_a_dry_run_names_it() {
        let home = TempDir::new().expect("tempdir");
        let (host, legacy) = host_with_legacy_copy(&home);

        assert!(is_installed(host, &at(&home)));
        let env = compute_update_envelope(host, true, &at(&home));
        assert_eq!(env.legacy_install_dir.as_deref(), legacy.to_str());
        assert!(legacy.exists(), "a dry run removes nothing");
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
    fn update_dry_run_without_home_names_the_reason_the_real_run_fails_for() {
        let env = compute_update_envelope(first_host(), true, &SkillEnv::default());

        assert_eq!(env.action, "skill-update");
        assert_eq!(env.status, "dry_run");
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
