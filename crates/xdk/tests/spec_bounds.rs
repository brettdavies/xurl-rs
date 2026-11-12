//! Every bound a validator in `xdk::api::shortcuts` checks is the one the
//! vendored spec sets, so a refresh that moves a bound fails here rather than
//! leaving the validator refusing what X accepts, or passing what X refuses.
//! Every integer constant `shortcuts.rs` declares has a row here or a reason
//! it mirrors no bound, so a bound added later cannot skip the check. A
//! validator that checks an input's characters agrees with the spec's
//! `pattern` on every sample input below.

use serde_json::Value;
use xdk::api::shortcuts::{
    ALT_TEXT_MAX_CHARS, BOOKMARK_FOLDER_NAME_MAX_CHARS, BROADCAST_CHAT_MAX_CHARS,
    CALL_DESCRIPTION_MAX_CHARS, CALL_DURATION_MAX_MINUTES, CALL_RECURRENCE_MAX_CHARS,
    CALL_TIME_ZONE_MAX_CHARS, CALL_TITLE_MAX_CHARS, CHANGED_CALL_INVITEES_MAX,
    DM_GROUP_PARTICIPANTS_MAX, DM_GROUP_PARTICIPANTS_MIN, LIST_DESCRIPTION_MAX_CHARS,
    LIST_NAME_MAX_CHARS, LOOKUP_IDS_MAX, NEW_CALL_INVITEES_MAX, POST_MEDIA_MAX,
    USER_SEARCH_QUERY_MAX_CHARS, WEBHOOK_URL_MAX_CHARS, validate_broadcast_id,
    validate_clip_post_id, validate_space_id, validate_user_search_query,
};

/// Integer constants in `shortcuts.rs` that mirror no spec bound, each with
/// the reason.
const NOT_SPEC_BOUNDS: &[(&str, &str)] = &[(
    "POST_BODY_MAX_CHARS",
    "deprecated: X sets a post's length per account, and the spec states none",
)];

/// Where the spec states a bound.
enum Site {
    /// A property of a component schema.
    Property(&'static str, &'static str),
    /// A query parameter of an operation, `METHOD /path`, by name.
    Query(&'static str, &'static str),
    /// A path parameter of an operation, `METHOD /path`, by name.
    Path(&'static str, &'static str),
}

/// A constant, its value, the spec keyword it mirrors, and each place the
/// spec states that keyword for a request the validator guards.
struct Bound {
    constant: &'static str,
    value: u64,
    keyword: &'static str,
    sites: &'static [Site],
}

#[allow(clippy::cast_possible_truncation)]
const fn n(value: usize) -> u64 {
    value as u64
}

