//! `.config/nextest.toml` decides what a green test run means, so this holds
//! every profile to two rules. A retry never turns a test green: nextest's
//! default `flaky-result` passes a test that fails and then passes, and a retry
//! can come from `--retries` or `NEXTEST_RETRIES` as well as the file, so every
//! profile resolves `flaky-result = "fail"`. A hung test is ended: nextest's
//! built-in `slow-timeout` never ends one, so the runner kills the job and the
//! log never names the test.
//!
//! A profile inherits from `default` unless it says otherwise, and each setting
//! resolves on its own, so the rules read what a profile ends up with, and report
//! a setting under the profile that writes it.

mod common;

use rstest::rstest;
use toml::{Table, Value};

const CONFIG: &str = ".config/nextest.toml";
const UNSAFE_RETRY: &str = "nextest-config/unsafe-retry";
const NO_TERMINATION: &str = "nextest-config/no-termination";

/// `profile`, then each profile it inherits from, as (name, table) pairs.
fn chain<'a>(profiles: &'a Table, profile: &'a str) -> Vec<(&'a str, &'a Table)> {
    let mut chain: Vec<(&str, &Table)> = Vec::new();
    let mut name = profile;
    while let Some(table) = profiles.get(name).and_then(Value::as_table) {
        if chain.iter().any(|(seen, _)| *seen == name) {
            break;
        }
        chain.push((name, table));
        name = match table.get("inherits").and_then(Value::as_str) {
            Some(parent) => parent,
            None if name == "default" => break,
            None => "default",
        };
    }
    chain
}

/// Where a profile's `key` comes from: the profile that writes it and its value,
/// or, when no profile in the chain writes it, the chain's root and nothing.
fn resolved<'a>(chain: &[(&'a str, &'a Table)], key: &str) -> (&'a str, Option<&'a Value>) {
    chain
        .iter()
        .find_map(|(name, table)| table.get(key).map(|value| (*name, Some(value))))
        .unwrap_or_else(|| (chain.last().map_or("default", |(name, _)| *name), None))
}

/// Why a `slow-timeout` does not end a hung test and fail it, or `None` when it
/// does.
fn termination_gap(slow_timeout: Option<&Value>) -> Option<String> {
    let Some(value) = slow_timeout else {
        return Some("unset, and nextest's built-in never ends a test".to_string());
    };
    if let Some(period) = value.as_str() {
        return Some(format!("{period:?} has no terminate-after"));
    }
    if value.get("terminate-after").is_none() {
        return Some("no terminate-after".to_string());
    }
    if value.get("on-timeout").and_then(Value::as_str) == Some("pass") {
        return Some("on-timeout = \"pass\" counts a test it ends as passed".to_string());
    }
    None
}

fn unsafe_retry(place: &str, what: &str) -> String {
    format!(
        "{UNSAFE_RETRY}: {CONFIG} {place} key flaky-result: {what}, so a test that fails and then \
         passes on a retry (from the file, --retries, or NEXTEST_RETRIES) turns the run green; set \
         flaky-result = \"fail\" (AGENTS.md § {UNSAFE_RETRY})"
    )
}

fn no_termination(place: &str, gap: &str) -> String {
    format!(
        "{NO_TERMINATION}: {CONFIG} {place} key slow-timeout: {gap}, so a hung test runs until \
         the runner kills the job, unnamed; set slow-timeout = {{ period = \"60s\", \
         terminate-after = 5 }} (AGENTS.md § {NO_TERMINATION})"
    )
}

