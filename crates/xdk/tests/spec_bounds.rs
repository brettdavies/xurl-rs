//! Every bound a validator in `xdk::api::shortcuts` checks is the one the
//! vendored spec sets, so a refresh that moves a bound fails here rather than
//! leaving the validator refusing what X accepts, or passing what X refuses.
//! Every integer constant `shortcuts.rs` declares has a row here or a reason
//! it mirrors no bound, so a bound added later cannot skip the check.

use serde_json::Value;
use xdk::api::shortcuts::{
    ALT_TEXT_MAX_CHARS, BOOKMARK_FOLDER_NAME_MAX_CHARS, BROADCAST_CHAT_MAX_CHARS,
    CALL_DESCRIPTION_MAX_CHARS, CALL_DURATION_MAX_MINUTES, CALL_RECURRENCE_MAX_CHARS,
    CALL_TIME_ZONE_MAX_CHARS, CALL_TITLE_MAX_CHARS, CHANGED_CALL_INVITEES_MAX,
    DM_GROUP_PARTICIPANTS_MAX, DM_GROUP_PARTICIPANTS_MIN, LIST_DESCRIPTION_MAX_CHARS,
    LIST_NAME_MAX_CHARS, LOOKUP_IDS_MAX, NEW_CALL_INVITEES_MAX, POST_MEDIA_MAX,
    USER_SEARCH_QUERY_MAX_CHARS, WEBHOOK_URL_MAX_CHARS,
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
    /// A query parameter of an operation, by path and name.
    Query(&'static str, &'static str),
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
            Site::Query("/2/users", "ids"),
            Site::Query("/2/users/by", "usernames"),
            Site::Query("/2/tweets", "ids"),
            Site::Query("/2/media", "media_keys"),
            Site::Query("/2/spaces", "ids"),
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
        keyword: "pattern",
        sites: &[Site::Query("/2/users/search", "query")],
    },
];

/// The bound `keyword` sets in `schema`. For `pattern`, the upper count of
/// the pattern's last `{m,n}` quantifier.
fn stated_bound(schema: &Value, keyword: &str) -> Option<u64> {
    if keyword != "pattern" {
        return schema[keyword].as_u64();
    }
    let pattern = schema["pattern"].as_str()?;
    let quantifier = &pattern[pattern.rfind('{')? + 1..];
    quantifier[..quantifier.find('}')?]
        .split(',')
        .nth(1)?
        .parse()
        .ok()
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
        Site::Query(path, name) => {
            let shared = &spec["components"]["parameters"];
            let found = spec["paths"][*path]["get"]["parameters"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|parameter| match parameter["$ref"].as_str() {
                    Some(reference) => &shared[reference.rsplit('/').next().unwrap_or_default()],
                    None => parameter,
                })
                .find(|parameter| parameter["in"] == "query" && parameter["name"] == *name)
                .map(|parameter| &parameter["schema"]);
            (format!("GET {path} query parameter {name}"), found)
        }
    }
}

#[test]
fn every_validator_bound_is_the_one_the_vendored_spec_sets() {
    let spec = spec();
    let mut drift = Vec::new();
    for bound in BOUNDS {
        for site in bound.sites {
            let (at, schema) = schema_at(&spec, site);
            let stated = schema.and_then(|schema| stated_bound(schema, bound.keyword));
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
