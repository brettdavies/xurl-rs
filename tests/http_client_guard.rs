//! One HTTP client, built once, with no timeout-free fallback.
//!
//! Every request, token exchange, refresh, and `/2/users/me` lookup goes
//! through the client `ApiClient` owns. A `Client::new()` fallback would put
//! the whole process on a timeout-free client the moment the builder failed,
//! so the library keeps exactly one construction site and it returns an
//! error instead.

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
fn the_http_client_is_built_in_exactly_one_place() {
    let pattern = regex::Regex::new(r"reqwest::Client::(builder|new)\(").unwrap();
    let hits = source_hits(&pattern);
    assert_eq!(
        hits.len(),
        1,
        "every request shares the one client `ApiClient` builds:\n{}",
        hits.join("\n")
    );
    assert!(
        hits[0].contains("src/api/request/mod.rs"),
        "the construction site lives on the client: {}",
        hits[0]
    );
}

#[test]
fn the_library_registers_no_signal_handler_and_owns_no_runtime() {
    let pattern =
        regex::Regex::new(r"tokio::signal|tokio::runtime|block_on\(|reqwest::blocking").unwrap();
    let hits: Vec<String> = source_hits(&pattern)
        .into_iter()
        .filter(|hit| !hit.contains("/src/cli/"))
        .collect();
    assert!(
        hits.is_empty(),
        "signals and runtimes belong to the binary under src/cli:\n{}",
        hits.join("\n")
    );
}
