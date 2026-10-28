//! Flags only a raw request reads, given ahead of a command word.
//!
//! clap parses root flags only before the command word, and each command
//! reads its own copy of `-u`, `--auth`, and `-t`, so the root's copy would
//! go unread: `xr -u work whoami` would answer as the default login. Such an
//! invocation is refused. When every misplaced flag is one the command
//! takes, the refusal carries the invocation with them after the command
//! word, checked by parsing it again.

use std::ffi::OsString;

use clap::parser::ValueSource;
use clap::{Arg, ArgMatches, Command, CommandFactory};

use crate::cli::Cli;
use crate::cli::classify::{ROOT_COMMAND, raw_only_flag_given};
use crate::cli::hints::shell_word;

/// The root flags only a raw request reads, by clap id.
const RAW_ONLY: [&str; 8] = [
    "method",
    "headers",
    "data",
    "auth_type",
    "username",
    "trace",
    "stream",
    "file",
];

/// The raw-only flags a step may move after the command word. Their values
/// name a login, a scheme, or nothing, so echoing them leaks no secret,
/// where a header or a body could.
const MOVABLE: [&str; 3] = ["auth_type", "username", "trace"];

/// Raw-only flags an invocation gave ahead of its command word.
pub(crate) struct Misplaced {
    /// Each flag in its short spelling, or its long one when it has none,
    /// in the order the root declares them.
    pub(crate) flags: Vec<String>,
    /// The invocation with those flags after the command word, when each is
    /// one the command takes.
    pub(crate) moved: Option<String>,
}

/// The raw-only flags `args` gave ahead of a command word, if any.
pub(crate) fn misplaced(cli: &Cli, args: &[OsString]) -> Option<Misplaced> {
    if cli.command.is_none() || !raw_only_flag_given(cli) {
        return None;
    }
    let root = Cli::command();
    let matches = root.clone().try_get_matches_from(args).ok()?;
    let given: Vec<&Arg> = root
        .get_arguments()
        .filter(|arg| RAW_ONLY.contains(&arg.get_id().as_str()))
        .filter(|arg| matches.value_source(arg.get_id().as_str()) == Some(ValueSource::CommandLine))
        .collect();
    let flags = given.iter().map(|arg| spelling(arg)).collect();
    let moved = given
        .iter()
        .all(|arg| MOVABLE.contains(&arg.get_id().as_str()))
        .then(|| move_after_command(&root, &matches, args))
        .flatten();
    Some(Misplaced { flags, moved })
}

/// How a message names `arg`: `-u`, or `--auth` for a flag with no short.
fn spelling(arg: &Arg) -> String {
    match (arg.get_short(), arg.get_long()) {
        (Some(short), _) => format!("-{short}"),
        (None, Some(long)) => format!("--{long}"),
        (None, None) => arg.get_id().to_string(),
    }
}

/// `args` with every raw-only token ahead of the command word moved to just
/// after the command path, rendered as a runnable invocation; `None` when a
/// token cannot be read or the moved invocation would not reach the command.
fn move_after_command(root: &Command, matches: &ArgMatches, args: &[OsString]) -> Option<String> {
    let words: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let mut kept = Vec::new();
    let mut moved = Vec::new();
    let mut at = 1;
    while let Some(word) = words.get(at) {
        if word == "--" || word == "-" || !word.starts_with('-') {
            break;
        }
        let (arg, inline_value) = root_flag(root, word)?;
        let separate_value =
            !inline_value && arg.get_action().takes_values() && !arg.is_require_equals_set();
        let end = if separate_value { at + 2 } else { at + 1 };
        let tokens = words.get(at..end)?;
        if RAW_ONLY.contains(&arg.get_id().as_str()) {
            moved.extend_from_slice(tokens);
        } else {
            kept.extend_from_slice(tokens);
        }
        at = end;
    }
    let mut depth = 0;
    let mut current = matches;
    while let Some((_, sub)) = current.subcommand() {
        depth += 1;
        current = sub;
    }
    let path_end = at.checked_add(depth).filter(|end| *end <= words.len())?;
    let rebuilt: Vec<String> = std::iter::once(words[0].clone())
        .chain(kept)
        .chain(words[at..path_end].iter().cloned())
        .chain(moved)
        .chain(words[path_end..].iter().cloned())
        .collect();
    reaches_command(root, &rebuilt).then(|| {
        std::iter::once(ROOT_COMMAND.to_string())
            .chain(rebuilt[1..].iter().map(|word| shell_word(word)))
            .collect::<Vec<_>>()
            .join(" ")
    })
}