const BOUNDS: &[Bound] = &[
    Bound {
        constant: "POST_MEDIA_MAX",
        value: n(POST_MEDIA_MAX),
        keyword: "maxItems",
        sites: &[Site::Property("CreatePostsMedia", "media_ids")],
    },
    Bound {
        constant: "DM_GROUP_PARTICIPANTS_MIN",
        value: n(DM_GROUP_PARTICIPANTS_MIN),
        keyword: "minItems",
        sites: &[Site::Property(
            "CreateDirectMessagesConversationRequest",
            "participant_ids",
        )],
    },
    Bound {
        constant: "DM_GROUP_PARTICIPANTS_MAX",
        value: n(DM_GROUP_PARTICIPANTS_MAX),
        keyword: "maxItems",
        sites: &[Site::Property(
            "CreateDirectMessagesConversationRequest",
            "participant_ids",
        )],
    },
    Bound {
        constant: "BROADCAST_CHAT_MAX_CHARS",
        value: n(BROADCAST_CHAT_MAX_CHARS),
        keyword: "maxLength",
        sites: &[Site::Property("SendBroadcastChatRequest", "text")],
    },
    Bound {
        constant: "LIST_NAME_MAX_CHARS",
        value: n(LIST_NAME_MAX_CHARS),
        keyword: "maxLength",
        sites: &[
            Site::Property("CreateListsRequest", "name"),
            Site::Property("UpdateListsRequest", "name"),
        ],
    },
    Bound {
        constant: "LIST_DESCRIPTION_MAX_CHARS",
        value: n(LIST_DESCRIPTION_MAX_CHARS),
        keyword: "maxLength",
        sites: &[
            Site::Property("CreateListsRequest", "description"),
            Site::Property("UpdateListsRequest", "description"),
        ],
    },
    Bound {
        constant: "LOOKUP_IDS_MAX",
        value: n(LOOKUP_IDS_MAX),
        keyword: "maxItems",
        sites: &[
            Site::Query("GET /2/users", "ids"),
            Site::Query("GET /2/users/by", "usernames"),
            Site::Query("GET /2/tweets", "ids"),
            Site::Query("GET /2/media", "media_keys"),
            Site::Query("GET /2/spaces", "ids"),
        ],
    },
    Bound {
        constant: "ALT_TEXT_MAX_CHARS",
        value: n(ALT_TEXT_MAX_CHARS),
        keyword: "maxLength",
        sites: &[Site::Property("CreateMediaMetadataMetadataAltText", "text")],
    },
    Bound {
        constant: "WEBHOOK_URL_MAX_CHARS",
        value: n(WEBHOOK_URL_MAX_CHARS),
        keyword: "maxLength",
        sites: &[Site::Property("CreateWebhooksRequest", "url")],
    },
    Bound {
        constant: "BOOKMARK_FOLDER_NAME_MAX_CHARS",
        value: n(BOOKMARK_FOLDER_NAME_MAX_CHARS),
        keyword: "maxLength",
        sites: &[Site::Property("CreateUsersBookmarkFolderRequest", "name")],
    },
    Bound {
        constant: "CALL_TITLE_MAX_CHARS",
        value: n(CALL_TITLE_MAX_CHARS),
        keyword: "maxLength",
        sites: &[
            Site::Property("ScheduleCallRequest", "title"),
            Site::Property("UpdateScheduledCallRequest", "title"),
        ],
    },
    Bound {
        constant: "CALL_DESCRIPTION_MAX_CHARS",
        value: n(CALL_DESCRIPTION_MAX_CHARS),
        keyword: "maxLength",
        sites: &[
            Site::Property("ScheduleCallRequest", "description"),
            Site::Property("UpdateScheduledCallRequest", "description"),
        ],
    },
    Bound {
        constant: "CALL_RECURRENCE_MAX_CHARS",
        value: n(CALL_RECURRENCE_MAX_CHARS),
        keyword: "maxLength",
        sites: &[
            Site::Property("ScheduleCallRequest", "recurrence"),
            Site::Property("UpdateScheduledCallRequest", "recurrence"),
        ],
    },
    Bound {
        constant: "CALL_TIME_ZONE_MAX_CHARS",
        value: n(CALL_TIME_ZONE_MAX_CHARS),
        keyword: "maxLength",
        sites: &[
            Site::Property("ScheduleCallRequest", "time_zone"),
            Site::Property("UpdateScheduledCallRequest", "time_zone"),
        ],
    },
    Bound {
        constant: "CALL_DURATION_MAX_MINUTES",
        value: CALL_DURATION_MAX_MINUTES as u64,
        keyword: "maximum",
        sites: &[
            Site::Property("ScheduleCallRequest", "duration_minutes"),
            Site::Property("UpdateScheduledCallRequest", "duration_minutes"),
        ],
    },
    Bound {
        constant: "NEW_CALL_INVITEES_MAX",
        value: n(NEW_CALL_INVITEES_MAX),
        keyword: "maxItems",
        sites: &[Site::Property("ScheduleCallRequest", "invitee_ids")],
    },
    Bound {
        constant: "CHANGED_CALL_INVITEES_MAX",
        value: n(CHANGED_CALL_INVITEES_MAX),
        keyword: "maxItems",
        sites: &[Site::Property("UpdateScheduledCallRequest", "invitee_ids")],
    },
    Bound {
        constant: "USER_SEARCH_QUERY_MAX_CHARS",
        value: n(USER_SEARCH_QUERY_MAX_CHARS),
        keyword: "maxLength",
        sites: &[Site::Query("GET /2/users/search", "query")],
    },
];

/// A validator that checks an input's characters, and each place the spec
/// states the `pattern` it mirrors.
struct Pattern {
    validator: &'static str,
    check: fn(&str) -> Result<(), &'static str>,
    sites: &'static [Site],
}

