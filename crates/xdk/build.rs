//! Build script. Codegen the auth-method matrix from
//! `vendor/x-api-openapi.json` into `$OUT_DIR/auth_matrix.rs`, and emit the
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

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    emit_auth_matrix(&manifest_dir);
    emit_build_info(&manifest_dir);
}

// ── Auth matrix codegen ─────────────────────────────────────────────────
//
// Walks `vendor/x-api-openapi.json`, filters to the shortcut-layer allowlist
// (the (method, path) pairs reachable from `src/api/shortcuts.rs` and
// `src/api/media.rs` after normalizing local param names to the spec), and
// emits a `phf::Map<&'static str, &'static [AuthScheme]>` keyed on
// `"METHOD\0/path/template"`. Wrapped at runtime by `src/api/auth_matrix.rs`.
//
// Missing or empty `security:` arrays are treated as "no entry" — the
// runtime treats those as permissive. Unknown spec security-scheme keys
// panic the build so silent matrix corruption is impossible.

/// (METHOD, spec path) tuples the shortcut + media layer actually calls.
/// Param names match the spec verbatim — code that uses different local
/// names normalizes at the shortcut site.
const SHORTCUT_TEMPLATES: &[(&str, &str)] = &[
    // tweets
    ("POST", "/2/tweets"),
    ("GET", "/2/tweets/{id}"),
    ("DELETE", "/2/tweets/{id}"),
    ("GET", "/2/tweets/search/recent"),
    // users (reads)
    ("GET", "/2/users/me"),
    ("GET", "/2/users/by/username/{username}"),
    ("GET", "/2/users/{id}/timelines/reverse_chronological"),
    ("GET", "/2/users/{id}/mentions"),
    ("GET", "/2/users/{id}/followers"),
    ("GET", "/2/users/{id}/liked_tweets"),
    // likes
    ("POST", "/2/users/{id}/likes"),
    ("DELETE", "/2/users/{id}/likes/{tweet_id}"),
    // retweets
    ("POST", "/2/users/{id}/retweets"),
    ("DELETE", "/2/users/{id}/retweets/{source_tweet_id}"),
    // bookmarks
    ("GET", "/2/users/{id}/bookmarks"),
    ("POST", "/2/users/{id}/bookmarks"),
    ("DELETE", "/2/users/{id}/bookmarks/{tweet_id}"),
    // following
    ("GET", "/2/users/{id}/following"),
    ("POST", "/2/users/{id}/following"),
    (
        "DELETE",
        "/2/users/{source_user_id}/following/{target_user_id}",
    ),
    // muting
    ("GET", "/2/users/{id}/muting"),
    ("POST", "/2/users/{id}/muting"),
    (
        "DELETE",
        "/2/users/{source_user_id}/muting/{target_user_id}",
    ),
    // blocking
    ("GET", "/2/users/{id}/blocking"),
    ("POST", "/2/users/{id}/blocking"),
    (
        "DELETE",
        "/2/users/{source_user_id}/blocking/{target_user_id}",
    ),
    // DMs
    ("POST", "/2/dm_conversations/with/{participant_id}/messages"),
    ("GET", "/2/dm_events"),
    // usage
    ("GET", "/2/usage/tweets"),
    ("GET", "/2/usage/credits"),
    // media
    ("POST", "/2/media/upload"),
    ("GET", "/2/media/upload"),
    ("POST", "/2/media/upload/initialize"),
    ("POST", "/2/media/upload/{id}/append"),
    ("POST", "/2/media/upload/{id}/finalize"),
];

#[derive(Deserialize)]
struct Spec {
    #[serde(default)]
    paths: BTreeMap<String, PathItem>,
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

/// Emit `$OUT_DIR/auth_matrix.rs` from `vendor/x-api-openapi.json`.
///
/// The output is a `phf::Map<&'static str, &'static [AuthScheme]>` plus one
/// `static SUP_<N>` slice per entry. Keys are packed as `"METHOD\0/path"`.
/// Iteration order over `SHORTCUT_TEMPLATES` plus the BTreeMap-backed spec
/// keeps the emitted source byte-deterministic.
fn emit_auth_matrix(manifest_dir: &Path) {
    let spec_path = manifest_dir.join("vendor").join("x-api-openapi.json");
    println!("cargo::rerun-if-changed=vendor/x-api-openapi.json");

    let content = fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", spec_path.display()));
    let spec: Spec = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("parse {}: {e}", spec_path.display()));

    let mut entries: Vec<Entry> = Vec::with_capacity(SHORTCUT_TEMPLATES.len());
    for (method, path) in SHORTCUT_TEMPLATES {
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

    // Emit SHORTCUT_TEMPLATES alongside AUTH_MATRIX so the runtime mirror
    // is regenerated from this same source list every build. Removes the
    // manual hand-copy at src/api/auth_matrix.rs and the test that existed
    // solely to catch drift between the two.
    src.push('\n');
    src.push_str("pub const SHORTCUT_TEMPLATES: &[(&str, &str)] = &[\n");
    for entry in SHORTCUT_TEMPLATES {
        writeln!(&mut src, "    ({:?}, {:?}),", entry.0, entry.1)
            .expect("write to String never fails");
    }
    src.push_str("];\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_path = out_dir.join("auth_matrix.rs");
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
fn emit_build_info(manifest_dir: &Path) {
    // Invalidate the build when HEAD moves so `XDK_CRATE_GIT_SHA` tracks
    // the actual current commit on rebuild.
    println!("cargo:rerun-if-changed=.git/HEAD");
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
    let spec_content = fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", spec_path.display()));
    let spec: serde_json::Value = serde_json::from_str(&spec_content)
        .unwrap_or_else(|e| panic!("parse {}: {e}", spec_path.display()));
    let spec_info_version = spec
        .get("info")
        .and_then(|v| v.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
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
