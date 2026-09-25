//! `xr skill install` handler. Bridges the clap-derived `SkillCmd` enum to
//! the `skill_install` module's pipeline.

use std::io::Write;

use crate::cli::SkillCmd;
use crate::cli::output::OutputConfig;
use crate::cli::skill_install;
use crate::cli::skill_install::SkillEnv;

/// Run the `skill` subcommand. Returns the process exit code.
///
/// `skill_env` is the environment each host's destination resolves against,
/// supplied by the caller rather than read from the process here.
pub fn run_skill(
    cmd: SkillCmd,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    skill_env: &SkillEnv,
) -> i32 {
    match cmd {
        SkillCmd::Install { host, all, dry_run } => {
            skill_install::run_install_multi(host, all, dry_run, out, stdout, skill_env)
        }
        SkillCmd::Update { host, all, dry_run } => {
            skill_install::run_update_multi(host, all, dry_run, out, stdout, skill_env)
        }
    }
}
