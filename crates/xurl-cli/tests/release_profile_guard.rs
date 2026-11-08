//! The release profile lives in two manifests. Cargo builds every workspace
//! binary, the release archives included, from the workspace root's
//! `[profile.release]` and ignores a member's; `cargo install xurl-rs` builds
//! the published crate as its own root and reads only the crate's. The two
//! tables must be equal, or a source install ships a different binary from the
//! release archives.

use std::path::Path;

fn release_profile(manifest: &Path) -> toml::Value {
    let text = std::fs::read_to_string(manifest)
        .unwrap_or_else(|e| panic!("{} must be readable: {e}", manifest.display()));
    let doc: toml::Table = text
        .parse()
        .unwrap_or_else(|e| panic!("{} must parse as TOML: {e}", manifest.display()));
    doc.get("profile")
        .and_then(|profile| profile.get("release"))
        .cloned()
        .unwrap_or_else(|| panic!("{} has no [profile.release]", manifest.display()))
}

#[test]
fn published_release_profile_equals_the_workspace_root() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = release_profile(&crate_dir.join("../../Cargo.toml"));
    let published = release_profile(&crate_dir.join("Cargo.toml"));
    assert_eq!(
        published, root,
        "crates/xurl-cli/Cargo.toml's [profile.release] differs from the workspace root's. \
         The release archives build from the root's table and `cargo install xurl-rs` from \
         the crate's, so they would ship different binaries. Make the crate's table a copy \
         of the root's."
    );
}
