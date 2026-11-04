//! What sits at a skill path, and the swap that replaces a bundle with a fresh
//! clone. A path holds a bundle when it is a clone of the bundle's repository,
//! read from the `origin` git wrote there: that names what the directory came
//! from whatever the repository holds, and a skill someone wrote themselves
//! under the same name has no such remote.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::git::spawn_git_clone;
use super::{DestinationStatus, InstallError, build_clone_command};

/// What [`Occupant::of`] found at a path, read without following a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Occupant {
    /// Nothing is there.
    Absent,
    /// An empty directory, which holds nothing to lose.
    EmptyDir,
    /// A clone of the bundle's repository.
    Bundle,
    /// A directory holding anything else, or a link leading to a directory:
    /// `skill install` never writes a link, so one is someone's own wiring
    /// even when it leads to a bundle.
    NotABundle,
    /// A file, a dangling link, or a device, socket, or fifo.
    NotADirectory,
}

impl Occupant {
    /// Reads what sits at `path`, measured against the bundle's `url`.
    pub(super) fn of(path: &Path, url: &str) -> Self {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Self::Absent,
            Err(_) => return Self::NotABundle,
        };
        if metadata.is_symlink() {
            return if path.is_dir() {
                Self::NotABundle
            } else {
                Self::NotADirectory
            };
        }
        if !metadata.is_dir() {
            return Self::NotADirectory;
        }
        let Ok(mut entries) = fs::read_dir(path) else {
            return Self::NotABundle;
        };
        if entries.next().is_none() {
            return Self::EmptyDir;
        }
        if origin_url(path).is_some_and(|origin| same_repository(&origin, url)) {
            Self::Bundle
        } else {
            Self::NotABundle
        }
    }

    /// Why `skill update` leaves this occupant in place, when it does.
    pub(super) fn refusal(self) -> Option<InstallError> {
        match self {
            Self::Absent | Self::EmptyDir | Self::Bundle => None,
            Self::NotABundle => Some(InstallError::DestNotEmpty),
            Self::NotADirectory => Some(InstallError::DestIsFile),
        }
    }

    /// The occupant as the envelope's `destination_status` names it.
    pub(super) fn status(self) -> DestinationStatus {
        match self {
            Self::Absent => DestinationStatus::Absent,
            Self::EmptyDir => DestinationStatus::EmptyDir,
            Self::Bundle | Self::NotABundle => DestinationStatus::NonEmptyDir,
            Self::NotADirectory => DestinationStatus::File,
        }
    }
}

/// The `url` of the `origin` remote in `dir`'s own `.git/config`. A `.git`
/// file, which a worktree or a submodule has, reads as no origin, since
/// `git clone` never writes one.
fn origin_url(dir: &Path) -> Option<String> {
    let config = fs::read_to_string(dir.join(".git").join("config")).ok()?;
    let mut in_origin = false;
    for line in config.lines().map(str::trim) {
        if line.starts_with('[') {
            in_origin = line == r#"[remote "origin"]"#;
        } else if in_origin
            && let Some((key, value)) = line.split_once('=')
            && key.trim().eq_ignore_ascii_case("url")
        {
            return Some(value.trim().to_string());
        }
    }
    None
}

/// Whether two remote URLs name one repository. GitHub serves a repository
/// under its HTTPS form and both SSH forms, with or without `.git`, and treats
/// the host, owner, and name without regard to case.
fn same_repository(left: &str, right: &str) -> bool {
    match (repository_path(left), repository_path(right)) {
        (Some(left), Some(right)) => left.eq_ignore_ascii_case(&right),
        _ => false,
    }
}

/// `host/owner/name` for an HTTPS or SSH remote URL; `None` for any other
/// form.
fn repository_path(url: &str) -> Option<String> {
    let path = if let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("ssh://git@"))
    {
        rest.to_string()
    } else {
        let (host, rest) = url.strip_prefix("git@")?.split_once(':')?;
        format!("{host}/{rest}")
    };
    let path = path.trim_end_matches('/');
    Some(path.strip_suffix(".git").unwrap_or(path).to_string())
}

/// A directory beside the destination, on its filesystem so a rename out of it
/// is atomic. Removed when dropped, unless it is left holding the only copy.
struct Staging {
    path: PathBuf,
    keep: bool,
}

impl Staging {
    fn beside(dest: &Path) -> io::Result<Self> {
        let name = dest
            .file_name()
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?
            .to_string_lossy();
        let path = dest.with_file_name(format!(".{name}.update-{}", std::process::id()));
        fs::create_dir(&path)?;
        Ok(Self { path, keep: false })
    }

