//! Each README a consumer lands on says what is stable before they depend on
//! it: the repository's, the CLI's, and the library's each carry a Stability
//! section that points at the versioning policy.

use std::path::Path;

/// The body of the `## Stability` section of `markdown`, up to the next
/// heading of the same level.
fn stability_section(markdown: &str) -> Option<&str> {
    let start = markdown.find("\n## Stability\n")? + "\n## Stability\n".len();
    let end = markdown[start..]
        .find("\n## ")
        .map_or(markdown.len(), |i| start + i);
    Some(&markdown[start..end])
}

#[test]
fn every_readme_carries_a_stability_section() {
    let root = Path::new(env!("CARGO_WORKSPACE_DIR"));
    for readme in [
        "README.md",
        "crates/xurl-cli/README.md",
        "crates/xdk/README.md",
    ] {
        let markdown = std::fs::read_to_string(root.join(readme))
            .unwrap_or_else(|e| panic!("{readme} must be readable: {e}"));
        let section = stability_section(&markdown)
            .unwrap_or_else(|| panic!("{readme} has no `## Stability` section"));
        assert!(
            section.contains("SemVer") || section.contains("#versioning"),
            "{readme}: the Stability section names neither SemVer nor the versioning policy"
        );
    }
}