/// What the rules find in a config's text, one message per offending key.
fn violations(text: &str) -> Vec<String> {
    let config: Table = text
        .parse()
        .unwrap_or_else(|e| panic!("{CONFIG} is not TOML: {e}"));
    let empty = Table::new();
    let profiles = config
        .get("profile")
        .and_then(Value::as_table)
        .unwrap_or(&empty);

    // A file with no [profile.default] still has that profile, on nextest's
    // built-in settings, and every other profile inherits from it.
    let mut names: Vec<&str> = profiles.keys().map(String::as_str).collect();
    if !names.contains(&"default") {
        names.push("default");
    }

    let mut found = Vec::new();
    for profile in names {
        let chain = chain(profiles, profile);

        match resolved(&chain, "flaky-result") {
            (_, Some(value)) if value.as_str() == Some("fail") => {}
            (owner, Some(value)) => {
                found.push(unsafe_retry(
                    &format!("profile.{owner}"),
                    &value.to_string(),
                ));
            }
            (owner, None) => found.push(unsafe_retry(
                &format!("profile.{owner}"),
                "unset, and nextest's default is \"pass\"",
            )),
        }

        let (owner, slow_timeout) = resolved(&chain, "slow-timeout");
        if let Some(gap) = termination_gap(slow_timeout) {
            found.push(no_termination(&format!("profile.{owner}"), &gap));
        }

        for (owner, table) in &chain {
            let overrides = table.get("overrides").and_then(Value::as_array);
            for (index, entry) in overrides.into_iter().flatten().enumerate() {
                let place = format!("profile.{owner}.overrides[{index}]");
                // Each setting comes from the first override that sets it, so a
                // "pass" here can reach a retry another override gives.
                if let Some(value) = entry.get("flaky-result")
                    && value.as_str() != Some("fail")
                {
                    found.push(unsafe_retry(&place, &value.to_string()));
                }
                if let Some(value) = entry.get("slow-timeout")
                    && let Some(gap) = termination_gap(Some(value))
                {
                    found.push(no_termination(&place, &gap));
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

#[test]
fn the_committed_config_keeps_both_rules() {
    let text = std::fs::read_to_string(common::workspace_root().join(CONFIG))
        .unwrap_or_else(|e| panic!("read {CONFIG}: {e}"));
    let found = violations(&text);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// Each code the guard fails with has a heading in AGENTS.md, which its message
/// names.
#[test]
fn each_code_has_a_heading_in_agents_md() {
    let agents = std::fs::read_to_string(common::workspace_root().join("AGENTS.md"))
        .expect("AGENTS.md is readable");
    for code in [UNSAFE_RETRY, NO_TERMINATION] {
        assert!(
            agents.contains(&format!("\n### `{code}`\n")),
            "this guard fails with {code}, and AGENTS.md has no `### `{code}`` heading"
        );
    }
    let sample = violations("");
    for code in [UNSAFE_RETRY, NO_TERMINATION] {
        assert!(
            sample.iter().any(|message| message.starts_with(code)
                && message.ends_with(&format!("(AGENTS.md § {code})"))),
            "a {code} message must end by naming its heading: {sample:#?}"
        );
    }
}

const SAFE_DEFAULT: &str = "[profile.default]\nslow-timeout = { period = \"60s\", terminate-after = 5 }\n\
                            flaky-result = \"fail\"\n";
const TERMINATING: &str = "slow-timeout = { period = \"60s\", terminate-after = 5 }\n";

#[rstest]
#[case::flaky_result_passing(
    "[profile.ci]\nretries = 1\nflaky-result = \"pass\"\n",
    UNSAFE_RETRY,
    "profile.ci key flaky-result"
)]
#[case::override_passing_a_profile_retry(
    "[profile.ci]\nretries = 1\n\
     [[profile.ci.overrides]]\nfilter = \"test(x)\"\nflaky-result = \"pass\"\n",
    UNSAFE_RETRY,
    "profile.ci.overrides[0] key flaky-result"
)]
#[case::override_passing_a_retry_another_override_gives(
    "[[profile.ci.overrides]]\nfilter = \"test(x)\"\nretries = 2\n\
     [[profile.ci.overrides]]\nfilter = \"test(x)\"\nflaky-result = \"pass\"\n",
    UNSAFE_RETRY,
    "profile.ci.overrides[1] key flaky-result"
)]
#[case::slow_timeout_passing_what_it_ends(
    "[profile.ci]\nslow-timeout = { period = \"60s\", terminate-after = 5, on-timeout = \"pass\" }\n",
    NO_TERMINATION,
    "profile.ci key slow-timeout"
)]
#[case::slow_timeout_as_a_string(
    "[profile.ci]\nslow-timeout = \"60s\"\n",
    NO_TERMINATION,
    "profile.ci key slow-timeout"
)]
#[case::slow_timeout_with_no_terminate_after(
    "[profile.ci]\nslow-timeout = { period = \"60s\" }\n",
    NO_TERMINATION,
    "profile.ci key slow-timeout"
)]
#[case::override_slow_timeout_with_no_terminate_after(
    "[[profile.ci.overrides]]\nfilter = \"test(x)\"\nslow-timeout = \"60s\"\n",
    NO_TERMINATION,
    "profile.ci.overrides[0] key slow-timeout"
)]
fn a_seeded_violation_names_its_code_profile_and_key(
    #[case] seeded: &str,
    #[case] code: &str,
    #[case] named: &str,
) {
    let found = violations(&format!("{SAFE_DEFAULT}{seeded}"));
    assert!(
        found
            .iter()
            .any(|message| message.starts_with(code) && message.contains(named)),
        "expected {code} naming {named:?}, found {found:#?}"
    );
}

