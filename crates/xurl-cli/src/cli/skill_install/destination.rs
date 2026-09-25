//! Destination resolution and inspection. Owns the pre-clone status
//! taxonomy and the path functions that resolve and vet an install directory.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{InstallError, SkillHost, config_dir_env, resolve_host};

/// The environment a skill destination resolves against, as data.
///
/// A host's own config-directory variable, when set, relocates that host's
/// destination. Otherwise `~` expands against `skill_home`, then `home`. An
/// empty value counts as unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkillEnv {
    /// `HOME`.
    pub home: Option<String>,
    /// `XURL_SKILL_HOME`, which stands in for `HOME` in skill destinations.
    pub skill_home: Option<String>,
    /// The host config-directory variables that are set, by name.
    pub config_dirs: BTreeMap<String, String>,
}

impl SkillEnv {
    /// `host`'s install directory, resolved from its `~`-prefixed destination
    /// template.
    pub fn destination(&self, host: SkillHost) -> Result<PathBuf, InstallError> {
        let template = resolve_host(host).1;
        if let Some((var, replaces)) = config_dir_env(host)
            && let Some(dir) = self.config_dirs.get(var).filter(|dir| !dir.is_empty())
        {
            let rest = template
                .strip_prefix(replaces)
                .and_then(|rest| rest.strip_prefix('/'))
                .expect("build.rs checks that `replaces` is a directory prefix of the template");
            return Ok(Path::new(dir).join(rest));
        }
        let base = self
            .skill_home
            .as_deref()
            .filter(|dir| !dir.is_empty())
            .or(self.home.as_deref());
        expand_tilde_with(template, base)
    }
}

/// Snapshot of what [`check_destination`] found at the resolved path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestinationStatus {
    /// Nothing exists at the resolved path.
    Absent,
    /// An empty directory exists; safe to clone into.
    EmptyDir,
    /// A directory with at least one entry blocks the clone.
    NonEmptyDir,
    /// A regular file (or device / socket / fifo) blocks the clone.
    File,
}

impl DestinationStatus {
    /// Kebab-case identifier for the JSON envelope `destination_status` field.
    pub fn as_envelope_str(self) -> &'static str {
        match self {
            DestinationStatus::Absent => "absent",
            DestinationStatus::EmptyDir => "empty-dir",
            DestinationStatus::NonEmptyDir => "non-empty-dir",
            DestinationStatus::File => "file",
        }
    }
}

/// Expands a leading `~` or `~/` to `home`. Pure passthrough on inputs that
/// do not start with `~`. `MissingHome` only fires when the input actually
/// begins with `~` and `home` is `None` or empty. The caller supplies `home`
/// from the environment it resolved, so nothing here reads the process.
pub fn expand_tilde_with(template: &str, home: Option<&str>) -> Result<PathBuf, InstallError> {
    let needs_home = template == "~" || template.starts_with("~/");
    if !needs_home {
        return Ok(PathBuf::from(template));
    }
    let home = home
        .filter(|s| !s.is_empty())
        .ok_or(InstallError::MissingHome)?;
    if template == "~" {
        return Ok(PathBuf::from(home));
    }
    let rest = template
        .strip_prefix("~/")
        .expect("template starts with ~/ per the branch guard");
    let mut p = PathBuf::from(home);
    p.push(rest);
    Ok(p)
}

/// Canonicalizes the path so a symlinked skills directory resolves to its
/// real target before the check runs. Returns `Absent`/`EmptyDir` on success;
/// `DestIsFile` for a regular file, `DestNotEmpty` for a populated directory.
pub fn check_destination(path: &Path) -> Result<DestinationStatus, InstallError> {
    match path.try_exists() {
        Ok(false) => return Ok(DestinationStatus::Absent),
        Ok(true) => {}
        Err(_) => return Err(InstallError::DestNotEmpty),
    }

    let canonical = fs::canonicalize(path).map_err(|_| InstallError::DestNotEmpty)?;
    let metadata = fs::metadata(&canonical).map_err(|_| InstallError::DestNotEmpty)?;

    if metadata.is_file() {
        return Err(InstallError::DestIsFile);
    }

    if metadata.is_dir() {
        let mut entries = fs::read_dir(&canonical).map_err(|_| InstallError::DestNotEmpty)?;
        if entries.next().is_some() {
            return Err(InstallError::DestNotEmpty);
        }
        return Ok(DestinationStatus::EmptyDir);
    }

    // Block / char devices, sockets, fifos — treat as a file conflict.
    Err(InstallError::DestIsFile)
}

#[cfg(test)]
mod tests {
    use super::super::CONFIG_DIR_VARS;
    use super::*;

    #[test]
    fn expand_tilde_replaces_leading_tilde_slash_with_home() {
        let got = expand_tilde_with("~/.claude/skills/xurl-rs", Some("/home/test"))
            .expect("HOME present + ~/ prefix should expand cleanly");
        assert_eq!(got, PathBuf::from("/home/test/.claude/skills/xurl-rs"));
    }

