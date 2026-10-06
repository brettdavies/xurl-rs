//! A comment explains the code to a reader who has the repository and nothing
//! else. A plan, an ID a plan gave one of its units or decisions, and a local
//! todo file are not in the repository, so a comment that cites one points at
//! something that reader cannot open. Comments under `crates/` and `scripts/`
//! state the reason itself.

mod common;

use std::path::{Path, PathBuf};

/// What a comment may not cite, each with a name for the report.
const CITATIONS: &[(&str, &str)] = &[
    (r"\b(?:U|R|KTD|AE)\d+\b", "a plan's ID"),
    (
        r"docs/plans/|-plan\.md\b|\b\w+(?:-\w+)+ plan\b",
        "a plan by name",
    ),
    (r"\bTODO\.md\b", "a local todo file"),
];

/// Every file under `dir`, recursively, sorted.
fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{} must be readable: {e}", dir.display()))
        {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// The files whose comments are checked, each with its line-comment marker.
fn commented_sources() -> Vec<(PathBuf, &'static str)> {
    let rust = common::member_dirs()
        .into_iter()
        .flat_map(|member| common::rust_files(&member))
        .map(|file| (file, "//"));
    let scripts = files_under(&common::workspace_root().join("scripts"))
        .into_iter()
        .map(|file| (file, "#"));
    rust.chain(scripts).collect()
}

#[test]
fn no_comment_cites_a_plan_or_a_local_file() {
    let citations: Vec<(regex::Regex, &str)> = CITATIONS
        .iter()
        .map(|(pattern, name)| (regex::Regex::new(pattern).expect("valid pattern"), *name))
        .collect();
    let root = common::workspace_root();
    let mut hits = Vec::new();
    for (file, marker) in commented_sources() {
        // A script may hold bytes that are not UTF-8; it has no comment to read.
        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        let shown = file.strip_prefix(root).unwrap_or(&file).display();
        for (i, line) in source.lines().enumerate() {
            let line = line.trim();
            if !line.starts_with(marker) {
                continue;
            }
            for (pattern, name) in &citations {
                if pattern.is_match(line) {
                    hits.push(format!("{shown}:{}: {name}: {line}", i + 1));
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "{} comment(s) cite something a reader of the repository cannot open; state the reason \
         in the comment, or remove it where the code already says it:\n{}",
        hits.len(),
        hits.join("\n")
    );
}
