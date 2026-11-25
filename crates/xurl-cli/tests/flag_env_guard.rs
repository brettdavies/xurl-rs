//! Fails when a flag and the variable of its own setting can refuse each other,
//! or when a refusal between a flag and a variable leaves the variable unnamed.
//!
//! Two walks read `xr`'s declarations, so a flag added later is covered with no
//! edit here. The override walk finds every pair of flags where typing the
//! second takes the first's value away, and holds each pair to one setting of
//! the table in `crates/xurl-cli/src/cli/settings.rs`, with no variable bound
//! in clap to either flag: clap counts a value read from the environment as
//! typed, and an override gives the argument back its slot, so a bound variable
//! refuses the typed flag of its own setting. The conflict walk finds every flag
//! that clap declares in conflict with a variable-backed argument, and runs
//! each pair through the binary.
//!
//! A test over a command built here, with categories `xr` lacks, shows the walks
//! see what they claim to.

mod common;

use std::any::TypeId;
use std::collections::BTreeSet;

use clap::builder::FalseyValueParser;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgGroup, Command};
use common::invoke::{SAMPLE, arguments_for, command_at, runnable_paths, tree};
use xurl::cli::parse::{read_value, without_false_bindings};
use xurl::cli::settings::{SETTINGS, conflicting};

/// A flag the walks can type: an option with a long name, other than the
/// help and version flags clap adds.
fn is_probed(arg: &Arg) -> bool {
    !arg.is_positional()
        && arg.get_long().is_some()
        && !matches!(arg.get_id().as_str(), "help" | "version")
}

fn flags_of(command: &Command) -> Vec<&Arg> {
    command
        .get_arguments()
        .filter(|arg| is_probed(arg))
        .collect()
}

fn is_boolean(arg: &Arg) -> bool {
    arg.get_value_parser().type_id() == TypeId::of::<bool>()
}

/// The values a flag is tried with, in order, when it declares no possible
/// value: the sample every walk uses, then one for each kind of parser `xr`
/// has (a count in a range, an address, a word).
const CANDIDATES: [&str; 4] = [SAMPLE, "1", "127.0.0.1", "a"];

/// The value a flag, or the variable that stands for it, is given: its first
/// possible value, or the first of [`CANDIDATES`] its parser reads.
fn sample_of(arg: &Arg) -> String {
    if let Some(value) = arg.get_possible_values().first() {
        return value.get_name().to_string();
    }
    CANDIDATES
        .into_iter()
        .find(|value| read_value(arg.get_value_parser(), value).is_ok())
        .unwrap_or_else(|| {
            panic!(
                "no value in CANDIDATES in flag_env_guard.rs is one `--{}` reads; add one the \
                 flag's parser accepts",
                arg.get_long().unwrap_or(arg.get_id().as_str())
            )
        })
        .to_string()
}

/// The words that type `arg`.
fn typed(arg: &Arg) -> Vec<String> {
    let flag = format!(
        "--{}",
        arg.get_long().expect("a probed flag has a long name")
    );
    if !arg.get_action().takes_values() {
        vec![flag]
    } else if arg.is_require_equals_set() {
        vec![format!("{flag}={}", sample_of(arg))]
    } else {
        vec![flag, sample_of(arg)]
    }
}

/// The value a variable holds that stands for `arg`: `1` for a boolean, which
/// reads as true.
fn variable_value(arg: &Arg) -> String {
    if is_boolean(arg) {
        "1".to_string()
    } else {
        sample_of(arg)
    }
}

// ── The override walk ──────────────────────────────────────────────────────

/// What a walk found, and how many cases it ran.
struct Probed {
    overrides: BTreeSet<(String, String)>,
    cases: usize,
}

