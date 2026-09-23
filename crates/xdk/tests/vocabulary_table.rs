//! The legacy post-vocabulary table `build.rs` derives from the vendored
//! spec, run here through the same function over the spec and over fixtures
//! built from it.

mod common;

#[path = "../codegen/vocabulary.rs"]
mod vocabulary;

use serde_json::{Value, json};
use vocabulary::{Pair, derive};

fn pairs(list: &[Pair]) -> Vec<(&str, &str)> {
    list.iter()
        .map(|pair| (pair.legacy.as_str(), pair.current.as_str()))
        .collect()
}

/// The pairs `after` holds that `before` does not.
fn added<'a>(after: &'a [Pair], before: &[Pair]) -> Vec<(&'a str, &'a str)> {
    after
        .iter()
        .filter(|pair| !before.contains(pair))
        .map(|pair| (pair.legacy.as_str(), pair.current.as_str()))
        .collect()
}

/// Declares `name` on the spec schema `schema`.
fn declare(spec: &mut Value, schema: &str, name: &str) {
    spec.pointer_mut(&format!("/components/schemas/{schema}/properties"))
        .and_then(Value::as_object_mut)
        .unwrap_or_else(|| panic!("the vendored spec has no {schema} properties"))
        .insert(name.to_string(), json!({"type": "integer"}));
}

#[test]
fn the_vendored_spec_admits_ten_pairs_and_excludes_the_two_it_still_declares() {
    let table = derive(&common::load_spec()).expect("the vendored spec derives");

    assert_eq!(
        pairs(&table.admitted),
        [
            ("cluster_tweets_results", "cluster_posts_results"),
            ("edit_history_tweet_ids", "edit_history_post_ids"),
            ("most_recent_tweet_id", "most_recent_post_id"),
            ("note_tweet", "note_post"),
            ("pinned_tweet_id", "pinned_post_id"),
            ("tweets", "posts"),
            ("previous_tweet_id", "previous_post_id"),
            ("referenced_tweets", "referenced_posts"),
            ("retweet_count", "repost_count"),
            ("total_tweet_count", "total_post_count"),
        ]
    );
    assert_eq!(
        pairs(&table.excluded),
        [("tweet_count", "post_count"), ("tweet_id", "post_id")]
    );
}

#[test]
fn a_renamed_property_added_to_the_spec_is_admitted_with_no_source_edit() {
    let baseline = derive(&common::load_spec()).expect("the vendored spec derives");
    let mut spec = common::load_spec();
    declare(&mut spec, "Post", "quote_post_count");

    let table = derive(&spec).expect("the extended spec derives");

    assert_eq!(
        added(&table.admitted, &baseline.admitted),
        [("quote_tweet_count", "quote_post_count")]
    );
    assert_eq!(table.admitted.len(), baseline.admitted.len() + 1);
    assert_eq!(pairs(&table.excluded), pairs(&baseline.excluded));
}

#[test]
fn an_added_property_whose_legacy_spelling_the_spec_declares_is_excluded() {
    let baseline = derive(&common::load_spec()).expect("the vendored spec derives");
    let mut spec = common::load_spec();
    declare(&mut spec, "Post", "quote_post_count");
    declare(&mut spec, "Trend", "quote_tweet_count");

    let table = derive(&spec).expect("the extended spec derives");

    assert_eq!(pairs(&table.admitted), pairs(&baseline.admitted));
    assert_eq!(
        added(&table.excluded, &baseline.excluded),
        [("quote_tweet_count", "quote_post_count")]
    );
    assert_eq!(table.excluded.len(), baseline.excluded.len() + 1);
}

#[test]
fn one_object_declaring_both_spellings_fails_naming_the_pair_the_object_and_the_rule() {
    let mut spec = common::load_spec();
    declare(&mut spec, "Trend", "post_count");

    let err = derive(&spec).expect_err("an object declaring both spellings fails");

    for part in [
        "/components/schemas/Trend/properties",
        "tweet_count",
        "post_count",
        "crates/xdk/codegen/vocabulary.rs",
    ] {
        assert!(err.contains(part), "the message names {part}: {err}");
    }
}

#[test]
fn an_inline_object_nested_under_a_path_is_walked_and_named_by_its_pointer() {
    let spec = json!({"paths": {"/2/tweets": {"post": {"responses": {"201": {"content": {
        "application/json": {"schema": {"properties": {"data": {"properties": {
            "edit_history_post_ids": {"type": "array"},
            "edit_history_tweet_ids": {"type": "array"}
        }}}}}
    }}}}}}});

    let err = derive(&spec).expect_err("the nested object declares both spellings");

    assert!(
        err.contains(
            "/paths/~12~1tweets/post/responses/201/content/application~1json/schema/properties/data/properties"
        ),
        "the message names the nested object by its JSON pointer: {err}"
    );
}
