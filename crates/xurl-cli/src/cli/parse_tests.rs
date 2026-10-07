//! Parse-level assertions on the clap definition: which argv binds to which
//! field. Assertions on what `xr` prints live in `tests/`, driven through the
//! runner or the built binary.

use clap::Parser;

use super::{
    AuthCommands, BroadcastsCommands, Cli, Commands, MediaCommands, ModeratorsCommands,
    ProcessingWait,
};

/// `xr auth oauth2 alice --no-browser --step 1` binds the positional to
/// `alice`. Driving the flow end to end lives in `tests/auth_remote_tests.rs`.
#[test]
fn oauth2_positional_username_binds() {
    let parsed = Cli::try_parse_from([
        "xr",
        "auth",
        "oauth2",
        "alice",
        "--no-browser",
        "--step",
        "1",
    ])
    .expect("positional + --no-browser --step 1 must parse");

    let Some(Commands::Auth { command }) = parsed.command else {
        panic!("expected Auth subcommand");
    };
    match command {
        AuthCommands::Oauth2 {
            no_browser,
            step,
            auth_url,
            username,
            ..
        } => {
            assert!(no_browser, "--no-browser should be set");
            assert_eq!(step, Some(1));
            assert!(auth_url.is_none());
            assert_eq!(
                username.as_deref(),
                Some("alice"),
                "positional username must bind to `alice`",
            );
        }
        other => panic!("expected AuthCommands::Oauth2, got {other:?}"),
    }
}

/// Which global boolean flags the parse landed on, plus the command word.
#[derive(Debug, PartialEq, Eq)]
struct ParsedFlags {
    quiet: bool,
    verbose: bool,
    no_interactive: bool,
    dry_run: bool,
    raw: bool,
    url: Option<String>,
    command: &'static str,
}

fn parse_flags(argv: &[&str]) -> ParsedFlags {
    let cli = Cli::try_parse_from(argv).unwrap_or_else(|e| panic!("{argv:?} must parse: {e}"));
    let command = match cli.command {
        Some(Commands::Whoami { .. }) => "whoami",
        Some(Commands::Search { ref query, .. }) => {
            assert_eq!(
                query, "topic",
                "search query must be the word after the flag"
            );
            "search"
        }
        Some(_) => "other",
        None => "none",
    };
    ParsedFlags {
        quiet: cli.quiet,
        verbose: cli.verbose,
        no_interactive: cli.no_interactive,
        dry_run: cli.dry_run,
        raw: cli.raw,
        url: cli.url,
        command,
    }
}

/// A boolean flag followed by a command word leaves the word to the
/// command; an explicit value needs `=`. These read the clap env bindings,
/// so they run outside any serial window that mutates the environment.
#[rstest::rstest]
#[case::quiet_long(&["xr", "--quiet", "whoami"], ParsedFlags { quiet: true, verbose: false, no_interactive: false, dry_run: false, raw: false, url: None, command: "whoami" })]
#[case::quiet_short(&["xr", "-q", "whoami"], ParsedFlags { quiet: true, verbose: false, no_interactive: false, dry_run: false, raw: false, url: None, command: "whoami" })]
#[case::verbose(&["xr", "--verbose", "whoami"], ParsedFlags { quiet: false, verbose: true, no_interactive: false, dry_run: false, raw: false, url: None, command: "whoami" })]
#[case::no_interactive(&["xr", "--no-interactive", "whoami"], ParsedFlags { quiet: false, verbose: false, no_interactive: true, dry_run: false, raw: false, url: None, command: "whoami" })]
#[case::raw(&["xr", "--raw", "whoami"], ParsedFlags { quiet: false, verbose: false, no_interactive: false, dry_run: false, raw: true, url: None, command: "whoami" })]
#[case::dry_run(&["xr", "--dry-run", "whoami"], ParsedFlags { quiet: false, verbose: false, no_interactive: false, dry_run: true, raw: false, url: None, command: "whoami" })]
#[case::quiet_equals_false(&["xr", "--quiet=false", "whoami"], ParsedFlags { quiet: false, verbose: false, no_interactive: false, dry_run: false, raw: false, url: None, command: "whoami" })]
#[case::quiet_then_false_word(&["xr", "--quiet", "false", "whoami"], ParsedFlags { quiet: true, verbose: false, no_interactive: false, dry_run: false, raw: false, url: Some("false".to_string()), command: "whoami" })]
#[case::quiet_search(&["xr", "-q", "search", "topic"], ParsedFlags { quiet: true, verbose: false, no_interactive: false, dry_run: false, raw: false, url: None, command: "search" })]
#[case::flag_after_subcommand(&["xr", "search", "--quiet", "topic"], ParsedFlags { quiet: true, verbose: false, no_interactive: false, dry_run: false, raw: false, url: None, command: "search" })]
#[case::short_equals_false(&["xr", "-q=false", "whoami"], ParsedFlags { quiet: false, verbose: false, no_interactive: false, dry_run: false, raw: false, url: None, command: "whoami" })]
#[case::short_cluster(&["xr", "-qv", "whoami"], ParsedFlags { quiet: true, verbose: true, no_interactive: false, dry_run: false, raw: false, url: None, command: "whoami" })]
#[serial_test::parallel]
fn boolean_flag_does_not_swallow_command_word(
    #[case] argv: &[&str],
    #[case] expected: ParsedFlags,
) {
    assert_eq!(parse_flags(argv), expected, "argv: {argv:?}");
}