/// Types each ordered pair of `pairs` on a clone of the built `command`, after
/// `base`, and collects the pairs where the second takes the first's
/// command-line value away.
///
/// A pair clap refuses as a conflict overrides nothing. Any other failure is
/// the probe's, so it fails the walk, naming the command and clap's error.
fn probe_overrides(
    name: &str,
    command: &Command,
    base: &[String],
    pairs: &[(&Arg, &Arg)],
) -> Probed {
    let mut command = command.clone().no_binary_name(true);
    let mut probed = Probed {
        overrides: BTreeSet::new(),
        cases: 0,
    };
    for (first, second) in pairs {
        if first.get_id() == second.get_id()
            && matches!(first.get_action(), ArgAction::Append | ArgAction::Count)
        {
            continue;
        }
        let mut words: Vec<String> = base
            .iter()
            .cloned()
            .chain(typed(first))
            .chain(typed(second))
            .collect();
        probed.cases += 1;
        let result = loop {
            match command.try_get_matches_from_mut(&words) {
                Err(error) if error.kind() == ErrorKind::MissingRequiredArgument => {
                    let wanted = required_by(&command, &error, &words);
                    if wanted.is_empty() {
                        break Err(error);
                    }
                    words.extend(wanted);
                }
                result => break result,
            }
        };
        match result {
            Ok(matches) => {
                let repeated = first.get_id() == second.get_id();
                if repeated
                    || matches.value_source(first.get_id().as_str())
                        != Some(ValueSource::CommandLine)
                {
                    probed
                        .overrides
                        .insert((first.get_id().to_string(), second.get_id().to_string()));
                }
            }
            Err(error) if error.kind() == ErrorKind::ArgumentConflict => {}
            Err(error) => panic!(
                "{name}: typing {words:?} failed for a reason other than the pair itself, so the \
                 walk cannot tell whether {second} overrides {first}: {error}",
                first = first.get_id(),
                second = second.get_id()
            ),
        }
    }
    probed
}

/// The words that type an argument `error` says a typed flag requires and
/// `words` lack, as `--auth-url` requires `--step`. A required group, which
/// clap writes `<COMMAND|--common>`, takes its first member that is a flag.
fn required_by(command: &Command, error: &clap::Error, words: &[String]) -> Vec<String> {
    let Some(ContextValue::Strings(missing)) = error.get(ContextKind::InvalidArg) else {
        return Vec::new();
    };
    let named = |name: &str| {
        command.get_arguments().find(|arg| {
            is_probed(arg)
                && (arg.to_string() == name
                    || format!("--{}", arg.get_long().unwrap_or_default()) == name)
        })
    };
    missing
        .iter()
        .filter_map(|name| {
            named(name).or_else(|| name.trim_matches(['<', '>']).split('|').find_map(&named))
        })
        .map(typed)
        .filter(|typing| !words.contains(&typing[0]))
        .flatten()
        .collect()
}

/// The ordered pairs of flags of `command` to probe at its path: every pair
/// when `globals` is probed, and otherwise only the pairs that hold one of the
/// command's own flags, since a pair of two globals is the same at every
/// command.
fn pairs_of(command: &Command, globals: bool) -> Vec<(&Arg, &Arg)> {
    let flags = flags_of(command);
    let mut pairs = Vec::new();
    for first in &flags {
        for second in &flags {
            if globals || !(first.is_global_set() && second.is_global_set()) {
                pairs.push((*first, *second));
            }
        }
    }
    pairs
}

/// The messages that fail the override walk for `overrides` found in `command`:
/// a participant that binds a variable in clap.
fn bound_participants(command: &Command, overrides: &BTreeSet<(String, String)>) -> Vec<String> {
    let participants: BTreeSet<&String> = overrides
        .iter()
        .flat_map(|(first, second)| [first, second])
        .collect();
    participants
        .into_iter()
        .filter_map(|id| {
            let arg = command
                .get_arguments()
                .find(|arg| arg.get_id() == id.as_str())?;
            let variable = arg.get_env()?;
            Some(format!(
                "`--{}` binds the variable {variable:?} in clap and takes part in an override. \
                 Clap counts a value read from the environment as typed, so that variable would \
                 refuse the typed flags of its own setting. Take `env =` off the flag and \
                 resolve the variable with the settings table in \
                 crates/xurl-cli/src/cli/settings.rs",
                arg.get_long().unwrap_or(id.as_str()),
                variable = variable.to_string_lossy()
            ))
        })
        .collect()
}

