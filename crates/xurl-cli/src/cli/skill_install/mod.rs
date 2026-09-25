//! `xr skill install <host>` — clone the xurl-rs skill bundle into a host's
//! canonical skills directory using a build-generated host map and a
//! hardened `git clone` invocation.
//!
//! Pipeline:
//!
//! ```text
//!   clap parse (host, --dry-run, --output)
//!         |
//!         v
//!   resolve_host(SkillHost) -> (url, dest_template)
//!         |
//!         v
//!   skill_env.destination(host)
//!     host config-dir var, else XURL_SKILL_HOME,
//!     else host base-dir var, else HOME
//!         |                   -- none set --> MissingHome (reason=home-not-set)
//!         v
//!      dry_run? --yes--> emit envelope (mode=dry-run, would_succeed)
//!         |
//!         no
//!         v
//!   check_destination()       -- conflict --> envelope(error, reason)
//!    (canonicalize)
//!         |
//!         v
//!   build_clone_command(url, dest) with hardening
//!         |
//!         v
//!      spawn git -- NotFound ---> reason=git-not-found
//!         |     -- nonzero ----> reason=git-clone-failed
//!         v
//!      exit 0 --> success envelope
//! ```
//!
//! Hardening surface:
//! - `GIT_HARDEN_FLAGS` — five `-c key=value` pairs (`credential.helper`,
//!   `core.askPass`, `protocol.allow=never`, `protocol.https.allow=always`,
//!   `http.followRedirects=false`).
//! - `GIT_HARDEN_ENV_REMOVE` — five env vars stripped before spawn (SSH /
//!   proxy / askpass / exec-path overrides).
//! - `GIT_HARDEN_ENV_SET` — `GIT_CONFIG_GLOBAL=/dev/null`,
//!   `GIT_CONFIG_SYSTEM=/dev/null`, `GIT_TERMINAL_PROMPT=0` — disable every
//!   layer of user-controlled git config and block credential prompts.
//!
//! Mirrors `agentnative-cli/src/skill_install.rs` so the two CLIs reject the
//! same malformed inputs and apply identical hardening.

use std::io::Write;

use crate::cli::output::OutputConfig;

mod destination;
mod git;
mod render;
mod update;

pub use destination::{DestinationStatus, SkillEnv, check_destination, expand_tilde_with};
pub use git::{
    GIT_HARDEN_ENV_REMOVE, GIT_HARDEN_ENV_SET, GIT_HARDEN_FLAGS, build_clone_command,
    format_clone_command,
};
pub use update::{run_update, run_update_multi};

use git::spawn_git_clone;
use render::{emit_envelope, render_envelope, render_multi, render_structured};

// `SkillHost`, `KNOWN_HOSTS`, `resolve_host`, `host_envelope_str`,
// `config_dir_env`, `CONFIG_DIR_VARS`, `base_dir_env`, and `BASE_DIR_VARS` are
// auto-generated at build time from `src/cli/skill_install/skill.json`. Edit the
// JSON file to add or remove hosts or change a host's variables; `cargo build`
// regenerates this file.
#[allow(missing_docs)]
mod generated_hosts {
    include!(concat!(env!("OUT_DIR"), "/generated_hosts.rs"));
}

pub use generated_hosts::{
    BASE_DIR_VARS, CONFIG_DIR_VARS, KNOWN_HOSTS, SkillHost, base_dir_env, config_dir_env,
    host_envelope_str, resolve_host,
};

/// Typed install error — closed set matching the envelope `reason` taxonomy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallError {
    /// Neither the host's config- or base-directory variable,
    /// `XURL_SKILL_HOME`, nor `$HOME` is set; cannot expand a `~` destination
    /// template.
    MissingHome,
    /// The resolved destination already holds files.
    DestNotEmpty,
    /// The resolved destination path exists as a regular file (or device /
    /// socket / fifo).
    DestIsFile,
    /// `git` was not found on `$PATH` when the spawn was attempted.
    GitNotFound,
    /// `git clone` exited with a non-zero status. `code` carries the
    /// observed exit code (1 when the process terminated without one).
    GitCloneFailed {
        /// Exit code reported by the failed `git clone` invocation.
        code: i32,
    },
}

impl InstallError {
    /// Kebab-case identifier for the envelope `reason` field.
    pub fn reason(&self) -> &'static str {
        match self {
            InstallError::MissingHome => "home-not-set",
            InstallError::DestNotEmpty => "destination-not-empty",
            InstallError::DestIsFile => "destination-is-file",
            InstallError::GitNotFound => "git-not-found",
            InstallError::GitCloneFailed { .. } => "git-clone-failed",
        }
    }
}

