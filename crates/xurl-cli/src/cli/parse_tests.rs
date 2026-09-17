//! Parse-level assertions on the clap definition: which argv binds to which
//! field. Assertions on what `xr` prints live in `tests/`, driven through the
//! runner or the built binary.

use clap::Parser;

use super::{AuthCommands, Cli, Commands};

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

/// The space-separated value form no longer parses as a value: the word
/// after the flag is a positional or a command, never the flag's value.
#[test]
#[serial_test::parallel]
fn quiet_space_true_is_not_a_flag_value() {
    let parsed = parse_flags(&["xr", "--quiet", "true"]);
    assert!(parsed.quiet);
    assert_eq!(parsed.url.as_deref(), Some("true"));
}
