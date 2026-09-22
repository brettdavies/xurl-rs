//! Build script. Codegen the auth-method matrix from
//! `vendor/x-api-openapi.json` into `$OUT_DIR/auth_matrix.rs` and the legacy
//! post-vocabulary table into `$OUT_DIR/vocabulary.rs`, and emit the
//! spec-provenance env vars `src/lib.rs` exposes as consts.
//!
//! The vendored spec and its metadata sidecar are the single source of truth —
//! updating either regenerates the consuming Rust modules on the next
//! `cargo build` (via `cargo::rerun-if-changed`). Mirrors the pattern in
//! `agentnative-cli/build.rs`.

use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[path = "codegen/vocabulary.rs"]
mod vocabulary;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let spec_path = manifest_dir.join("vendor").join("x-api-openapi.json");
    println!("cargo::rerun-if-changed=vendor/x-api-openapi.json");
    let content = fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", spec_path.display()));
    let spec: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("parse {}: {e}", spec_path.display()));

    let spec_version = emit_auth_matrix(&spec, &spec_path);
    emit_vocabulary(&spec, &spec_path);
    emit_build_info(&manifest_dir, spec_version.as_deref());
}

// ── Auth matrix codegen ─────────────────────────────────────────────────
//
// Walks `vendor/x-api-openapi.json`, filters to `SHORTCUT_TEMPLATES` (every
// endpoint `src/api/shortcuts.rs` and `src/api/media.rs` call, which they
// name through the generated `endpoints` module), and emits a
// `phf::Map<&'static str, &'static [AuthScheme]>` keyed on
// `"METHOD\0/path/template"`. Wrapped at runtime by `src/api/auth_matrix.rs`.
//
// Missing or empty `security:` arrays are treated as "no entry" — the
// runtime treats those as permissive. Unknown spec security-scheme keys
// panic the build so silent matrix corruption is impossible.

