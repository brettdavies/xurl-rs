//! `.config/nextest.toml` decides what a green test run means, so this holds
//! every profile to two rules. A retry never turns a test green: nextest's
//! default `flaky-result` passes a test that fails and then passes. A hung test
//! is ended: nextest's built-in `slow-timeout` never ends one, so the runner
//! kills the job and the log never names the test.
//!
//! A profile inherits from `default` unless it says otherwise, and each setting
//! resolves on its own, so the rules read what a profile ends up with rather
//! than what it writes.

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

fn resolved<'a>(chain: &[(&str, &'a Table)], key: &str) -> Option<&'a Value> {
    chain.iter().find_map(|(_, table)| table.get(key))
}

/// `retries = 2` or `retries = { backoff = "fixed", count = 2, delay = "1s" }`.
fn retry_count(retries: &Value) -> i64 {
    match retries {
        Value::Integer(count) => *count,
        other => other.get("count").and_then(Value::as_integer).unwrap_or(0),
    }
}

/// `slow-timeout = "60s"` has no way to say when to end a test, and
/// `on-timeout = "pass"` ends one and counts it as passed.
fn terminates(slow_timeout: &Value) -> bool {
    slow_timeout.get("terminate-after").is_some()
        && slow_timeout.get("on-timeout").and_then(Value::as_str) != Some("pass")
}

fn unsafe_retry(place: &str, key: &str, flaky_result: &str) -> String {
    format!(
        "{UNSAFE_RETRY}: {CONFIG} {place} key {key}: a retry under flaky-result {flaky_result:?} \
         lets a test that fails and then passes turn the run green; set flaky-result = \"fail\" \
         (AGENTS.md § Testing)"
    )
}

fn no_termination(place: &str) -> String {
    format!(
        "{NO_TERMINATION}: {CONFIG} {place} key slow-timeout: without terminate-after a hung \
         test runs until the runner kills the job, unnamed, and on-timeout = \"pass\" counts it \
         as passed; set slow-timeout = {{ period = \"60s\", terminate-after = 5 }} (AGENTS.md § \
         Testing)"
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
        let place = format!("profile.{profile}");
        let flaky_result = resolved(&chain, "flaky-result")
            .and_then(Value::as_str)
            .unwrap_or("pass");
        let retries = resolved(&chain, "retries").map_or(0, retry_count);
        if retries > 0 && flaky_result != "fail" {
            found.push(unsafe_retry(&place, "retries", flaky_result));
        }
        if !resolved(&chain, "slow-timeout").is_some_and(terminates) {
            found.push(no_termination(&place));
        }

        for (owner, table) in &chain {
            let overrides = table.get("overrides").and_then(Value::as_array);
            for (index, entry) in overrides.into_iter().flatten().enumerate() {
                let place = format!("profile.{owner}.overrides[{index}]");
                let own_retries = entry.get("retries").map_or(0, retry_count);
                let own_flaky = entry.get("flaky-result").and_then(Value::as_str);
                // Each setting comes from the first override that sets it, so a
                // "pass" here can reach a retry that another override gives.
                if own_flaky == Some("pass") {
                    found.push(unsafe_retry(&place, "flaky-result", "pass"));
                } else if own_retries > 0 && own_flaky.unwrap_or(flaky_result) != "fail" {
                    found.push(unsafe_retry(&place, "retries", flaky_result));
                }
                if entry.get("slow-timeout").is_some_and(|v| !terminates(v)) {
                    found.push(no_termination(&place));
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

const SAFE_DEFAULT: &str =
    "[profile.default]\nslow-timeout = { period = \"60s\", terminate-after = 5 }\n";

#[rstest]
#[case::retry_with_no_flaky_result(
    "[profile.ci]\nretries = 1\n",
    UNSAFE_RETRY,
    "profile.ci key retries"
)]
#[case::retry_with_flaky_passing(
    "[profile.ci]\nretries = 1\nflaky-result = \"pass\"\n",
    UNSAFE_RETRY,
    "profile.ci key retries"
)]
#[case::retry_table_with_no_flaky_result(
    "[profile.ci]\nretries = { backoff = \"fixed\", count = 2, delay = \"1s\" }\n",
    UNSAFE_RETRY,
    "profile.ci key retries"
)]
#[case::retry_inherited_from_default(
    "retries = 1\n[profile.ci]\nfail-fast = false\n",
    UNSAFE_RETRY,
    "profile.ci key retries"
)]
#[case::override_retry_with_no_flaky_result(
    "[[profile.ci.overrides]]\nfilter = \"test(x)\"\nretries = 2\n",
    UNSAFE_RETRY,
    "profile.ci.overrides[0] key retries"
)]
#[case::override_passing_a_profile_retry(
    "[profile.ci]\nretries = 1\nflaky-result = \"fail\"\n\
     [[profile.ci.overrides]]\nfilter = \"test(x)\"\nflaky-result = \"pass\"\n",
    UNSAFE_RETRY,
    "profile.ci.overrides[0] key flaky-result"
)]
#[case::override_passing_a_retry_another_override_gives(
    "[profile.ci]\nflaky-result = \"fail\"\n\
     [[profile.ci.overrides]]\nfilter = \"test(x)\"\nretries = 2\n\
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

#[rstest]
#[case::no_config_at_all("")]
#[case::a_default_profile_with_no_slow_timeout("[profile.default]\nfail-fast = false\n")]
fn a_profile_on_the_built_in_slow_timeout_has_no_termination(#[case] text: &str) {
    let found = violations(text);
    assert!(
        found
            .iter()
            .any(|message| message.starts_with(NO_TERMINATION)
                && message.contains("profile.default key slow-timeout")),
        "{found:#?}"
    );
}

#[rstest]
#[case::retry_failing_flaky("[profile.ci]\nretries = 1\nflaky-result = \"fail\"\n")]
#[case::retry_table_failing_flaky(
    "[profile.ci]\nretries = { backoff = \"fixed\", count = 2, delay = \"1s\" }\n\
     flaky-result = \"fail\"\n"
)]
#[case::flaky_inherited_from_default(
    "[profile.ci]\nflaky-result = \"fail\"\n[profile.other]\ninherits = \"ci\"\nretries = 1\n"
)]
#[case::override_retry_under_a_failing_profile(
    "[profile.ci]\nflaky-result = \"fail\"\n\
     [[profile.ci.overrides]]\nfilter = \"test(x)\"\nretries = 2\n"
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
