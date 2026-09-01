//! `xr skill install` handler. Bridges the clap-derived `SkillCmd` enum to
//! the `skill_install` module's pipeline.

use std::io::Write;

use crate::cli::SkillCmd;
use crate::output::OutputConfig;
use crate::skill_install;

/// Run the `skill` subcommand. Returns the process exit code.
///
/// `home` is the directory a `~`-prefixed destination expands against,
/// resolved by the caller rather than read from the process here.
pub fn run_skill(
    cmd: SkillCmd,
    out: &OutputConfig,
    stdout: &mut dyn Write,
    home: Option<&str>,
) -> i32 {
    match cmd {
        SkillCmd::Install { host, all, dry_run } => {
            skill_install::run_install_multi(host, all, dry_run, out, stdout, home)
        }
        SkillCmd::Update { host, all, dry_run } => {
            skill_install::run_update_multi(host, all, dry_run, out, stdout, home)
        }
    }
}