/// Every endpoint the shortcut and media layer calls: the constant name the
/// generated `endpoints` module gives it, its method, and its spec path.
/// Param names match the spec verbatim; a call site that uses different
/// local names normalizes them when it binds the parameters.
const SHORTCUT_TEMPLATES: &[(&str, &str, &str)] = &[
    // tweets
    ("CREATE_POST", "POST", "/2/tweets"),
    ("READ_POST", "GET", "/2/tweets/{id}"),
    ("DELETE_POST", "DELETE", "/2/tweets/{id}"),
    ("SEARCH_POSTS", "GET", "/2/tweets/search/recent"),
    // users (reads)
    ("GET_ME", "GET", "/2/users/me"),
    ("LOOKUP_USER", "GET", "/2/users/by/username/{username}"),
    (
        "GET_TIMELINE",
        "GET",
        "/2/users/{id}/timelines/reverse_chronological",
    ),
    ("GET_MENTIONS", "GET", "/2/users/{id}/mentions"),
    ("GET_FOLLOWERS", "GET", "/2/users/{id}/followers"),
    ("GET_LIKED_POSTS", "GET", "/2/users/{id}/liked_tweets"),
    // likes
    ("LIKE_POST", "POST", "/2/users/{id}/likes"),
    ("UNLIKE_POST", "DELETE", "/2/users/{id}/likes/{tweet_id}"),
    // retweets
    ("REPOST", "POST", "/2/users/{id}/retweets"),
    (
        "UNREPOST",
        "DELETE",
        "/2/users/{id}/retweets/{source_tweet_id}",
    ),
    // bookmarks
    ("GET_BOOKMARKS", "GET", "/2/users/{id}/bookmarks"),
    ("BOOKMARK", "POST", "/2/users/{id}/bookmarks"),
    ("UNBOOKMARK", "DELETE", "/2/users/{id}/bookmarks/{tweet_id}"),
    // following
    ("GET_FOLLOWING", "GET", "/2/users/{id}/following"),
    ("FOLLOW_USER", "POST", "/2/users/{id}/following"),
    (
        "UNFOLLOW_USER",
        "DELETE",
        "/2/users/{source_user_id}/following/{target_user_id}",
    ),
    // muting
    ("GET_MUTED", "GET", "/2/users/{id}/muting"),
    ("MUTE_USER", "POST", "/2/users/{id}/muting"),
    (
        "UNMUTE_USER",
        "DELETE",
        "/2/users/{source_user_id}/muting/{target_user_id}",
    ),
    // blocking
    ("GET_BLOCKED", "GET", "/2/users/{id}/blocking"),
    ("BLOCK_USER", "POST", "/2/users/{id}/blocking"),
    (
        "UNBLOCK_USER",
        "DELETE",
        "/2/users/{source_user_id}/blocking/{target_user_id}",
    ),
    // DMs
    (
        "SEND_DM",
        "POST",
        "/2/dm_conversations/with/{participant_id}/messages",
    ),
    ("GET_DM_EVENTS", "GET", "/2/dm_events"),
    // usage
    ("GET_USAGE", "GET", "/2/usage/tweets"),
    ("GET_USAGE_CREDITS", "GET", "/2/usage/credits"),
    // media
    ("MEDIA_UPLOAD", "POST", "/2/media/upload"),
    ("MEDIA_UPLOAD_STATUS", "GET", "/2/media/upload"),
    (
        "MEDIA_UPLOAD_INITIALIZE",
        "POST",
        "/2/media/upload/initialize",
    ),
    ("MEDIA_UPLOAD_APPEND", "POST", "/2/media/upload/{id}/append"),
    (
        "MEDIA_UPLOAD_FINALIZE",
        "POST",
        "/2/media/upload/{id}/finalize",
    ),
    // broadcasts
    (
        "GET_CHAT_MODERATORS",
        "GET",
        "/2/broadcasts/chat/moderators",
    ),
    (
        "ADD_CHAT_MODERATOR",
        "POST",
        "/2/broadcasts/chat/moderators",
    ),
    (
        "REMOVE_CHAT_MODERATOR",
        "DELETE",
        "/2/broadcasts/chat/moderators/{user_id}",
    ),
];

#[derive(Deserialize)]
struct Spec {
    #[serde(default)]
    info: Info,
    #[serde(default)]
    paths: BTreeMap<String, PathItem>,
}

#[derive(Deserialize, Default)]
struct Info {
    #[serde(default)]
    version: Option<String>,
}

#[derive(Deserialize, Default)]
struct PathItem {
    #[serde(default)]
    get: Option<Operation>,
    #[serde(default)]
    post: Option<Operation>,
    #[serde(default)]
    put: Option<Operation>,
    #[serde(default)]
    delete: Option<Operation>,
    #[serde(default)]
    patch: Option<Operation>,
}

#[derive(Deserialize)]
struct Operation {
    #[serde(default)]
    security: Option<Vec<BTreeMap<String, Vec<String>>>>,
}

/// One concrete matrix entry, ready to emit.
struct Entry {
    method: &'static str,
    path: &'static str,
    schemes: Vec<SchemeRepr>,
}

/// In-memory representation of an `AuthScheme` value pre-codegen.
enum SchemeRepr {
    Bearer,
    OAuth1User,
    OAuth2User(Vec<String>),
}

