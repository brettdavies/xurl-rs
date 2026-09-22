//! Reads X's legacy post vocabulary under the names the vendored spec uses.
//!
//! X renamed its post vocabulary and the wire followed per endpoint, so a
//! response can spell a key `edit_history_tweet_ids` where the spec says
//! `edit_history_post_ids`. [`normalize`] runs inside `decode()`, the path
//! every typed response takes, before serde parses the body:
//!
//! ```text
//! decode(value)
//!   ├─ empty-body and errors-only guards
//!   ├─ normalize(&mut value)
//!   │    each object, each legacy key k in ADMITTED or EXCLUDED:
//!   │      current spelling present? ── yes ─► drop k, report (collision)
//!   │                                 └─ no ──► k admitted? ── yes ─► rename k, report
//!   │                                                        └─ no ──► leave k
//!   │    reports: one DEBUG event per legacy key per response, on VOCABULARY_TARGET
//!   └─ serde_json::from_value::<T>
//! ```
//!
//! `build.rs` derives both tables from the vendored spec. An excluded
//! pair's legacy spelling is a current name elsewhere in the spec, so a lone
//! one stays as sent; both spellings in one object can only be the rename,
//! which the build checks. Raw requests never reach this module.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

/// Target of the events reporting each legacy key a typed response was read
/// under its current name: one `DEBUG` event per legacy key per response,
/// carrying `legacy` and `normalized` (the two spellings), `value_type` (the
/// JSON type of the value X sent under the legacy key), `value_len` (its
/// length, for a string, array, or object), and `collision` (whether the
/// object also carried the current spelling, so the legacy key was dropped).
/// No event carries a value, an id, or a request path.
pub const VOCABULARY_TARGET: &str = "xdk::vocabulary";

mod generated {
    include!(concat!(env!("OUT_DIR"), "/vocabulary.rs"));
}

/// Rewrites every object under `value` into the spec's vocabulary. An
/// admitted legacy key is renamed to its current spelling; a legacy key
/// whose current spelling the same object carries is dropped. Values move
/// with their keys and are never altered. Each legacy key is reported once
/// per call, however many objects carry it.
pub(crate) fn normalize(value: &mut Value) {
    walk(value, &mut BTreeSet::new());
}

/// Normalizes every object under `value`; `reported` holds the legacy keys
/// this response has already reported.
fn walk(value: &mut Value, reported: &mut BTreeSet<&'static str>) {
    match value {
        Value::Object(map) => {
            normalize_keys(map, reported);
            for child in map.values_mut() {
                walk(child, reported);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk(item, reported);
            }
        }
        _ => {}
    }
}

/// Applies both tables to one object's own keys.
fn normalize_keys(map: &mut Map<String, Value>, reported: &mut BTreeSet<&'static str>) {
    let admitted = generated::ADMITTED.iter().map(|pair| (pair, true));
    let excluded = generated::EXCLUDED.iter().map(|pair| (pair, false));
    for (&(legacy, current), renamed) in admitted.chain(excluded) {
        let Some(sent) = map.get(legacy) else {
            continue;
        };
        let collision = map.contains_key(current);
        if !collision && !renamed {
            continue;
        }
        if reported.insert(legacy) {
            report(legacy, current, sent, collision);
        }
        let moved = map.remove(legacy);
        if !collision && let Some(value) = moved {
            map.insert(current.to_owned(), value);
        }
    }
}

/// Emits one event on [`VOCABULARY_TARGET`]. Every field is a schema
/// identifier: the spellings come from the spec-derived tables, and the
/// value X sent contributes only its JSON type and length.
fn report(legacy: &'static str, normalized: &'static str, sent: &Value, collision: bool) {
    let (value_type, value_len) = shape(sent);
    tracing::debug!(
        target: VOCABULARY_TARGET,
        legacy,
        normalized,
        value_type,
        value_len,
        collision
    );
}

