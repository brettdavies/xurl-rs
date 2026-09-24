//! The streaming table derivation, run over spec fixtures.

#[path = "../codegen/streaming.rs"]
mod streaming;

use serde_json::json;

#[test]
fn only_paths_with_a_marked_operation_stream() {
    let spec = json!({
        "paths": {
            "/2/b/stream": { "get": { "x-twitter-streaming": true } },
            "/2/a/stream": { "parameters": [], "post": { "x-twitter-streaming": true } },
            "/2/b/stream/rules": { "get": {} },
            "/2/c/stream": { "get": { "x-twitter-streaming": false } },
        }
    });
    assert_eq!(
        streaming::derive(&spec).unwrap(),
        ["/2/a/stream", "/2/b/stream"]
    );
}

#[test]
fn a_spec_that_marks_nothing_fails_the_build() {
    let spec = json!({ "paths": { "/2/users/me": { "get": {} } } });
    let err = streaming::derive(&spec).unwrap_err();
    assert!(err.contains("x-twitter-streaming"), "{err}");
}
