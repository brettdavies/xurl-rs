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
//!   │      current spelling present? ── yes ─► drop k
//!   │                                 └─ no ──► k admitted? ── yes ─► rename k
//!   │                                                        └─ no ──► leave k
//!   └─ serde_json::from_value::<T>
//! ```
//!
//! `build.rs` derives both tables from the vendored spec. An excluded
//! pair's legacy spelling is a current name elsewhere in the spec, so a lone
//! one stays as sent; both spellings in one object can only be the rename,
//! which the build checks. Raw requests never reach this module.

use serde_json::{Map, Value};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/vocabulary.rs"));
}

/// Rewrites every object under `value` into the spec's vocabulary. An
/// admitted legacy key is renamed to its current spelling; a legacy key
/// whose current spelling the same object carries is dropped. Values move
/// with their keys and are never altered.
pub(crate) fn normalize(value: &mut Value) {
    match value {
        Value::Object(map) => {
            normalize_keys(map);
            map.values_mut().for_each(normalize);
        }
        Value::Array(items) => items.iter_mut().for_each(normalize),
        _ => {}
    }
}

/// Applies both tables to one object's own keys.
fn normalize_keys(map: &mut Map<String, Value>) {
    let admitted = generated::ADMITTED.iter().map(|pair| (pair, true));
    let excluded = generated::EXCLUDED.iter().map(|pair| (pair, false));
    for (&(legacy, current), renamed) in admitted.chain(excluded) {
        if !map.contains_key(legacy) {
            continue;
        }
        if map.contains_key(current) {
            map.remove(legacy);
        } else if renamed && let Some(value) = map.remove(legacy) {
            map.insert(current.to_owned(), value);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::super::types::{
        BookmarkedResult, Post, RepostedResult, User, decode, deserialize_response,
    };

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
}