/// The pairs of `overrides` that sit in no one setting of the table.
fn outside_the_table(overrides: &BTreeSet<(String, String)>) -> Vec<String> {
    overrides
        .iter()
        .filter(|(first, second)| {
            !SETTINGS.iter().any(|setting| {
                let holds = |id: &String| setting.spellings.iter().any(|s| s.id == id.as_str());
                holds(first) && holds(second)
            })
        })
        .map(|(first, second)| {
            format!(
                "`--{second}` overrides `--{first}`, and no setting in the table in \
                 crates/xurl-cli/src/cli/settings.rs holds both. Add the setting to the table, or \
                 stop the flags overriding each other"
            )
        })
        .collect()
}

/// The ordered pairs of spellings of a setting that do not override each
/// other.
fn missing_overrides(overrides: &BTreeSet<(String, String)>) -> Vec<String> {
    let mut missing = Vec::new();
    for setting in SETTINGS {
        for first in setting.spellings {
            for second in setting.spellings {
                let pair = (first.id.to_string(), second.id.to_string());
                if !overrides.contains(&pair) {
                    missing.push(format!(
                        "{}: typing `{}` and then `{}` leaves `{}` its value, so the spellings of \
                         one setting do not override one another",
                        setting.name, first.flag, second.flag, first.flag
                    ));
                }
            }
        }
    }
    missing
}

#[test]
fn flags_that_override_one_another_sit_in_one_setting_with_no_bound_variable() {
    let mut overrides = BTreeSet::new();
    let mut failures = Vec::new();
    let mut cases = 0;
    for path in std::iter::once("version".to_string()).chain(runnable_paths()) {
        let command = command_at(&path);
        let base = if path == "version" {
            Vec::new()
        } else {
            arguments_for(&path)
        };
        let pairs = pairs_of(command, path == "version");
        let probed = probe_overrides(&path, command, &base, &pairs);
        cases += probed.cases;
        failures.extend(bound_participants(command, &probed.overrides));
        overrides.extend(probed.overrides);
    }

    assert!(cases > 0, "the override walk ran no case");
    failures.extend(outside_the_table(&overrides));
    failures.extend(missing_overrides(&overrides));
    let failures: BTreeSet<String> = failures.into_iter().collect();
    assert!(
        failures.is_empty(),
        "{}",
        failures.into_iter().collect::<Vec<_>>().join("\n\n")
    );
}

// ── The conflict walk ──────────────────────────────────────────────────────

/// An argument whose value a variable can give: it binds one in clap, or its
/// spelling is in the settings table.
fn variable_of(arg: &Arg) -> Option<String> {
    arg.get_env()
        .map(|name| name.to_string_lossy().into_owned())
        .or_else(|| {
            SETTINGS
                .iter()
                .flat_map(|setting| setting.spellings)
                .find(|spelling| spelling.id == arg.get_id().as_str())
                .map(|spelling| spelling.variable.to_string())
        })
}

/// What the walk runs for one conflict.
struct Conflict {
    path: String,
    flag: String,
    variable_arg: String,
    variable: String,
    flag_words: Vec<String>,
    variable_value: String,
    boolean: bool,
}

/// The words that reach the command at `path`.
fn reaching(path: &str) -> Vec<String> {
    if path == "schema" {
        vec!["schema".to_string(), "--list".to_string()]
    } else {
        path.split(' ')
            .map(str::to_string)
            .chain(
                arguments_for(path)
                    .into_iter()
                    .map(|word| word.replace("{file}", "no-such-file")),
            )
            .collect()
    }
}

fn conflicts() -> Vec<Conflict> {
    let mut found = Vec::new();
    for path in std::iter::once("schema".to_string()).chain(runnable_paths()) {
        let command = command_at(&path);
        for flag in flags_of(command) {
            let declared = conflicting(command, flag.get_id().as_str());
            for variable_arg in &declared {
                let Some(variable) = variable_of(variable_arg) else {
                    continue;
                };
                let both_global = flag.is_global_set() && variable_arg.is_global_set();
                if both_global && path != "schema" {
                    continue;
                }
                found.push(Conflict {
                    path: path.clone(),
                    flag: format!("--{}", flag.get_long().unwrap_or_default()),
                    variable_arg: variable_arg.get_id().to_string(),
                    variable,
                    flag_words: typed(flag),
                    variable_value: variable_value(variable_arg),
                    boolean: is_boolean(variable_arg),
                });
            }
        }
    }
    found
}