const PATTERNS: &[Pattern] = &[
    Pattern {
        validator: "validate_space_id",
        check: validate_space_id,
        sites: &[
            Site::Query("GET /2/spaces", "ids"),
            Site::Path("GET /2/spaces/{id}", "id"),
            Site::Path("GET /2/spaces/{id}/tweets", "id"),
        ],
    },
    Pattern {
        validator: "validate_broadcast_id",
        check: validate_broadcast_id,
        sites: &[
            Site::Path("GET /2/broadcasts/{id}", "id"),
            Site::Path("DELETE /2/broadcasts/{id}", "id"),
            Site::Path("GET /2/broadcasts/{id}/chat", "id"),
            Site::Query("DELETE /2/broadcasts/chat/mutes/{user_id}", "broadcast_id"),
            Site::Property("MuteChatUserRequest", "broadcast_id"),
        ],
    },
    Pattern {
        validator: "validate_clip_post_id",
        check: validate_clip_post_id,
        sites: &[Site::Property("CreatePostsClip", "post_id")],
    },
    Pattern {
        validator: "validate_user_search_query",
        check: validate_user_search_query,
        sites: &[Site::Query("GET /2/users/search", "query")],
    },
];

/// Inputs on both sides of each pattern above: lengths at and past each
/// bound, letters, marks, digits, and spaces outside ASCII, and the
/// characters a pattern names or leaves out.
fn samples() -> Vec<String> {
    let mut samples: Vec<String> = [
        "",
        "1",
        "1585341984679469056",
        "1DXxyRYNejbKM",
        "1DXxyRYNejbKMxxxxxxxxxxx",
        "1DXxyRYNejbKMxxxxxxxxxxxx",
        "not-an-id!",
        "\u{e9}1",
        "rust lang",
        "rust-lang",
        "rust#lang",
        "rust_lang",
        "@jack",
        "O'Brien",
        "O\u{2019}Brien",
        "A&B",
        "\"quoted\"",
        "J. R. R.",
        "Jos\u{e9}",
        "Jose\u{301}",
        "\u{540d}\u{524d}",
        "\u{661}\u{662}",
        "full\u{3000}width",
        "tab\there",
        "line\u{2028}break",
        "\u{24b6}",
        "crab \u{1f980}",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    samples.extend([49, 50, 51].map(|n| "a".repeat(n)));
    samples.extend([19, 20].map(|n| "9".repeat(n)));
    samples
}

/// Whether `schema` accepts `input`: its `pattern`, and its `minLength` and
/// `maxLength` where set.
fn spec_accepts(schema: &Value, pattern: &regex::Regex, input: &str) -> bool {
    let chars = input.chars().count() as u64;
    pattern.is_match(input)
        && schema["minLength"].as_u64().is_none_or(|min| chars >= min)
        && schema["maxLength"].as_u64().is_none_or(|max| chars <= max)
}

fn spec() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/x-api-openapi.json");
    serde_json::from_str(&std::fs::read_to_string(&path).expect("the vendored spec"))
        .expect("the vendored spec is JSON")
}

/// The schema at `site`, and a name for it in a failure message.
fn schema_at<'a>(spec: &'a Value, site: &Site) -> (String, Option<&'a Value>) {
    match site {
        Site::Property(schema, property) => (
            format!("components.schemas.{schema}.properties.{property}"),
            spec.pointer(&format!(
                "/components/schemas/{schema}/properties/{property}"
            )),
        ),
        Site::Query(operation, name) => (
            format!("{operation} query parameter {name}"),
            parameter(spec, operation, "query", name),
        ),
        Site::Path(operation, name) => (
            format!("{operation} path parameter {name}"),
            parameter(spec, operation, "path", name),
        ),
    }
}

/// The schema of the parameter `name` in `location` on `operation`,
/// `METHOD /path`.
fn parameter<'a>(
    spec: &'a Value,
    operation: &str,
    location: &str,
    name: &str,
) -> Option<&'a Value> {
    let (method, path) = operation.split_once(' ')?;
    let shared = &spec["components"]["parameters"];
    spec["paths"][path][method.to_ascii_lowercase()]["parameters"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|parameter| match parameter["$ref"].as_str() {
            Some(reference) => &shared[reference.rsplit('/').next().unwrap_or_default()],
            None => parameter,
        })
        .find(|parameter| parameter["in"] == location && parameter["name"] == name)
        .map(|parameter| &parameter["schema"])
}