/// Emit `$OUT_DIR/auth_matrix.rs` from the parsed `vendor/x-api-openapi.json`
/// and return the spec's own `info.version`, so the sidecar cross-check in
/// [`emit_build_info`] reuses this parse of the 900 KB document.
///
/// The output is a `phf::Map<&'static str, &'static [AuthScheme]>` plus one
/// `static SUP_<N>` slice per entry. Keys are packed as `"METHOD\0/path"`.
/// Iteration order over `SHORTCUT_TEMPLATES` plus the BTreeMap-backed spec
/// keeps the emitted source byte-deterministic.
fn emit_auth_matrix(spec: &serde_json::Value, spec_path: &Path) -> Option<String> {
    let spec =
        Spec::deserialize(spec).unwrap_or_else(|e| panic!("parse {}: {e}", spec_path.display()));

    let mut entries: Vec<Entry> = Vec::with_capacity(SHORTCUT_TEMPLATES.len());
    for (_, method, path) in SHORTCUT_TEMPLATES {
        let item = spec.paths.get(*path).unwrap_or_else(|| {
            panic!(
                "{}: SHORTCUT_TEMPLATES references {path:?} but spec has no such path; \
                 fix the allowlist or refresh the spec",
                spec_path.display()
            )
        });
        let op = match *method {
            "GET" => item.get.as_ref(),
            "POST" => item.post.as_ref(),
            "PUT" => item.put.as_ref(),
            "DELETE" => item.delete.as_ref(),
            "PATCH" => item.patch.as_ref(),
            other => panic!("SHORTCUT_TEMPLATES has unsupported method {other:?} for {path:?}"),
        };
        let Some(op) = op else {
            panic!(
                "{}: spec path {path:?} declares no `{method}` operation; \
                 the shortcut layer claims to call it — fix the allowlist or refresh the spec",
                spec_path.display()
            );
        };
        let Some(security) = op.security.as_ref() else {
            continue;
        };
        if security.is_empty() {
            continue;
        }
        let mut schemes = Vec::with_capacity(security.len());
        for entry in security {
            if entry.len() != 1 {
                panic!(
                    "{}: {method} {path}: each security entry must name exactly one scheme \
                     (got {} keys: {:?})",
                    spec_path.display(),
                    entry.len(),
                    entry.keys().collect::<Vec<_>>(),
                );
            }
            let (key, scopes) = entry.iter().next().expect("len == 1 above");
            let scheme = match key.as_str() {
                "BearerToken" => SchemeRepr::Bearer,
                "OAuth2UserToken" => SchemeRepr::OAuth2User(scopes.clone()),
                "UserToken" => SchemeRepr::OAuth1User,
                other => panic!(
                    "{}: {method} {path}: unknown security scheme {other:?} — \
                     extend the build.rs translation table or update the spec",
                    spec_path.display()
                ),
            };
            schemes.push(scheme);
        }
        entries.push(Entry {
            method,
            path,
            schemes,
        });
    }

    let mut src = String::new();
    src.push_str(
        "// @generated by build.rs from vendor/x-api-openapi.json. Do not edit by hand.\n",
    );
    src.push_str("// Refresh via scripts/refresh-x-openapi.sh; `cargo build` regenerates.\n\n");

    for (i, entry) in entries.iter().enumerate() {
        writeln!(
            &mut src,
            "static SUP_{i}: &[AuthScheme] = &[{}];",
            render_schemes(&entry.schemes)
        )
        .expect("write to String never fails");
    }
    src.push('\n');

    let mut map = phf_codegen::Map::new();
    let values: Vec<String> = (0..entries.len()).map(|i| format!("&SUP_{i}")).collect();
    for (entry, value) in entries.iter().zip(values.iter()) {
        let key = format!("{}\0{}", entry.method, entry.path);
        map.entry(key, value.as_str());
    }
    writeln!(
        &mut src,
        "pub static AUTH_MATRIX: phf::Map<&'static str, &'static [AuthScheme]> = {};",
        map.build()
    )
    .expect("write to String never fails");

    // The named endpoint constants and the `(method, path)` list come from
    // the same rows as the matrix, so a call site, the matrix, and the
    // testing mock cannot name an endpoint the others do not know.
    src.push('\n');
    src.push_str("pub mod endpoints {\n");
    src.push_str("    use crate::api::auth_matrix::Endpoint;\n\n");
    for (name, method, path) in SHORTCUT_TEMPLATES {
        writeln!(
            &mut src,
            "    /// `{method} {path}`\n    pub const {name}: Endpoint = Endpoint {{ method: {method:?}, path: {path:?} }};"
        )
        .expect("write to String never fails");
    }
    src.push_str("}\n\n");
    src.push_str("pub const SHORTCUT_TEMPLATES: &[(&str, &str)] = &[\n");
    for (_, method, path) in SHORTCUT_TEMPLATES {
        writeln!(&mut src, "    ({method:?}, {path:?}),").expect("write to String never fails");
    }
    src.push_str("];\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_path = out_dir.join("auth_matrix.rs");
    fs::write(&out_path, src)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", out_path.display()));

    spec.info.version
}

// ── Legacy post-vocabulary table ────────────────────────────────────────
//
// `codegen/vocabulary.rs` derives the pairs; this emits them as two
// `(legacy, current)` tables for the response decoder. A spec object that
// declares both spellings of a pair fails the build, naming the object.

/// Emit `$OUT_DIR/vocabulary.rs`: `ADMITTED`, the pairs the decoder renames,
/// and `EXCLUDED`, the pairs whose legacy spelling is a current name
/// elsewhere in the spec.
fn emit_vocabulary(spec: &serde_json::Value, spec_path: &Path) {
    let table = vocabulary::derive(spec).unwrap_or_else(|e| panic!("{}: {e}", spec_path.display()));

    let mut src = String::new();
    src.push_str(
        "// @generated by build.rs from vendor/x-api-openapi.json. Do not edit by hand.\n",
    );
    src.push_str("// Refresh via scripts/refresh-x-openapi.sh; `cargo build` regenerates.\n");
    for (name, pairs) in [("ADMITTED", &table.admitted), ("EXCLUDED", &table.excluded)] {
        writeln!(&mut src, "\npub const {name}: &[(&str, &str)] = &[")
            .expect("write to String never fails");
        for pair in pairs {
            writeln!(&mut src, "    ({:?}, {:?}),", pair.legacy, pair.current)
                .expect("write to String never fails");
        }
        src.push_str("];\n");
    }

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("vocabulary.rs");
    fs::write(&out_path, src)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", out_path.display()));
}

// ── Build-info emission ─────────────────────────────────────────────────
//
// Populates compile-time env vars consumed by `src/lib.rs`'s `BUILD_INFO`
// const.
//
// API spec metadata (`info_version`, `content_sha256`, `refreshed_at`) is
// vendored alongside the spec itself at `vendor/spec-metadata.json` and
// read from disk here. The sidecar exists so the metadata always describes
// the actual bytes that ship: an uncommitted local refresh sees the new
// metadata immediately, and a crates.io tarball build (no `.git`) carries
// the metadata in the published package. `scripts/refresh-x-openapi.sh`
// writes the sidecar atomically alongside `vendor/x-api-openapi.json`.
//
// Crate git SHA is genuinely build-context provenance and stays
// best-effort: `git rev-parse HEAD` runs when a `.git` directory exists
// and resolves to `None` on crates.io tarball installs.

/// Emit `XDK_CRATE_GIT_SHA`, `XDK_API_SPEC_VERSION`, `XDK_API_SPEC_SHA256`,
/// and `XDK_API_SPEC_DATE` env vars for `src/lib.rs` to consume via
/// `env!` / `option_env!`. Panics if the vendored spec and sidecar
/// disagree on `info.version` (a stale sidecar — re-run
/// `scripts/refresh-x-openapi.sh`).
fn emit_build_info(manifest_dir: &Path, spec_info_version: Option<&str>) {
    // Invalidate the build when HEAD moves so `XDK_CRATE_GIT_SHA` tracks the
    // actual current commit on rebuild. Git resolves the path because HEAD is
    // neither beside this manifest nor always under `.git/` — a linked
    // worktree keeps it in `.git/worktrees/<name>/`. A path cargo cannot stat
    // makes it rerun this script on every build, so a resolution that fails
    // (a crates.io tarball, no git) declares nothing instead.
    if let Some(head) = run_git(manifest_dir, &["rev-parse", "--git-path", "HEAD"])
        .filter(|head| Path::new(head).exists())
    {
        println!("cargo:rerun-if-changed={head}");
    }
    println!("cargo:rerun-if-changed=vendor/spec-metadata.json");

    if let Some(sha) = run_git(manifest_dir, &["rev-parse", "HEAD"]) {
        println!("cargo:rustc-env=XDK_CRATE_GIT_SHA={sha}");
    }

    // Read the vendored metadata sidecar. The sidecar is the source of
    // truth for shipped spec identity.
    let metadata_path = manifest_dir.join("vendor").join("spec-metadata.json");
    let metadata_content = fs::read_to_string(&metadata_path).unwrap_or_else(|e| {
        panic!(
            "read {}: {e} — run scripts/refresh-x-openapi.sh to regenerate",
            metadata_path.display()
        )
    });
    let metadata: serde_json::Value = serde_json::from_str(&metadata_content).unwrap_or_else(|e| {
        panic!("parse {}: {e}", metadata_path.display());
    });
    let metadata_version = string_field(&metadata, "info_version", &metadata_path);
    let metadata_sha256 = string_field(&metadata, "content_sha256", &metadata_path);
    let metadata_refreshed = string_field(&metadata, "refreshed_at", &metadata_path);

    // Cross-check: the sidecar's `info_version` must match the vendored
    // JSON's `info.version`. A mismatch means one was updated without the
    // other; fail the build with a pointer to the refresh script rather
    // than letting drifted metadata reach consumers.
    let spec_path = manifest_dir.join("vendor").join("x-api-openapi.json");
    let spec_info_version = spec_info_version.unwrap_or_else(|| {
        panic!(
            "{}: must have an `info.version` string field",
            spec_path.display()
        )
    });
    if spec_info_version != metadata_version {
        panic!(
            "vendor/x-api-openapi.json `info.version` is {spec_info_version:?} but \
             vendor/spec-metadata.json `info_version` is {metadata_version:?} — \
             run scripts/refresh-x-openapi.sh to resync"
        );
    }

    println!("cargo:rustc-env=XDK_API_SPEC_VERSION={metadata_version}");
    println!("cargo:rustc-env=XDK_API_SPEC_SHA256={metadata_sha256}");
    println!("cargo:rustc-env=XDK_API_SPEC_DATE={metadata_refreshed}");
}

/// Extract a required string field from a `serde_json::Value` object;
/// panic with a helpful message that names the file and field when
/// missing.
fn string_field<'a>(value: &'a serde_json::Value, field: &str, path: &Path) -> &'a str {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            panic!(
                "{}: missing required string field {field:?} — \
                 run scripts/refresh-x-openapi.sh to regenerate",
                path.display()
            )
        })
}

/// Run `git <args>` from `cwd` and return trimmed stdout on success.
/// Returns `None` when git is unavailable, the command fails, or stdout
/// is empty. Used for best-effort provenance metadata that must degrade
/// gracefully on crates.io tarball builds (no `.git` directory).
fn run_git(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Render a slice of `SchemeRepr` as the body of a `&[AuthScheme]` literal.
fn render_schemes(schemes: &[SchemeRepr]) -> String {
    let mut out = String::new();
    for (i, s) in schemes.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        match s {
            SchemeRepr::Bearer => out.push_str("AuthScheme::Bearer"),
            SchemeRepr::OAuth1User => out.push_str("AuthScheme::OAuth1User"),
            SchemeRepr::OAuth2User(scopes) => {
                out.push_str("AuthScheme::OAuth2User(&[");
                for (j, scope) in scopes.iter().enumerate() {
                    if j > 0 {
                        out.push_str(", ");
                    }
                    write!(&mut out, "{scope:?}").expect("write to String never fails");
                }
                out.push_str("])");
            }
        }
    }
    out
}