/// The JSON type name of `value`, and its length for a string (in
/// characters), an array, or an object.
fn shape(value: &Value) -> (&'static str, Option<usize>) {
    match value {
        Value::Null => ("null", None),
        Value::Bool(_) => ("boolean", None),
        Value::Number(_) => ("number", None),
        Value::String(text) => ("string", Some(text.chars().count())),
        Value::Array(items) => ("array", Some(items.len())),
        Value::Object(map) => ("object", Some(map.len())),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::sync::{Arc, Mutex};

    use serde_json::{Value, json};
    use tracing::field::{Field, Visit};
    use tracing::{Event, Metadata, Subscriber, span};

    use super::super::types::{
        BookmarkedResult, Post, RepostedResult, User, decode, deserialize_response,
    };
    use super::VOCABULARY_TARGET;

    #[test]
    fn edit_history_tweet_ids_reads_as_edit_history_post_ids_with_its_value_untouched() {
        for ids in [json!([""]), json!([]), json!(["2101712260468977783"])] {
            let post = deserialize_response::<Post>(json!({
                "data": {"id": "1", "text": "t", "edit_history_tweet_ids": ids}
            }))
            .expect("decodes");
            assert_eq!(post.data.extra.get("edit_history_post_ids"), Some(&ids));
            assert!(
                !post.data.extra.contains_key("edit_history_tweet_ids"),
                "extra: {:?}",
                post.data.extra
            );
        }
    }

    #[test]
    fn legacy_keys_are_renamed_at_every_depth() {
        let body = decode::<Value>(json!({
            "data": [
                {"id": "1", "public_metrics": {"retweet_count": 3}, "edit_history_tweet_ids": ["1"]},
                {"id": "2", "referenced_tweets": [{"id": "3", "type": "quoted"}]}
            ],
            "includes": {
                "tweets": [{"id": "3", "note_tweet": {"text": "long"}}],
                "users": [{"id": "9", "pinned_tweet_id": "3", "most_recent_tweet_id": "3"}]
            }
        }))
        .expect("decodes");

        assert_eq!(
            body,
            json!({
                "data": [
                    {"id": "1", "public_metrics": {"repost_count": 3}, "edit_history_post_ids": ["1"]},
                    {"id": "2", "referenced_posts": [{"id": "3", "type": "quoted"}]}
                ],
                "includes": {
                    "posts": [{"id": "3", "note_post": {"text": "long"}}],
                    "users": [{"id": "9", "pinned_post_id": "3", "most_recent_post_id": "3"}]
                }
            })
        );
    }

    #[test]
    fn keys_outside_the_table_pass_through_unchanged() {
        let body = json!({"data": {"retweeted": true, "tweet_mode": "extended", "future": [1]}});
        assert_eq!(decode::<Value>(body.clone()).expect("decodes"), body);

        let reposted = deserialize_response::<RepostedResult>(body).expect("decodes");
        assert!(reposted.data.retweeted);
        assert_eq!(reposted.data.extra["tweet_mode"], "extended");
    }

    #[test]
    fn a_lone_tweet_id_is_never_renamed() {
        let bookmarked = deserialize_response::<BookmarkedResult>(json!({
            "data": {"bookmarked": true, "tweet_id": "42"}
        }))
        .expect("decodes");
        assert_eq!(bookmarked.data.extra.get("tweet_id"), Some(&json!("42")));
        assert!(!bookmarked.data.extra.contains_key("post_id"));
    }

    #[test]
    fn both_spellings_of_a_declared_field_keep_the_current_value() {
        let post = deserialize_response::<Post>(json!({
            "data": {"id": "1", "text": "t", "public_metrics": {"retweet_count": 1, "repost_count": 2}}
        }))
        .expect("both spellings in one object decode");
        let metrics = post.data.public_metrics.expect("public_metrics present");
        assert_eq!(metrics.repost_count, 2);
        assert!(metrics.extra.is_empty(), "extra: {:?}", metrics.extra);
    }

    #[test]
    fn both_spellings_of_an_undeclared_key_keep_the_current_value() {
        let post = deserialize_response::<Post>(json!({
            "data": {
                "id": "1",
                "text": "t",
                "edit_history_tweet_ids": ["old"],
                "edit_history_post_ids": ["new"]
            }
        }))
        .expect("decodes");
        assert_eq!(
            post.data.extra.get("edit_history_post_ids"),
            Some(&json!(["new"]))
        );
        assert!(!post.data.extra.contains_key("edit_history_tweet_ids"));
    }

    #[test]
    fn both_spellings_of_an_excluded_pair_keep_the_current_value() {
        let user = deserialize_response::<User>(json!({
            "data": {
                "id": "1",
                "name": "n",
                "username": "u",
                "public_metrics": {"tweet_count": 5, "post_count": 7}
            }
        }))
        .expect("both spellings of an excluded pair decode");
        let metrics = user.data.public_metrics.expect("public_metrics present");
        assert_eq!(metrics.post_count, 7);
        assert!(metrics.extra.is_empty(), "extra: {:?}", metrics.extra);
    }

    #[test]
    fn a_lone_tweet_count_still_fills_post_count_through_its_alias() {
        let user = deserialize_response::<User>(json!({
            "data": {"id": "1", "name": "n", "username": "u", "public_metrics": {"tweet_count": 5}}
        }))
        .expect("decodes");
        assert_eq!(user.data.public_metrics.map(|m| m.post_count), Some(5));
    }

    // ── Events ──────────────────────────────────────────────────────

    /// One captured event: the fields its callsite declares, and the ones
    /// it recorded, with their values.
    #[derive(Debug, Clone, PartialEq)]
    struct Captured {
        declared: Vec<&'static str>,
        recorded: Vec<(&'static str, String)>,
    }

    /// Collects every event on the vocabulary target.
    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<Captured>>>);

    impl Subscriber for Capture {
        fn enabled(&self, metadata: &Metadata<'_>) -> bool {
            metadata.target() == VOCABULARY_TARGET
        }
        fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
            span::Id::from_u64(1)
        }
        fn record(&self, _: &span::Id, _: &span::Record<'_>) {}
        fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}
        fn event(&self, event: &Event<'_>) {
            let mut recorded = Recorded::default();
            event.record(&mut recorded);
            let declared = event.metadata().fields().iter().map(|f| f.name()).collect();
            self.0.lock().expect("capture lock").push(Captured {
                declared,
                recorded: recorded.0,
            });
        }
        fn enter(&self, _: &span::Id) {}
        fn exit(&self, _: &span::Id) {}
    }

    #[derive(Default)]
    struct Recorded(Vec<(&'static str, String)>);

    impl Visit for Recorded {
        fn record_str(&mut self, field: &Field, value: &str) {
            self.0.push((field.name(), value.to_string()));
        }
        fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
            self.0.push((field.name(), format!("{value:?}")));
        }
    }

    /// Decodes `body` under a capturing subscriber and returns its events.
    fn events(body: Value) -> Vec<Captured> {
        let capture = Capture::default();
        tracing::subscriber::with_default(capture.clone(), || {
            decode::<Value>(body).expect("decodes");
        });
        capture.0.lock().expect("capture lock").clone()
    }

    const FIVE: [&str; 5] = [
        "legacy",
        "normalized",
        "value_type",
        "value_len",
        "collision",
    ];

    fn recorded(fields: &[(&'static str, &str)]) -> Vec<(&'static str, String)> {
        fields
            .iter()
            .map(|(name, value)| (*name, (*value).to_string()))
            .collect()
    }

    #[test]
    fn a_renamed_key_reports_one_event_with_exactly_the_five_fields() {
        let events = events(json!({"data": {"id": "1", "edit_history_tweet_ids": [""]}}));

        assert_eq!(
            events,
            [Captured {
                declared: FIVE.to_vec(),
                recorded: recorded(&[
                    ("legacy", "edit_history_tweet_ids"),
                    ("normalized", "edit_history_post_ids"),
                    ("value_type", "array"),
                    ("value_len", "1"),
                    ("collision", "false"),
                ]),
            }]
        );
    }

    #[test]
    fn a_key_repeated_across_posts_reports_once_per_response() {
        let body = json!({"data": [
            {"id": "1", "edit_history_tweet_ids": ["1"]},
            {"id": "2", "edit_history_tweet_ids": ["2"]},
            {"id": "3", "edit_history_tweet_ids": ["3"]}
        ]});

        assert_eq!(events(body.clone()).len(), 1);
        assert_eq!(events(body).len(), 1, "a second response reports again");
    }

    #[test]
    fn a_collision_reports_one_event_marked_as_one() {
        let events = events(json!({
            "data": {"public_metrics": {"retweet_count": 1, "repost_count": 2}}
        }));

        let fields: Vec<_> = events.iter().map(|e| e.recorded.clone()).collect();
        assert_eq!(
            fields,
            [recorded(&[
                ("legacy", "retweet_count"),
                ("normalized", "repost_count"),
                ("value_type", "number"),
                ("collision", "true"),
            ])]
        );
    }

    #[test]
    fn an_excluded_pair_reports_its_collision_and_never_a_lone_legacy_key() {
        let collided = events(json!({
            "data": {"public_metrics": {"tweet_count": 5, "post_count": 7}}
        }));
        let fields: Vec<_> = collided.iter().map(|e| e.recorded.clone()).collect();
        assert_eq!(
            fields,
            [recorded(&[
                ("legacy", "tweet_count"),
                ("normalized", "post_count"),
                ("value_type", "number"),
                ("collision", "true"),
            ])]
        );

        assert_eq!(
            events(json!({"data": {"public_metrics": {"tweet_count": 5}, "tweet_id": "9"}})),
            []
        );
    }

    #[test]
    fn a_string_reports_its_length_in_characters() {
        let events = events(json!({"data": {"pinned_tweet_id": "ñ12"}}));
        let fields: Vec<_> = events.iter().map(|e| e.recorded.clone()).collect();
        assert_eq!(
            fields,
            [recorded(&[
                ("legacy", "pinned_tweet_id"),
                ("normalized", "pinned_post_id"),
                ("value_type", "string"),
                ("value_len", "3"),
                ("collision", "false"),
            ])]
        );
    }

    #[test]
    fn keys_outside_the_table_report_nothing() {
        assert_eq!(
            events(json!({"data": {"retweeted": true, "tweet_mode": "extended"}})),
            []
        );
    }
}