/// Result envelope shared by both `--output text` and `--output json`.
/// Uniform across success and error paths per the project envelope rules.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct InstallEnvelope {
    /// Action discriminator: `"skill-install"` or `"skill-update"`.
    pub action: &'static str,
    /// Target host slug (e.g. `"claude_code"`).
    pub host: &'static str,
    /// Resolved destination path, with `~` expanded or a host config- or
    /// base-directory variable applied.
    pub install_dir: String,
    /// Human-visible `git clone` command (hardening flags omitted).
    pub command_preview: String,
    /// Pre-clone state of `install_dir` per [`DestinationStatus`].
    pub destination_status: &'static str,
    /// Outcome discriminator: `"ok"`, `"error"`, or `"dry_run"`.
    pub status: &'static str,
    /// Dry-run-only: would the install have succeeded?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub would_succeed: Option<bool>,
    /// Process exit code for the install run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Kebab-case error reason from [`InstallError::reason`]; `None` on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
}

/// Multi-host envelope for `--all` invocations.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct InstallMultiEnvelope {
    /// Action discriminator: `"skill-install"` or `"skill-update"`.
    pub action: &'static str,
    /// Aggregate outcome: `"ok"`, `"error"`, or `"dry_run"`.
    pub status: &'static str,
    /// One [`InstallEnvelope`] per host, in [`KNOWN_HOSTS`] order.
    pub installations: Vec<InstallEnvelope>,
    /// Worst exit code observed across [`Self::installations`].
    pub exit_code: i32,
}

const ACTION_INSTALL: &str = "skill-install";
const ACTION_UPDATE: &str = "skill-update";
const STATUS_DRY_RUN: &str = "dry_run";
const STATUS_OK: &str = "ok";
const STATUS_ERROR: &str = "error";
const STATUS_SKIPPED: &str = "skipped";

/// Reason paired with [`STATUS_SKIPPED`] when `skill update --all` passes over a
/// host whose destination does not exist. Update refreshes what is installed;
/// installing everywhere is what `skill install --all` is for.
const REASON_NOT_INSTALLED: &str = "not-installed";

