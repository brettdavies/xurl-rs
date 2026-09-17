//! The repository README repeats the library README's landing program so the
//! first screen of the repository is the first screen of the crate; only the
//! library copy is a doctest, so this keeps the two identical.

use std::path::Path;

fn first_rust_fence(markdown: &str) -> String {
    let start = markdown.find("```rust").expect("a rust fence exists");
    let body_start = markdown[start..].find('\n').expect("fence line ends") + start + 1;
    let end = markdown[body_start..]
        .find("```")
        .expect("the fence closes")
        + body_start;
    markdown[body_start..end].to_string()
}

#[test]
fn the_repository_readme_carries_the_library_landing_program_verbatim() {
    let root = Path::new(env!("CARGO_WORKSPACE_DIR"));
    let repository = std::fs::read_to_string(root.join("README.md")).expect("root README");
    let library =
        std::fs::read_to_string(root.join("crates/xdk/README.md")).expect("library README");
    assert_eq!(
        first_rust_fence(&repository),
        first_rust_fence(&library),
        "the root README's landing program drifted from the doctested copy in crates/xdk/README.md"
    );
}
