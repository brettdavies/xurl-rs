//! The add-a-command recipe in `AGENTS.md` describes what adding a family
//! actually requires: every file it names exists, every walk that guards a
//! surface is named in it, and no document restates the shortcut function
//! names that `crates/xdk/src/api/shortcuts.rs` already is the list of.

mod common;

use std::collections::BTreeSet;
use std::path::Path;

const RECIPE_HEADING: &str = "### Adding a command family";

/// How many distinct shortcut names a document may mention before it counts
/// as restating the list.
const LIST_THRESHOLD: usize = 3;
const DOCS_WITHOUT_FUNCTION_LISTS: &[&str] =
    &["AGENTS.md", "CONTRIBUTING.md", "README.md", "CONCEPTS.md"];

/// The recipe section of `AGENTS.md`: from its heading to the next heading.
fn recipe() -> String {
    let agents = std::fs::read_to_string(common::workspace_root().join("AGENTS.md"))
        .expect("AGENTS.md is readable");
    let start = agents
        .find(RECIPE_HEADING)
        .unwrap_or_else(|| panic!("AGENTS.md has no `{RECIPE_HEADING}` section"));
    let body = &agents[start + RECIPE_HEADING.len()..];
    let end = body
        .find("\n## ")
        .or_else(|| body.find("\n### "))
        .unwrap_or(body.len());
    body[..end].to_string()
}

/// Every backticked token in `text` that looks like a repository path.
fn named_paths(text: &str) -> BTreeSet<String> {
    regex::Regex::new(r"`([A-Za-z0-9_./-]+/[A-Za-z0-9_./-]+)`")
        .unwrap()
        .captures_iter(text)
        .map(|c| c[1].to_string())
        .collect()
}

#[test]
fn every_file_the_recipe_names_exists() {
    let missing: Vec<String> = named_paths(&recipe())
        .into_iter()
        .filter(|path| !common::workspace_root().join(path).exists())
        .collect();
    assert!(
        missing.is_empty(),
        "the add-a-command recipe in AGENTS.md names files that do not exist: {missing:?}\n\
         Cause: a file was moved or renamed and the recipe still points at the old path.\n\
         Fix: update the path under `{RECIPE_HEADING}` in AGENTS.md."
    );
}

/// The integration test files whose failure messages carry the walk shape
/// (a problem, a `Cause:`, and a `Fix:`), which is how a registry walk
/// announces itself.
fn walk_files() -> BTreeSet<String> {
    let root = common::workspace_root();
    common::member_dirs()
        .iter()
        .flat_map(|member| common::rust_files(&member.join("tests")))
        .filter(|file| std::fs::read_to_string(file).is_ok_and(|source| source.contains("Cause:")))
        .map(|file| {
            file.strip_prefix(root)
                .expect("under the workspace")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

#[test]
fn every_walk_is_named_in_the_recipe() {
    let named = named_paths(&recipe());
    let unnamed: Vec<String> = walk_files()
        .into_iter()
        .filter(|walk| !named.contains(walk))
        .collect();
    assert!(
        !walk_files().is_empty(),
        "no test file carries a `Cause:` message, so the walk set is empty"
    );
    assert!(
        unnamed.is_empty(),
        "these registry walks are not named in the add-a-command recipe: {unnamed:?}\n\
         Cause: a walk was added or moved without the recipe naming the surface it guards.\n\
         Fix: under `{RECIPE_HEADING}` in AGENTS.md, name the file beside the surface it covers."
    );
}

#[test]
fn no_document_lists_the_shortcut_functions_by_hand() {
    let shortcuts =
        std::fs::read_to_string(common::workspace_root().join("crates/xdk/src/api/shortcuts.rs"))
            .expect("shortcuts.rs is readable");
    let names: Vec<String> = regex::Regex::new(r"pub fn ([a-z_]+)\(")
        .unwrap()
        .captures_iter(&shortcuts)
        .map(|c| format!("`{}`", &c[1]))
        .collect();
    let mut listed = Vec::new();
    for doc in DOCS_WITHOUT_FUNCTION_LISTS {
        let path = common::workspace_root().join(doc);
        if !Path::new(&path).exists() {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("doc is readable");
        let named: Vec<&String> = names
            .iter()
            .filter(|name| text.contains(name.as_str()))
            .collect();
        if named.len() >= LIST_THRESHOLD {
            listed.push(format!("{doc}: {named:?}"));
        }
    }
    assert!(
        listed.is_empty(),
        "these documents list shortcut function names by hand: {listed:?}\n\
         Cause: a prose list of shortcuts is a hand-maintained registry that goes stale the day \
         a shortcut is added; crates/xdk/src/api/shortcuts.rs is the list. A document may \
         mention one or two functions in passing; {LIST_THRESHOLD} or more is a list.\n\
         Fix: point at the file instead of naming its functions."
    );
}