    fn remove(mut self) -> io::Result<()> {
        self.keep = true;
        fs::remove_dir_all(&self.path)
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

/// Replaces the bundle at `dest` with a fresh clone of `url`.
///
/// The clone lands in a staging directory first, so a failure before the swap
/// leaves `dest` as it was. A swap whose second rename fails moves the old
/// copy back. Directories cannot be renamed over one another on every
/// platform, so the swap is two renames, and `dest` is briefly absent between
/// them.
pub(super) fn replace_with_clone(url: &str, dest: &Path) -> Result<(), InstallError> {
    let mut staging = Staging::beside(dest).map_err(|_| InstallError::RemoveFailed)?;
    let clone = staging.path.join("new");
    let replaced = staging.path.join("old");

    spawn_git_clone(&mut build_clone_command(url, &clone))?;

    fs::rename(dest, &replaced).map_err(|_| InstallError::RemoveFailed)?;
    if fs::rename(&clone, dest).is_err() {
        staging.keep = fs::rename(&replaced, dest).is_err();
        return Err(InstallError::RemoveFailed);
    }
    staging.remove().map_err(|_| InstallError::RemoveFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://github.com/brettdavies/xurl-rs-skill.git";

    #[test]
    fn every_form_github_serves_the_repository_under_is_the_same_repository() {
        for form in [
            URL,
            "https://github.com/brettdavies/xurl-rs-skill",
            "https://github.com/brettdavies/xurl-rs-skill/",
            "https://github.com/BrettDavies/xurl-rs-skill.git",
            "git@github.com:brettdavies/xurl-rs-skill.git",
            "git@github.com:brettdavies/xurl-rs-skill",
            "ssh://git@github.com/brettdavies/xurl-rs-skill.git",
            "ssh://git@github.com/brettdavies/xurl-rs-skill",
        ] {
            assert!(same_repository(form, URL), "{form}");
        }
    }

    #[test]
    fn anything_else_is_another_repository() {
        for other in [
            "https://github.com/someone/xurl-rs-skill.git",
            "git@github.com:someone/xurl-rs-skill.git",
            "https://github.com/brettdavies/xurl-rs.git",
            "https://github.com/brettdavies/xurl-rs-skill-extra.git",
            "https://github.com.example.org/brettdavies/xurl-rs-skill.git",
            "https://gitlab.com/brettdavies/xurl-rs-skill.git",
            "http://github.com/brettdavies/xurl-rs-skill.git",
            "file:///home/someone/xurl-rs-skill.git",
            "/home/someone/xurl-rs-skill",
            "",
        ] {
            assert!(!same_repository(other, URL), "{other}");
        }
    }

    fn write_config(dir: &Path, config: &str) {
        fs::create_dir_all(dir.join(".git")).expect("create .git");
        fs::write(dir.join(".git").join("config"), config).expect("write config");
    }

    #[test]
    fn origin_is_read_from_its_own_section() {
        let tmp = tempfile::tempdir().expect("tempdir");
        write_config(
            tmp.path(),
            "[remote \"upstream\"]\n\turl = https://example.org/a.git\n\
             [remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n\
             \turl = https://example.org/b.git\n[branch \"main\"]\n\tremote = origin\n",
        );
        assert_eq!(
            origin_url(tmp.path()).as_deref(),
            Some("https://example.org/b.git")
        );
    }

    #[test]
    fn a_config_with_no_origin_has_no_origin_url() {
        let tmp = tempfile::tempdir().expect("tempdir");
        write_config(tmp.path(), "[remote \"upstream\"]\n\turl = https://a/b\n");
        assert_eq!(origin_url(tmp.path()), None);
    }

    /// The parser against git's own output, not a transcription of it. The
    /// `GIT_DIR` a pre-push hook exports would send these at the repository
    /// being pushed, so they are unset here.
    #[test]
    fn origin_is_read_from_what_git_writes() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let git = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .args(args)
                .current_dir(tmp.path())
                .env_remove("GIT_DIR")
                .env_remove("GIT_WORK_TREE")
                .env_remove("GIT_INDEX_FILE")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_SYSTEM", "/dev/null")
                .status()
                .expect("run git");
            assert!(status.success(), "git {args:?}");
        };
        git(&["init", "--quiet"]);
        git(&["remote", "add", "origin", URL]);

        assert_eq!(origin_url(tmp.path()).as_deref(), Some(URL));
        assert_eq!(Occupant::of(tmp.path(), URL), Occupant::Bundle);
    }

    #[test]
    fn an_empty_directory_and_a_missing_path_hold_nothing_to_lose() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert_eq!(Occupant::of(tmp.path(), URL), Occupant::EmptyDir);
        assert_eq!(
            Occupant::of(&tmp.path().join("nothing"), URL),
            Occupant::Absent
        );
    }

    #[test]
    fn a_clone_that_cannot_stage_leaves_no_staging_behind() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dest = tmp.path().join("no-such-parent").join("xurl-rs");
        assert!(matches!(
            replace_with_clone(URL, &dest),
            Err(InstallError::RemoveFailed)
        ));
        assert!(!dest.parent().expect("a parent").exists());
    }
}
