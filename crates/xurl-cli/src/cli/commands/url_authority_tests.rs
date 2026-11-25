//! The webhook URL validator and the dry run's URL redaction read a URL's
//! authority the way `url::Url` does. A generated corpus of URLs, built from
//! the parts a URL parser normalizes before it reads the authority, is run
//! through all three.

use url::Url;
use xdk::api::shortcuts::validate_webhook_url;
use xdk::api::split_special_url;

use super::without_userinfo;

const USERNAME: &str = "sekrit";
const PASSWORD: &str = "hunter2";

/// Text a URL parser strips from the front of its input.
const LEADING: [&str; 6] = ["", " ", "\t", "\n", "\u{1}", " \t\n"];

/// What sits between `https:` and the authority.
const SLASHES: [&str; 10] = [
    "", "/", "//", "///", "\\", "/\\", "\\\\", "/\t/", "\t//", "//\t/",
];

/// Userinfo as written, and whether `Url` reports a username or password
/// for it: `@` alone is empty userinfo, which `Url` reports as nothing.
const USERINFO: [(&str, bool); 5] = [
    ("", false),
    ("sekrit@", true),
    ("sekrit:hunter2@", true),
    (":hunter2@", true),
    ("@", false),
];

/// A port as written, and whether `Url` reports it: it drops the scheme's
/// default.
const PORTS: [(&str, bool); 3] = [("", false), (":443", false), (":8443", true)];

/// What follows the host and port: a path, a query, or a fragment holding an
/// `@`, and a backslash, which `Url` reads as the start of the path.
const TAILS: [&str; 7] = [
    "",
    "/hook",
    "/a@b",
    "?u=a@b",
    "#f@g",
    "\\hook@x",
    "\\:8443/x",
];

struct Sample {
    raw: String,
    authority: String,
    tail: &'static str,
    userinfo_written: bool,
    userinfo_reported: bool,
    port_written: bool,
    port_reported: bool,
}

fn corpus() -> Vec<Sample> {
    let mut samples = Vec::new();
    for lead in LEADING {
        for slashes in SLASHES {
            for (userinfo, userinfo_reported) in USERINFO {
                for (port, port_reported) in PORTS {
                    for tail in TAILS {
                        samples.push(Sample {
                            raw: format!("{lead}https:{slashes}{userinfo}example.com{port}{tail}"),
                            authority: format!("{userinfo}example.com{port}"),
                            tail,
                            userinfo_written: !userinfo.is_empty(),
                            userinfo_reported,
                            port_written: !port.is_empty(),
                            port_reported,
                        });
                    }
                }
            }
        }
    }
    samples
}

/// The samples `Url` parses, each with its parse.
fn parsed_corpus() -> Vec<(Sample, Url)> {
    let all = corpus();
    let total = all.len();
    let parsed: Vec<(Sample, Url)> = all
        .into_iter()
        .filter_map(|sample| Url::parse(&sample.raw).ok().map(|url| (sample, url)))
        .collect();
    assert!(
        parsed.len() * 10 > total * 9,
        "{} of {total} samples parse",
        parsed.len()
    );
    parsed
}

fn assert_none_disagree(disagreements: &[String]) {
    assert!(
        disagreements.is_empty(),
        "{} samples disagree; the first five:\n{}",
        disagreements.len(),
        disagreements
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn the_corpus_reads_the_same_under_url() {
    let mut disagreements = Vec::new();
    for (sample, url) in parsed_corpus() {
        let reported = !url.username().is_empty() || url.password().is_some();
        if url.host_str() != Some("example.com")
            || reported != sample.userinfo_reported
            || url.port().is_some() != sample.port_reported
        {
            disagreements.push(format!("{:?}", sample.raw));
        }
    }
    assert_none_disagree(&disagreements);
}

#[test]
fn the_authority_is_the_one_the_corpus_wrote() {
    let mut disagreements = Vec::new();
    for (sample, _) in parsed_corpus() {
        let read = split_special_url(&sample.raw).authority;
        if read != sample.authority {
            disagreements.push(format!("{:?}: read {read:?}", sample.raw));
        }
    }
    assert_none_disagree(&disagreements);
}

#[test]
fn the_validator_refuses_what_url_reads_as_a_credential_or_a_port() {
    let mut disagreements = Vec::new();
    for (sample, url) in parsed_corpus() {
        let raw = &sample.raw;
        let reports_credentials = !url.username().is_empty() || url.password().is_some();
        let expected = if !raw.starts_with("https://") {
            Err("webhook-url-not-https")
        } else if sample.port_written || url.port().is_some() {
            Err("webhook-url-has-port")
        } else if sample.userinfo_written || reports_credentials {
            Err("webhook-url-has-credentials")
        } else {
            Ok(())
        };
        let answered = validate_webhook_url(raw);
        if answered != expected {
            disagreements.push(format!("{raw:?}: {answered:?}, expected {expected:?}"));
        }
    }
    assert_none_disagree(&disagreements);
}

#[test]
fn the_redaction_leaves_no_credential_in_what_it_prints() {
    let mut disagreements = Vec::new();
    for (sample, _) in parsed_corpus() {
        let raw = &sample.raw;
        let shown = without_userinfo(raw);
        let carries_credential = Url::parse(&shown)
            .is_ok_and(|again| !again.username().is_empty() || again.password().is_some());
        if shown.contains(USERNAME) || shown.contains(PASSWORD) || carries_credential {
            disagreements.push(format!("leaks a credential: {raw:?} is shown as {shown:?}"));
        } else if !shown.contains("example.com") || !shown.contains(sample.tail) {
            disagreements.push(format!("drops the rest: {raw:?} is shown as {shown:?}"));
        }
    }
    disagreements.sort_by_key(|disagreement| !disagreement.starts_with("leaks"));
    assert_none_disagree(&disagreements);
}