    #[test]
    fn expand_tilde_missing_home_only_when_input_starts_with_tilde() {
        let err = expand_tilde_with("~/anything", None)
            .expect_err("HOME unset + tilde input should be MissingHome");
        assert_eq!(err, InstallError::MissingHome);

        let err_empty =
            expand_tilde_with("~", Some("")).expect_err("HOME empty string is treated as unset");
        assert_eq!(err_empty, InstallError::MissingHome);
    }

    #[test]
    fn expand_tilde_no_tilde_passthrough() {
        let got = expand_tilde_with("/abs/path", Some("/home/test"))
            .expect("non-tilde input never errors");
        assert_eq!(got, PathBuf::from("/abs/path"));

        let got_no_home =
            expand_tilde_with("/abs/path", None).expect("non-tilde input ignores HOME");
        assert_eq!(got_no_home, PathBuf::from("/abs/path"));
    }

    #[test]
    fn check_destination_absent_for_nonexistent_path() {
        let tmp = tempfile::tempdir().expect("tempdir creation");
        let target = tmp.path().join("does-not-exist");
        let status = check_destination(&target).expect("absent path should be Ok(Absent)");
        assert_eq!(status, DestinationStatus::Absent);
    }

    #[test]
    fn check_destination_empty_dir() {
        let tmp = tempfile::tempdir().expect("tempdir creation");
        let status = check_destination(tmp.path()).expect("empty tempdir should be Ok(EmptyDir)");
        assert_eq!(status, DestinationStatus::EmptyDir);
    }

    #[test]
    fn check_destination_non_empty_dir_errors() {
        let tmp = tempfile::tempdir().expect("tempdir creation");
        std::fs::write(tmp.path().join("placeholder"), b"x").expect("write placeholder");
        let err = check_destination(tmp.path()).expect_err("populated dir should be DestNotEmpty");
        assert_eq!(err, InstallError::DestNotEmpty);
    }

    #[test]
    fn check_destination_regular_file_errors() {
        let tmp = tempfile::tempdir().expect("tempdir creation");
        let target = tmp.path().join("a-file");
        std::fs::write(&target, b"contents").expect("write file");
        let err = check_destination(&target).expect_err("file should be DestIsFile");
        assert_eq!(err, InstallError::DestIsFile);
    }

    /// The first host that documents a config-dir variable, with its variable.
    fn host_with_var() -> (SkillHost, &'static str, &'static str) {
        SkillHost::ALL
            .iter()
            .find_map(|&host| config_dir_env(host).map(|(var, replaces)| (host, var, replaces)))
            .expect("a host with a config-dir variable")
    }

    fn env(home: Option<&str>, skill_home: Option<&str>, dirs: &[(&str, &str)]) -> SkillEnv {
        SkillEnv {
            home: home.map(String::from),
            skill_home: skill_home.map(String::from),
            config_dirs: dirs
                .iter()
                .map(|(var, dir)| ((*var).to_string(), (*dir).to_string()))
                .collect(),
        }
    }

    #[test]
    fn a_host_config_dir_resolves_without_home_and_wins_over_skill_home() {
        let (host, var, replaces) = host_with_var();
        let template = resolve_host(host).1;
        let rest = &template[replaces.len() + 1..];
        let got = env(None, Some("/skill"), &[(var, "/cfg")])
            .destination(host)
            .expect("a set host variable needs no home");
        assert_eq!(got, Path::new("/cfg").join(rest));
    }

    #[test]
    fn skill_home_resolves_without_home_and_an_empty_one_falls_back_to_home() {
        let (host, _, _) = host_with_var();
        let template = resolve_host(host).1;
        let rest = template.strip_prefix("~/").expect("~/ template");
        let with_skill_home = env(None, Some("/skill"), &[]).destination(host);
        assert_eq!(with_skill_home, Ok(Path::new("/skill").join(rest)));
        let empty_skill_home = env(Some("/home"), Some(""), &[]).destination(host);
        assert_eq!(empty_skill_home, Ok(Path::new("/home").join(rest)));
    }

    #[test]
    fn nothing_set_is_missing_home() {
        let (host, _, _) = host_with_var();
        let got = SkillEnv::default().destination(host);
        assert_eq!(got, Err(InstallError::MissingHome));
    }

    #[test]
    fn a_host_without_a_variable_ignores_every_other_hosts_variable() {
        let Some(host) = SkillHost::ALL
            .iter()
            .copied()
            .find(|&host| config_dir_env(host).is_none())
        else {
            return;
        };
        let template = resolve_host(host).1;
        let all_vars: Vec<(&str, &str)> = CONFIG_DIR_VARS
            .iter()
            .map(|var| (*var, "/elsewhere"))
            .collect();
        let got = env(Some("/home"), None, &all_vars).destination(host);
        let rest = template.strip_prefix("~/").expect("~/ template");
        assert_eq!(got, Ok(Path::new("/home").join(rest)));
    }
}