/// The root flag `word` spells, and whether `word` carries its value too
/// (`--auth=oauth2`, `-uwork`). `None` for a word the root does not define,
/// or a cluster of short flags.
fn root_flag<'a>(root: &'a Command, word: &str) -> Option<(&'a Arg, bool)> {
    if let Some(long) = word.strip_prefix("--") {
        let (name, inline) = long
            .split_once('=')
            .map_or((long, false), |(name, _)| (name, true));
        let arg = root
            .get_arguments()
            .find(|arg| arg.get_long() == Some(name))?;
        return Some((arg, inline));
    }
    let mut shorts = word.strip_prefix('-')?.chars();
    let short = shorts.next()?;
    let arg = root
        .get_arguments()
        .find(|arg| arg.get_short() == Some(short))?;
    let rest = shorts.as_str();
    if rest.is_empty() {
        Some((arg, false))
    } else if arg.get_action().takes_values() {
        Some((arg, true))
    } else {
        None
    }
}

/// Whether `rebuilt` parses with no raw-only flag at the root and every one
/// it moved read by the command.
fn reaches_command(root: &Command, rebuilt: &[String]) -> bool {
    let Ok(matches) = root.clone().try_get_matches_from(rebuilt) else {
        return false;
    };
    let from_command_line =
        |matches: &ArgMatches, id: &str| matches.value_source(id) == Some(ValueSource::CommandLine);
    if RAW_ONLY.iter().any(|id| from_command_line(&matches, id)) {
        return false;
    }
    let mut command = &matches;
    while let Some((_, sub)) = command.subcommand() {
        command = sub;
    }
    MOVABLE
        .iter()
        .filter(|id| rebuilt.iter().any(|word| word_names(root, word, id)))
        .all(|id| {
            command
                .try_contains_id(id)
                .is_ok_and(|_| from_command_line(command, id))
        })
}

/// Whether `word` spells the root flag `id`.
fn word_names(root: &Command, word: &str, id: &str) -> bool {
    word.starts_with('-') && root_flag(root, word).is_some_and(|(arg, _)| arg.get_id() == id)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    fn run(args: &[&str]) -> Option<Misplaced> {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        let cli = Cli::try_parse_from(&args).expect("the invocation parses");
        misplaced(&cli, &args)
    }

    #[test]
    fn a_login_ahead_of_a_command_moves_after_its_word() {
        let found = run(&["xr", "--output", "json", "-u", "work", "whoami"]).expect("refused");
        assert_eq!(found.flags, ["-u"]);
        assert_eq!(
            found.moved.as_deref(),
            Some("xr --output json whoami -u work")
        );
    }

    #[test]
    fn each_spelling_of_a_movable_flag_moves() {
        for (args, moved) in [
            (
                &["xr", "--username=work", "whoami"][..],
                "xr whoami --username=work",
            ),
            (&["xr", "-uwork", "whoami"], "xr whoami -uwork"),
            (
                &["xr", "--auth", "oauth1", "-t", "post", "hi"],
                "xr post --auth oauth1 -t hi",
            ),
            (
                &["xr", "-u", "work", "media", "upload", "a.png"],
                "xr media upload -u work a.png",
            ),
        ] {
            assert_eq!(run(args).and_then(|m| m.moved).as_deref(), Some(moved));
        }
    }

    #[test]
    fn a_flag_the_command_lacks_or_one_that_could_hold_a_secret_has_no_move() {
        let found = run(&["xr", "-X", "POST", "whoami"]).expect("refused");
        assert_eq!(found.flags, ["-X"]);
        assert!(found.moved.is_none());
        let found = run(&["xr", "-u", "work", "-H", "a: b", "whoami"]).expect("refused");
        assert_eq!(found.flags, ["-H", "-u"]);
        assert!(found.moved.is_none());
        let found = run(&["xr", "-u", "work", "auth", "status"]).expect("refused");
        assert!(found.moved.is_none(), "auth status takes no -u");
    }

    #[test]
    fn a_login_given_twice_has_no_move() {
        let found = run(&["xr", "-u", "a", "whoami", "-u", "b"]).expect("refused");
        assert!(found.moved.is_none(), "{:?}", found.moved);
    }

    #[test]
    fn a_raw_request_and_a_command_alone_are_not_refused() {
        assert!(run(&["xr", "-u", "work", "/2/users/me"]).is_none());
        assert!(run(&["xr", "whoami", "-u", "work"]).is_none());
        assert!(run(&["xr", "--app", "work", "whoami"]).is_none());
    }
}