/// `--no-browser` on `auth oauth2` keeps the username positional after it.
#[rstest::rstest]
#[case::flag_then_username(&["xr", "auth", "oauth2", "--no-browser", "alice"], true)]
#[case::equals_false_then_username(&["xr", "auth", "oauth2", "--no-browser=false", "alice"], false)]
#[serial_test::parallel]
fn no_browser_keeps_username_positional(#[case] argv: &[&str], #[case] expected: bool) {
    let parsed = Cli::try_parse_from(argv).unwrap_or_else(|e| panic!("{argv:?} must parse: {e}"));
    let Some(Commands::Auth { command }) = parsed.command else {
        panic!("expected Auth subcommand");
    };
    let AuthCommands::Oauth2 {
        no_browser,
        username,
        ..
    } = command
    else {
        panic!("expected auth oauth2");
    };
    assert_eq!(no_browser, expected, "argv: {argv:?}");
    assert_eq!(username.as_deref(), Some("alice"), "argv: {argv:?}");
}

/// The space-separated form is not a value: the word after the flag is a
/// positional or a command, never the flag's value.
#[test]
#[serial_test::parallel]
fn quiet_space_true_is_not_a_flag_value() {
    let parsed = parse_flags(&["xr", "--quiet", "true"]);
    assert!(parsed.quiet);
    assert_eq!(parsed.url.as_deref(), Some("true"));
}

/// `xr broadcasts moderators add @alice` binds the handle to the add verb of
/// the moderators family.
#[test]
fn broadcasts_moderators_add_binds_the_handle() {
    let parsed = Cli::try_parse_from(["xr", "broadcasts", "moderators", "add", "@alice"])
        .expect("the family parses");

    let Some(Commands::Broadcasts { target }) = parsed.command else {
        panic!("expected Broadcasts subcommand");
    };
    let BroadcastsCommands::Moderators { action } = target;
    match action {
        ModeratorsCommands::Add {
            target_username, ..
        } => assert_eq!(target_username, "@alice"),
        other => panic!("expected the add verb, got {other:?}"),
    }
}

/// The deadline `xr media <verb> ...` waits to, as clap resolved `--wait`.
fn media_wait(argv: &[&str]) -> ProcessingWait {
    let parsed = Cli::try_parse_from(argv).unwrap_or_else(|e| panic!("{argv:?} must parse: {e}"));
    match parsed.command {
        Some(Commands::Media {
            command: MediaCommands::Upload { wait, .. } | MediaCommands::Status { wait, .. },
        }) => wait,
        other => panic!("expected media upload or status, got {other:?}"),
    }
}

/// `--wait` reads as one flag on both media verbs: bare or `true` is the
/// default deadline, a number is that many seconds, and `false` or `0` is no
/// wait. Upload waits when the flag is absent; status does not.
#[rstest::rstest]
#[case::status_absent(&["xr", "media", "status", "1"], None)]
#[case::status_bare(&["xr", "media", "status", "1", "--wait"], Some(60))]
#[case::status_short(&["xr", "media", "status", "1", "-w"], Some(60))]
#[case::status_true(&["xr", "media", "status", "1", "--wait=true"], Some(60))]
#[case::status_seconds(&["xr", "media", "status", "1", "--wait=120"], Some(120))]
#[case::status_false(&["xr", "media", "status", "1", "--wait=false"], None)]
#[case::status_zero(&["xr", "media", "status", "1", "--wait=0"], None)]
#[case::upload_absent(&["xr", "media", "upload", "clip.mp4"], Some(60))]
#[case::upload_bare(&["xr", "media", "upload", "clip.mp4", "--wait"], Some(60))]
#[case::upload_seconds(&["xr", "media", "upload", "clip.mp4", "--wait=300"], Some(300))]
#[case::upload_false(&["xr", "media", "upload", "clip.mp4", "--wait=false"], None)]
#[case::upload_zero(&["xr", "media", "upload", "clip.mp4", "--wait=0"], None)]
#[serial_test::parallel]
fn wait_takes_a_deadline_true_or_false(#[case] argv: &[&str], #[case] secs: Option<u64>) {
    assert_eq!(
        media_wait(argv),
        ProcessingWait(secs.map(std::time::Duration::from_secs)),
        "argv: {argv:?}"
    );
}

#[test]
#[serial_test::parallel]
fn wait_rejects_a_value_that_is_neither_seconds_nor_a_bool() {
    let error = Cli::try_parse_from(["xr", "media", "status", "1", "--wait=soon"])
        .expect_err("soon is not a deadline");
    assert!(
        error
            .to_string()
            .contains("expected a number of seconds, 'true', or 'false'"),
        "{error}"
    );
}
