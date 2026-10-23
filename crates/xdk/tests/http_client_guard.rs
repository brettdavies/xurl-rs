//! One HTTP client for X, built once, with no timeout-free fallback.
//!
//! Every request, token exchange, refresh, and `/2/users/me` lookup goes
//! through the client `Client` owns. A `reqwest::Client::new()` fallback would put
//! the whole process on a timeout-free client the moment the builder failed,
//! so the library keeps exactly one construction site for X and it returns an
//! error instead. The webhook forward talks to the caller's own endpoint, not
//! to X, with redirects off, so it builds the one other client.

use std::path::Path;

fn rust_sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("readable source tree") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn source_hits(pattern: &regex::Regex) -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&root, &mut files);
    let mut hits = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).expect("readable source");
        for (i, line) in source.lines().enumerate() {
            if pattern.is_match(line) {
                hits.push(format!("{}:{}: {}", file.display(), i + 1, line.trim()));
            }
        }
    }
    hits
}

#[test]
fn no_client_new_fallback_survives_under_src() {
    let pattern = regex::Regex::new(r"unwrap_or_else\(\|_\| .*Client::new").unwrap();
    let hits = source_hits(&pattern);
    assert!(
        hits.is_empty(),
        "a builder failure must surface as an error, not a timeout-free client:\n{}",
        hits.join("\n")
    );
}

#[test]
fn the_http_client_is_built_in_exactly_one_place_for_x_and_one_for_forwards() {
    let pattern = regex::Regex::new(r"reqwest::Client::(builder|new)\(").unwrap();
    let hits = source_hits(&pattern);
    assert_eq!(
        hits.len(),
        2,
        "every request to X shares the one client `Client` builds, and every forward \
         the one `UrlTarget` builds:\n{}",
        hits.join("\n")
    );
    for site in ["src/api/request/mod.rs", "src/webhooks/forward.rs"] {
        assert!(
            hits.iter().any(|hit| hit.contains(site)),
            "a construction site lives in {site}: {hits:?}"
        );
    }
}

#[test]
fn the_library_registers_no_signal_handler_and_owns_no_runtime() {
    let pattern =
        regex::Regex::new(r"tokio::signal|tokio::runtime|block_on\(|reqwest::blocking").unwrap();
    let hits = source_hits(&pattern);
    assert!(
        hits.is_empty(),
        "signals and runtimes belong to the xr binary, not the library:\n{}",
        hits.join("\n")
    );
}