#[test]
fn every_validator_bound_is_the_one_the_vendored_spec_sets() {
    let spec = spec();
    let mut drift = Vec::new();
    for bound in BOUNDS {
        for site in bound.sites {
            let (at, schema) = schema_at(&spec, site);
            let stated = schema.and_then(|schema| schema[bound.keyword].as_u64());
            if stated != Some(bound.value) {
                drift.push(format!(
                    "{} is {}, and the spec's {} at {at} is {}",
                    bound.constant,
                    bound.value,
                    bound.keyword,
                    stated.map_or_else(|| "absent".to_string(), |v| v.to_string()),
                ));
            }
        }
    }
    assert!(
        drift.is_empty(),
        "a validator bound in crates/xdk/src/api/shortcuts.rs differs from the vendored spec:\n{}\n\
         Set each constant to the spec's value. A bound the spec lowered refuses calls that \
         passed before, so the PR's Changelog records it as a change in behavior.",
        drift.join("\n")
    );
}

#[test]
fn every_character_validator_agrees_with_the_vendored_spec_pattern() {
    let spec = spec();
    let samples = samples();
    let mut drift = Vec::new();
    for row in PATTERNS {
        for site in row.sites {
            let (at, schema) = schema_at(&spec, site);
            let schema = schema.map(|schema| {
                if schema["type"] == "array" {
                    &schema["items"]
                } else {
                    schema
                }
            });
            let Some((schema, pattern)) = schema.and_then(|schema| {
                let pattern = regex::Regex::new(schema["pattern"].as_str()?).ok()?;
                Some((schema, pattern))
            }) else {
                drift.push(format!("{at} states no pattern this test can compile"));
                continue;
            };
            for input in &samples {
                let spec_says = spec_accepts(schema, &pattern, input);
                if (row.check)(input).is_ok() != spec_says {
                    drift.push(format!(
                        "{} {} {input:?}, which the spec at {at} {}",
                        row.validator,
                        if spec_says { "refuses" } else { "accepts" },
                        if spec_says { "accepts" } else { "refuses" },
                    ));
                }
            }
        }
    }
    assert!(
        drift.is_empty(),
        "a validator in crates/xdk/src/api/shortcuts.rs disagrees with the vendored spec's \
         pattern:\n{}\nChange the validator to the spec's pattern, and the sentence for its \
         rule in crates/xurl-cli/src/cli/output/validation.rs with it. A pattern the spec \
         narrowed refuses calls that passed before, so the PR's Changelog records it as a \
         change in behavior.",
        drift.join("\n")
    );
}

#[test]
fn every_integer_constant_in_shortcuts_has_a_bound_row() {
    let source = include_str!("../src/api/shortcuts.rs");
    let declared: Vec<&str> = source
        .lines()
        .filter_map(|line| line.strip_prefix("pub const "))
        .filter_map(|rest| rest.split_once(": "))
        .filter(|(_, rest)| {
            ["usize", "u64", "u32", "u16", "u8", "isize", "i64", "i32"]
                .iter()
                .any(|ty| rest.starts_with(&format!("{ty} =")))
        })
        .map(|(name, _)| name)
        .collect();
    assert!(
        declared.contains(&"POST_MEDIA_MAX"),
        "the walk found no constants in crates/xdk/src/api/shortcuts.rs: {declared:?}"
    );

    let unpinned: Vec<&str> = declared
        .iter()
        .copied()
        .filter(|name| {
            !BOUNDS.iter().any(|bound| bound.constant == *name)
                && !NOT_SPEC_BOUNDS.iter().any(|(exempt, _)| exempt == name)
        })
        .collect();
    assert!(
        unpinned.is_empty(),
        "crates/xdk/src/api/shortcuts.rs declares integer constants this file does not hold to \
         the vendored spec: {unpinned:?}. Give each a BOUNDS row naming the spec keyword and \
         the request schema or query parameter it mirrors, or a NOT_SPEC_BOUNDS entry saying \
         why it mirrors none."
    );

    let stale: Vec<&str> = BOUNDS
        .iter()
        .map(|bound| bound.constant)
        .chain(NOT_SPEC_BOUNDS.iter().map(|(name, _)| *name))
        .filter(|name| !declared.contains(name))
        .collect();
    assert!(
        stale.is_empty(),
        "rows here name constants crates/xdk/src/api/shortcuts.rs no longer declares: {stale:?}"
    );
}