/// A retry can come from `--retries` or `NEXTEST_RETRIES` as well as the file, so
/// a profile that does not resolve `flaky-result = "fail"` is unsafe even with no
/// `retries` key anywhere.
#[rstest]
#[case::unset_with_no_retries_anywhere("")]
#[case::unset_under_a_profile_that_retries("[profile.ci]\nretries = 1\n")]
#[case::passing_on_default("flaky-result = \"pass\"\n")]
fn a_default_profile_not_failing_flaky_tests_is_an_unsafe_retry(#[case] seeded: &str) {
    let found = violations(&format!("[profile.default]\n{TERMINATING}{seeded}"));
    assert!(
        found.iter().any(|message| message.starts_with(UNSAFE_RETRY)
            && message.contains("profile.default key flaky-result")),
        "{found:#?}"
    );
}

/// A setting a profile inherits is reported once, under the profile that writes
/// it, and an unset one says so rather than naming nextest's default as a value.
#[test]
fn an_inherited_setting_is_reported_where_it_is_written() {
    let found =
        violations("[profile.default]\nslow-timeout = \"60s\"\n[profile.ci]\nretries = 1\n");
    let named = |key: &str| found.iter().filter(|message| message.contains(key)).count();
    assert_eq!(named("profile.default key slow-timeout"), 1, "{found:#?}");
    assert_eq!(named("profile.ci key slow-timeout"), 0, "{found:#?}");
    assert_eq!(named("profile.default key flaky-result"), 1, "{found:#?}");
    assert_eq!(named("profile.ci key flaky-result"), 0, "{found:#?}");
    assert!(
        found
            .iter()
            .any(|message| message.contains("profile.default key flaky-result: unset")),
        "{found:#?}"
    );
    assert!(
        found.iter().any(|message| message
            .contains("profile.default key slow-timeout: \"60s\" has no terminate-after")),
        "{found:#?}"
    );
}

#[rstest]
#[case::no_config_at_all("")]
#[case::a_default_profile_with_no_slow_timeout("[profile.default]\nfail-fast = false\n")]
fn a_profile_on_the_built_in_slow_timeout_has_no_termination(#[case] text: &str) {
    let found = violations(text);
    assert!(
        found
            .iter()
            .any(|message| message.starts_with(NO_TERMINATION)
                && message.contains("profile.default key slow-timeout: unset")),
        "{found:#?}"
    );
}

#[rstest]
#[case::retry_failing_flaky("[profile.ci]\nretries = 1\nflaky-result = \"fail\"\n")]
#[case::retry_table_failing_flaky(
    "[profile.ci]\nretries = { backoff = \"fixed\", count = 2, delay = \"1s\" }\n\
     flaky-result = \"fail\"\n"
)]
#[case::flaky_inherited_from_default("[profile.ci]\nretries = 1\n")]
#[case::flaky_inherited_through_a_named_parent(
    "[profile.ci]\nflaky-result = \"fail\"\n[profile.other]\ninherits = \"ci\"\nretries = 1\n"
)]
#[case::override_retry_under_a_failing_profile(
    "[[profile.ci.overrides]]\nfilter = \"test(x)\"\nretries = 2\n"
)]
#[case::no_retries("[profile.ci]\nfail-fast = false\n")]
#[case::inherited_termination("[profile.ci]\nretries = 0\n")]
#[case::override_with_its_own_termination(
    "[[profile.ci.overrides]]\nfilter = \"test(x)\"\n\
     slow-timeout = { period = \"10s\", terminate-after = 2 }\n"
)]
fn a_config_within_the_rules_passes(#[case] seeded: &str) {
    let found = violations(&format!("{SAFE_DEFAULT}{seeded}"));
    assert!(found.is_empty(), "{found:#?}");
}
