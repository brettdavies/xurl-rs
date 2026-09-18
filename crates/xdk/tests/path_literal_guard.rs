//! The request layer and the testing mock name an endpoint through the
//! declaration `build.rs` emits (`auth_matrix::endpoints`), never through a
//! path literal of their own. A path spelled at a call site or in a route is
//! a second copy of the declaration, and nothing notices when the two
//! disagree until X answers 404 or the mock does. Comment lines are not
//! scanned, so a doc example may still show a path.

use std::path::Path;

/// The files that name endpoints, relative to the crate root.
const ENDPOINT_READERS: &[&str] = &[
    "src/api/shortcuts.rs",
    "src/api/media.rs",
    "src/testing/mod.rs",
];

#[test]
fn no_endpoint_reader_spells_an_api_path() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hits = Vec::new();
    for file in ENDPOINT_READERS {
        let source = std::fs::read_to_string(root.join(file))
            .unwrap_or_else(|e| panic!("{file} must be readable: {e}"));
        for (i, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if line.contains("\"/2/") {
                hits.push(format!("{file}:{}: {}", i + 1, line.trim()));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "these lines spell an API path instead of naming its declaration:\n{}\n\
         Cause: a call site or a mock route carries its own copy of a path that build.rs \
         already declares, so the two can disagree without a test noticing.\n\
         Fix: declare the endpoint once in SHORTCUT_TEMPLATES in crates/xdk/build.rs, then name \
         `endpoints::<NAME>` there instead of spelling the path.",
        hits.join("\n")
    );
}