fn run(conflict: &Conflict, value: &str) -> (i32, String) {
    let output = common::xr()
        .env(&conflict.variable, value)
        .args(reaching(&conflict.path))
        .args(&conflict.flag_words)
        .output()
        .expect("xr runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The refusal's text above its usage line and help pointer.
fn head(stderr: &str) -> &str {
    let end = ["\n\nUsage:", "\n\nTry '"]
        .iter()
        .filter_map(|marker| stderr.find(marker))
        .min()
        .unwrap_or(stderr.len());
    &stderr[..end]
}

#[test]
fn a_flag_beside_a_variable_it_conflicts_with_is_refused_naming_the_variable() {
    let cases = conflicts();
    assert!(!cases.is_empty(), "the conflict walk found no case");
    let mut failures = Vec::new();
    for case in &cases {
        let (code, stderr) = run(case, &case.variable_value);
        let named = format!("the environment variable '{}'", case.variable);
        let mut problems = Vec::new();
        if code != 2 {
            problems.push(format!("exited {code}, not 2"));
        }
        if !head(&stderr).contains(&named) {
            problems.push(format!("does not say {named}"));
        }
        let own_flag = format!("--{}", case.variable_arg);
        if head(&stderr).contains(&own_flag) {
            problems.push(format!(
                "names `{own_flag}`, a flag the caller did not type, in place of the variable"
            ));
        }
        if !problems.is_empty() {
            failures.push(format!(
                "`xr {}` with `{}` typed and {}={} set: {}\n{stderr}",
                case.path,
                case.flag,
                case.variable,
                case.variable_value,
                problems.join("; ")
            ));
        }
    }
    for case in cases.iter().filter(|case| case.boolean) {
        let (_, stderr) = run(case, "0");
        if stderr.contains("cannot be used with") {
            failures.push(format!(
                "`xr {}` with `{}` typed and {}=0 set is refused, though a false boolean \
                 variable is unset:\n{stderr}",
                case.path, case.flag, case.variable
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn the_conflict_walk_sees_the_page_token_pairs() {
    let pairs: BTreeSet<(String, String)> = conflicts()
        .into_iter()
        .map(|case| (case.flag, case.variable))
        .collect();
    for (flag, variable) in [
        ("--cursor", "XURL_PAGE"),
        ("--after", "XURL_PAGE"),
        ("--page", "XURL_CURSOR"),
        ("--page", "XURL_AFTER"),
    ] {
        assert!(
            pairs.contains(&(flag.to_string(), variable.to_string())),
            "the walk does not pair {flag} with {variable}: {pairs:?}"
        );
    }
}

// ── The fixture: categories `xr` lacks ─────────────────────────────────────

/// A command with a group of three flags that each bind a variable, an
/// exclusive flag, and an override pair in which one side binds a variable.
fn fixture() -> Command {
    let flag = |id: &'static str| {
        Arg::new(id)
            .long(id)
            .env(format!("FIXTURE_{}", id.to_uppercase()))
            .value_parser(FalseyValueParser::new())
            .action(ArgAction::SetTrue)
    };
    let mut command = Command::new("fixture")
        .arg(flag("a"))
        .arg(flag("b"))
        .arg(flag("c"))
        .group(
            ArgGroup::new("members")
                .args(["a", "b", "c"])
                .multiple(false),
        )
        .arg(
            Arg::new("x")
                .long("x")
                .action(ArgAction::SetTrue)
                .exclusive(true),
        )
        .arg(flag("p").overrides_with("q"))
        .arg(Arg::new("q").long("q").action(ArgAction::SetTrue));
    command.build();
    command
}

#[test]
fn the_reader_reports_a_groups_pairs_and_an_exclusive_flag() {
    let command = fixture();
    for first in ["a", "b", "c"] {
        let conflicts: BTreeSet<&str> = conflicting(&command, first)
            .iter()
            .map(|arg| arg.get_id().as_str())
            .collect();
        for second in ["a", "b", "c"]
            .into_iter()
            .filter(|second| *second != first)
        {
            assert!(
                conflicts.contains(second),
                "{first} is not reported in conflict with {second}: {conflicts:?}"
            );
        }
        assert!(
            conflicts.contains("x"),
            "{first} and the exclusive flag: {conflicts:?}"
        );
    }
    let exclusive: BTreeSet<&str> = conflicting(&command, "x")
        .iter()
        .map(|arg| arg.get_id().as_str())
        .collect();
    assert_eq!(
        exclusive,
        ["a", "b", "c", "help", "p", "q"]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        "the exclusive flag against every other argument"
    );
}

#[test]
fn the_override_probe_finds_a_pair_and_the_check_flags_its_bound_member() {
    let command = fixture();
    let flags = flags_of(&command);
    let pairs: Vec<(&Arg, &Arg)> = flags
        .iter()
        .flat_map(|first| flags.iter().map(move |second| (*first, *second)))
        .collect();
    let probed = probe_overrides("fixture", &command, &[], &pairs);

    assert!(probed.cases > 0);
    assert!(
        probed
            .overrides
            .contains(&("p".to_string(), "q".to_string())),
        "the pair is found: {:?}",
        probed.overrides
    );
    let messages = bound_participants(&command, &probed.overrides);
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("`--p` binds the variable \"FIXTURE_P\""));
    assert!(
        !outside_the_table(&probed.overrides).is_empty(),
        "the fixture's pair is in no setting of the table"
    );
}

#[test]
fn the_false_variable_step_removes_a_false_members_binding() {
    let lookup = |name: &str| (name == "FIXTURE_A").then(|| "0".to_string());
    let command = without_false_bindings(fixture(), &lookup);
    let env_of = |id: &str| {
        command
            .get_arguments()
            .find(|arg| arg.get_id() == id)
            .and_then(|arg| arg.get_env())
            .is_some()
    };
    assert!(!env_of("a"), "the false member keeps no binding");
    assert!(
        env_of("b"),
        "a member the lookup does not hold keeps its binding"
    );
}

// ── What the argv read assumes ─────────────────────────────────────────────

/// The commands of the built tree, root included.
fn every_command() -> Vec<&'static Command> {
    fn collect(command: &'static Command, out: &mut Vec<&'static Command>) {
        out.push(command);
        for sub in command.get_subcommands() {
            collect(sub, out);
        }
    }
    let mut out = Vec::new();
    collect(tree(), &mut out);
    out
}

#[test]
fn the_argv_read_holds_for_every_argument_in_the_tree() {
    let commands = every_command();
    let mut checked = 0;
    let mut failures = Vec::new();
    for command in commands {
        for arg in command.get_arguments() {
            checked += 1;
            let name = format!("{} {}", command.get_name(), arg.get_id());
            if arg.is_allow_hyphen_values_set() {
                failures.push(format!("{name} accepts hyphen-led values"));
            }
            if arg.is_trailing_var_arg_set() {
                failures.push(format!("{name} is a trailing variable argument"));
            }
        }
    }
    for spelling in SETTINGS.iter().flat_map(|setting| setting.spellings) {
        let arg = tree()
            .get_arguments()
            .find(|arg| arg.get_id() == spelling.id)
            .unwrap_or_else(|| panic!("the table names `{}`, which the root lacks", spelling.id));
        if arg.get_short().is_some() || arg.get_all_short_aliases().is_some() {
            failures.push(format!("{} has a short form", spelling.flag));
        }
        if arg.get_all_aliases().is_some() {
            failures.push(format!("{} has an alias", spelling.flag));
        }
        if format!("--{}", arg.get_long().unwrap_or_default()) != spelling.flag {
            failures.push(format!(
                "{} is not the flag of `{}`",
                spelling.flag, spelling.id
            ));
        }
    }
    assert!(checked > 0, "the argv-read check saw no argument");
    assert!(
        failures.is_empty(),
        "the typed-order read of `--output`, `--json`, `--jsonl`, `--cursor`, and `--after` \
         from argv assumes every spelling is a long flag with no alias and no argument takes a \
         hyphen-led value:\n{}",
        failures.join("\n")
    );
}
