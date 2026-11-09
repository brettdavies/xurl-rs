//! `article_blocks` turns a plain-text article body into the content blocks
//! X's Articles API takes: a blank line ends a paragraph, consecutive plain
//! lines join into one, and a line starting `#`, `##`, `###`, `-`, `1.`, or
//! `>` is a block of its own.

mod common;

use serde_json::Value;
use xdk::api::shortcuts::article_blocks;

const EVERY_MARKER: &str =
    "# One\n## Two\n### Three\n- item\n1. first\n12. twelfth\n> quoted\nplain";

fn kinds_and_texts(body: &str) -> Vec<(String, String)> {
    article_blocks(body)
        .into_iter()
        .map(|block| (block.kind.to_string(), block.text))
        .collect()
}

fn pair(kind: &str, text: &str) -> (String, String) {
    (kind.to_string(), text.to_string())
}

#[test]
fn every_block_type_has_its_marker() {
    assert_eq!(
        kinds_and_texts(EVERY_MARKER),
        vec![
            pair("header-one", "One"),
            pair("header-two", "Two"),
            pair("header-two", "Three"),
            pair("unordered-list-item", "item"),
            pair("ordered-list-item", "first"),
            pair("ordered-list-item", "twelfth"),
            pair("blockquote", "quoted"),
            pair("unstyled", "plain"),
        ]
    );
}

#[test]
fn every_block_type_is_one_the_vendored_spec_lists() {
    let spec = common::load_spec();
    let listed = spec
        .pointer("/components/schemas/ArticleCreateDraftContentStateBlocks/properties/type/enum")
        .and_then(Value::as_array)
        .expect("the spec lists the article block types");
    for block in article_blocks(EVERY_MARKER) {
        assert!(
            listed.iter().any(|kind| kind == block.kind),
            "`{}` reads as block type `{}`, which the vendored spec does not list",
            block.text,
            block.kind
        );
    }
}

#[test]
fn plain_lines_join_until_a_blank_line() {
    let body = "It is fast\nand it is safe.\n\nA second paragraph.\r\n\r\n\n";
    assert_eq!(
        kinds_and_texts(body),
        vec![
            pair("unstyled", "It is fast and it is safe."),
            pair("unstyled", "A second paragraph."),
        ]
    );
}

#[test]
fn a_marker_needs_its_space_and_a_marked_line_ends_a_paragraph() {
    let body = "#hashtag stays plain\n-5 degrees\nsee 1.5 percent\n- now a list item";
    assert_eq!(
        kinds_and_texts(body),
        vec![
            pair(
                "unstyled",
                "#hashtag stays plain -5 degrees see 1.5 percent"
            ),
            pair("unordered-list-item", "now a list item"),
        ]
    );
}

#[test]
fn a_blank_body_has_no_blocks() {
    assert!(article_blocks(" \n\n\t\n").is_empty());
}
