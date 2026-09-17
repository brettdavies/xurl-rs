//! The request layer names an endpoint through the declaration `build.rs`
//! emits (`auth_matrix::endpoints`), never through a path literal of its
//! own. A path spelled at a call site is a second copy of the declaration,
//! and nothing notices when the two disagree until X answers 404.

use std::path::Path;

/// The files that call endpoints, relative to the crate root.
const REQUEST_LAYER: &[&str] = &["src/api/shortcuts.rs", "src/api/media.rs"];

#[test]
fn the_request_layer_spells_no_api_path() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hits = Vec::new();
    for file in REQUEST_LAYER {
        let source = std::fs::read_to_string(root.join(file))
            .unwrap_or_else(|e| panic!("{file} must be readable: {e}"));
        for (i, line) in source.lines().enumerate() {
            if line.contains("\"/2/") {
                hits.push(format!("{file}:{}: {}", i + 1, line.trim()));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "these lines spell an API path instead of naming its declaration:\n{}\n\
         Cause: a call site carries its own copy of a path that build.rs already declares, so \
         the request and the auth matrix can disagree without a test noticing.\n\
         Fix: declare the endpoint once in SHORTCUT_TEMPLATES in crates/xdk/build.rs, then read \
         `endpoints::<NAME>.path` (and `.method`) at the call site.",
        hits.join("\n")
    );
}