/// Compute the envelope without performing I/O (dry-run) or, in install mode,
/// after spawning `git`.
pub fn compute_install_envelope(
    host: SkillHost,
    dry_run: bool,
    skill_env: &SkillEnv,
) -> InstallEnvelope {
    let (url, dest_template) = resolve_host(host);
    let host_str = host_envelope_str(host);

    // Step 1: resolve the destination. MissingHome surfaces as an envelope error.
    let dest = match skill_env.destination(host) {
        Ok(p) => p,
        Err(InstallError::MissingHome) => {
            // Without a home we cannot show the resolved destination. Surface
            // the template (with its literal `~`) and the matching command.
            let command_preview = format_clone_command(url, dest_template);
            return InstallEnvelope {
                action: ACTION_INSTALL,
                host: host_str,
                install_dir: dest_template.to_string(),
                command_preview,
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

    // Step 2: destination check.
    let dest_status = match check_destination(&dest) {
        Ok(s) => s,
        Err(e @ InstallError::DestIsFile) | Err(e @ InstallError::DestNotEmpty) => {
            let status_label = match e {
                InstallError::DestIsFile => DestinationStatus::File,
                InstallError::DestNotEmpty => DestinationStatus::NonEmptyDir,
                _ => unreachable!(),
            };
            return InstallEnvelope {
                action: ACTION_INSTALL,
                host: host_str,
                install_dir: dest_display,
                command_preview,
                destination_status: status_label.as_envelope_str(),
                status: STATUS_ERROR,
                would_succeed: if dry_run { Some(false) } else { None },
                exit_code: Some(1),
                reason: Some(e.reason()),
            };
        }
        Err(_) => unreachable!("check_destination only emits DestIsFile / DestNotEmpty"),
    };

    let dest_status_str = dest_status.as_envelope_str();

    if dry_run {
        return InstallEnvelope {
            action: ACTION_INSTALL,
            host: host_str,
            install_dir: dest_display,
            command_preview,
            destination_status: dest_status_str,
            status: STATUS_DRY_RUN,
            would_succeed: Some(true),
            exit_code: Some(0),
            reason: None,
        };
    }

    // Step 3: spawn `git`.
    let mut cmd = build_clone_command(url, &dest);
    match spawn_git_clone(&mut cmd) {
        Ok(()) => InstallEnvelope {
            action: ACTION_INSTALL,
            host: host_str,
            install_dir: dest_display,
            command_preview,
            destination_status: dest_status_str,
            status: STATUS_OK,
            would_succeed: None,
            exit_code: Some(0),
            reason: None,
        },
        Err(InstallError::GitCloneFailed { code }) => InstallEnvelope {
            action: ACTION_INSTALL,
            host: host_str,
            install_dir: dest_display,
            command_preview,
            destination_status: dest_status_str,
            status: STATUS_ERROR,
            would_succeed: None,
            exit_code: Some(code),
            reason: Some(InstallError::GitCloneFailed { code }.reason()),
        },
        Err(InstallError::GitNotFound) => InstallEnvelope {
            action: ACTION_INSTALL,
            host: host_str,
            install_dir: dest_display,
            command_preview,
            destination_status: dest_status_str,
            status: STATUS_ERROR,
            would_succeed: None,
            exit_code: Some(1),
            reason: Some(InstallError::GitNotFound.reason()),
        },
        Err(_) => unreachable!("spawn_git_clone only emits GitCloneFailed / GitNotFound"),
    }
}

/// Orchestrate the install pipeline for a single host. Writes the envelope to
/// `stdout` and returns the exit code.
pub fn run_install(
    host: SkillHost,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    skill_env: &SkillEnv,
) -> i32 {
    let envelope = compute_install_envelope(host, dry_run, skill_env);
    let rendered = render_envelope(&envelope, &out.format);
    let _ = writeln!(stdout, "{rendered}");
    if envelope.status == STATUS_ERROR {
        envelope.exit_code.unwrap_or(1)
    } else {
        0
    }
}

/// Multi-host wrapper: when `all` is set, iterates every entry in
/// [`KNOWN_HOSTS`]; otherwise dispatches to [`run_install`] for the single
/// host. Per-host failures are reported in the result stream but do not abort
/// the run — the exit code is the worst observed.
pub fn run_install_multi(
    host: Option<SkillHost>,
    all: bool,
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    skill_env: &SkillEnv,
) -> i32 {
    if all {
        return run_for_all_hosts(dry_run, out, stdout, skill_env);
    }
    let Some(host) = host else {
        // Missing host AND missing --all. Emit a hint envelope listing the
        // known hosts and return EXIT_USAGE_ERROR (2).
        return emit_missing_host_envelope(out, stdout);
    };
    run_install(host, dry_run, out, stdout, skill_env)
}

/// Render a per-host envelope sequence as a single multi envelope.
fn run_for_all_hosts(
    dry_run: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    skill_env: &SkillEnv,
) -> i32 {
    let mut installations = Vec::with_capacity(SkillHost::ALL.len());
    let mut worst: i32 = 0;
    for host in SkillHost::ALL {
        let env = compute_install_envelope(*host, dry_run, skill_env);
        if env.status == STATUS_ERROR {
            worst = worst.max(env.exit_code.unwrap_or(1));
        }
        installations.push(env);
    }
    let multi = InstallMultiEnvelope {
        action: ACTION_INSTALL,
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
    worst
}

/// Emit a hint envelope listing the known hosts. Returned when neither a host
/// nor `--all` is supplied.
fn emit_missing_host_envelope(out: &OutputConfig, stdout: &mut dyn Write) -> i32 {
    if out.format.is_structured() {
        let json = serde_json::json!({
            "action": ACTION_INSTALL,
            "status": "error",
            "reason": "missing-host",
            "exit_code": 2,
            "message": "missing target host; pass <host> or --all",
            "known_hosts": KNOWN_HOSTS,
        });
        let _ = writeln!(stdout, "{}", render_structured(&json, &out.format));
    } else {
        let _ = writeln!(
            stdout,
            "error: missing target host; pass <host> or --all\nsupported hosts:"
        );
        for h in KNOWN_HOSTS {
            let _ = writeln!(stdout, "  {h}");
        }
    }
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_host_key_round_trips_through_from_key() {
        for &expected in KNOWN_HOSTS {
            let parsed = SkillHost::from_key(expected)
                .unwrap_or_else(|| panic!("KNOWN_HOSTS entry {expected:?} not parseable"));
            assert_eq!(host_envelope_str(parsed), expected);
        }
        assert_eq!(SkillHost::from_key("no_such_host"), None);
    }

    #[test]
    fn known_hosts_matches_all_in_count_and_order() {
        let variant_names: Vec<&str> = SkillHost::ALL
            .iter()
            .map(|host| host_envelope_str(*host))
            .collect();
        assert_eq!(
            variant_names,
            KNOWN_HOSTS.to_vec(),
            "SkillHost::ALL and KNOWN_HOSTS must stay in lockstep",
        );
    }

    #[test]
    fn resolve_host_returns_expected_pair_for_every_variant() {
        let fixture_text = include_str!("skill.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_text).expect("fixture is valid JSON");
        let install = fixture
            .get("install")
            .and_then(|v| v.as_object())
            .expect("fixture has install map");

        for &host_name in KNOWN_HOSTS {
            let cmd = install
                .get(host_name)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("fixture missing install.{host_name}"));
            let tokens: Vec<&str> = cmd.split_whitespace().collect();
            let expected_url = tokens[4];
            let expected_dest = tokens[5];

            let host = SkillHost::from_key(host_name)
                .unwrap_or_else(|| panic!("KNOWN_HOSTS entry {host_name:?} unparseable"));
            let (url, dest) = resolve_host(host);

            assert_eq!(url, expected_url, "url mismatch for {host_name}");
            assert_eq!(dest, expected_dest, "dest mismatch for {host_name}");
        }
    }

    #[test]
    fn install_error_reasons_match_closed_set() {
        assert_eq!(InstallError::MissingHome.reason(), "home-not-set");
        assert_eq!(InstallError::DestNotEmpty.reason(), "destination-not-empty");
        assert_eq!(InstallError::DestIsFile.reason(), "destination-is-file");
        assert_eq!(InstallError::GitNotFound.reason(), "git-not-found");
        assert_eq!(
            InstallError::GitCloneFailed { code: 128 }.reason(),
            "git-clone-failed"
        );
    }
}
