//! Fails when a test picks a port by binding port 0 and letting the socket go
//! before the port is used.
//!
//! Between the drop and the use the port is free, and the OS can hand it to
//! any socket that binds port 0 meanwhile: a mock server in a test running
//! beside this one then answers a request meant for an address nothing
//! listens on, or the code under test cannot bind the port it was handed.
//! Such a test passes alone and fails under parallel load.
//!
//! Two shapes drop the socket. A port-0 bind whose address is read in the
//! calls chained onto it leaves the listener a temporary. A listener bound to
//! a name and used only for its address goes when its scope ends: in a helper
//! that returns a value, or in a block expression, that is before the caller
//! uses the address. A test function's own listener lives until the test
//! ends, so it holds its port, as does one moved, returned, or stored.
//! A third drops the held listener of a helper that returns one with its port:
//! `let (_, port) = held_port();` binds nothing to the listener, so it goes at
//! the end of that statement, and the port is free again.
//!
//! Every `.rs` under each workspace member's `tests/` and `examples/` is
//! scanned, subdirectories included, so the helpers in `tests/common/` and
//! `tests/support/` are held to it too. So are the unit tests under `src/`:
//! an inline `#[cfg(test)] mod name { ... }` from its marker to the end of the
//! file, and a file a `#[cfg(test)] mod name;` declares, whole.

mod common;

use std::path::{Path, PathBuf};

use common::production_source::up_to_inline_tests;
use common::{member_dirs, rust_files, workspace_name, workspace_sources};
use regex::Regex;

/// A bind on port 0: a `"host:0"` literal, or a `(host, 0)` tuple.
fn port_zero_bind() -> Regex {
    Regex::new(r#"bind\(\s*(?:"[^"]*:0"|\(\s*[^,()]+\s*,\s*0(?:u16)?\s*\))\s*\)"#)
        .expect("the pattern compiles")
}

/// A `let` whose tuple pattern puts `_` where `held_port` returns its
/// listener.
fn discarded_hold() -> Regex {
    Regex::new(r"let\s*\(\s*_\s*,[^)]*\)\s*=\s*(?:\w+::)*held_port\s*\(")
        .expect("the pattern compiles")
}

/// The calls chained onto an expression: `.method(args)` with one level of
/// nested parentheses in the arguments, `.await`, and `?`.
fn method_chain() -> Regex {
    Regex::new(r"^(?:\s*(?:\.\w+\((?:[^()]|\([^()]*\))*\)|\.await|\?))*")
        .expect("the pattern compiles")
}

/// The 1-based line `offset` falls on.
fn line_of(source: &str, offset: usize) -> usize {
    source[..offset].matches('\n').count() + 1
}

/// The offset of the innermost `{` still open at `offset`.
fn enclosing_open(source: &str, offset: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in source[..offset].char_indices().rev() {
        match c {
            '}' => depth += 1,
            '{' if depth == 0 => return Some(i),
            '{' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// The offset just past the `}` that closes the `{` at `open`.
fn block_end(source: &str, open: usize) -> usize {
    let mut depth = 0usize;
    for (i, c) in source[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + i + 1;
                }
            }
            _ => {}
        }
    }
    source.len()
}

/// Where a listener bound in the block opening at `open` stops holding its
/// port before its address is used: the end of a function that returns a
/// value and is not a test, or the end of a block expression. `None` for a
/// test's own body and for any other block.
fn early_end(source: &str, open: usize) -> Option<String> {
    let before = source[..open].trim_end();
    if before.ends_with('=') {
        return Some("a block expression, so the listener is dropped when the block ends".into());
    }
    let header_start = before.rfind(['}', ';']).map_or(0, |i| i + 1);
    let header = &before[header_start..];
    let signature = Regex::new(r"fn\s+(\w+)[^{]*$").expect("the pattern compiles");
    let test = Regex::new(r"#\[(?:tokio::)?test\b").expect("the pattern compiles");
    let found = signature.captures(header)?;
    let returns = found.get(0).expect("the match").as_str().contains("->");
    (returns && !test.is_match(header)).then(|| {
        format!(
            "fn {}, so the listener is dropped when it returns",
            &found[1]
        )
    })
}

/// Each place in `source` that drops a port-0 socket before its address is
/// used.
fn violations_in(file: &str, source: &str) -> Vec<String> {
    let bind = port_zero_bind();
    let chain = method_chain();
    let binding = Regex::new(r"let\s+(?:mut\s+)?(\w+)\s*(?::[^=;]*)?=\s*[^;]*$")
        .expect("the pattern compiles");
    let mut found = Vec::new();

    for m in discarded_hold().find_iter(source) {
        found.push(format!(
            "{file}:{}: binds `_` where `held_port` returns its listener, which is dropped at \
             the end of the statement, so the port is free again",
            line_of(source, m.start())
        ));
    }

    for m in bind.find_iter(source) {
        let line = line_of(source, m.start());
        let calls = chain.find(&source[m.end()..]).map_or("", |c| c.as_str());
        if calls.contains(".local_addr(") {
            found.push(format!(
                "{file}:{line}: binds port 0 and reads its address from the temporary \
                 listener, which is dropped before the address is used"
            ));
            continue;
        }

        let statement_start = source[..m.start()]
            .rfind([';', '{', '}'])
            .map_or(0, |i| i + 1);
        let Some(named) = binding.captures(&source[statement_start..m.start()]) else {
            continue;
        };
        let name = &named[1];
        let Some(open) = enclosing_open(source, statement_start) else {
            continue;
        };
        let Some(why) = early_end(source, open) else {
            continue;
        };
        let after = m.end() + calls.len();
        let scope = &source[after..block_end(source, open)];
        let uses = Regex::new(&format!(r"\b{name}\b")).expect("the pattern compiles");
        let mut uses = uses.find_iter(scope).peekable();
        if uses.peek().is_some()
            && uses.all(|u| scope[u.end()..].trim_start().starts_with(".local_addr("))
        {
            found.push(format!(
                "{file}:{line}: binds port 0 into `{name}` and uses it only for its address \
                 in {why}"
            ));
        }
    }
    found
}

/// The file `#[cfg(test)] mod name;` in `declaring` names: beside it for a
/// `mod.rs`, `lib.rs`, or `main.rs`, and in a directory named for the file
/// otherwise.
fn declared_module(declaring: &Path, name: &str) -> PathBuf {
    let dir = declaring.parent().expect("a file has a directory");
    let stem = declaring.file_stem().and_then(|s| s.to_str());
    let base = match stem {
        Some("mod" | "lib" | "main") => dir.to_path_buf(),
        _ => dir.join(stem.expect("a file name")),
    };
    [
        base.join(format!("{name}.rs")),
        base.join(name).join("mod.rs"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
    .unwrap_or_else(|| {
        panic!(
            "{}: `#[cfg(test)] mod {name};` names no file under {}",
            declaring.display(),
            base.display()
        )
    })
}

/// The unit tests of one file under `src/`, as each file's path and the text
/// to scan. An inline test module comes with the lines above it blanked, so a
/// line number in the text is the file's own. A declared test module is the
/// file it names, whole.
fn unit_tests_in(path: &Path, source: &str) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let shipped = up_to_inline_tests(source);
    if shipped.len() < source.len() {
        let blanked = "\n".repeat(shipped.matches('\n').count());
        found.push((
            path.to_path_buf(),
            format!("{blanked}{}", &source[shipped.len()..]),
        ));
    }
    let declaration = Regex::new(r"#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;")
        .expect("the pattern compiles");
    for module in declaration.captures_iter(source) {
        let file = declared_module(path, &module[1]);
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", file.display()));
        found.push((file, text));
    }
    found
}

/// Every file or part of a file the guard reads: all of each member's
/// `tests/` and `examples/`, and the unit tests under `src/`.
fn scanned() -> Vec<(PathBuf, String)> {
    let read = |path: &Path| {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
    };
    let mut scanned = Vec::new();
    for member in member_dirs() {
        for dir in ["tests", "examples"] {
            let dir = member.join(dir);
            if !dir.is_dir() {
                continue;
            }
            for path in rust_files(&dir) {
                if path.file_name().is_some_and(|n| n == "port_race_guard.rs") {
                    continue;
                }
                let text = read(&path);
                scanned.push((path, text));
            }
        }
    }
    for path in workspace_sources() {
        scanned.extend(unit_tests_in(&path, &read(&path)));
    }
    scanned
}

#[test]
fn no_test_releases_a_port_it_bound_before_using_it() {
    let mut violations = Vec::new();
    for (path, text) in scanned() {
        violations.extend(violations_in(&workspace_name(&path), &text));
    }

    assert!(
        violations.is_empty(),
        "a test picks a port by binding port 0 and letting the socket go:\n  {}\n\n\
         The port is free between the drop and its use, so a test running beside this one can \
         bind it (xurl-rs #455). Use instead:\n  \
         - for an address nothing answers, `common::CLOSED` (http://127.0.0.1:1), which no \
         test binds;\n  \
         - for a run that must not bind, the listener held for the whole test with its port \
         (`held_port`);\n  \
         - for code under test that binds, port 0 handed to it and the port it bound read from \
         its own output (`listen_while`).\n\
         A test of binding a port it must name first draws one below the ephemeral range \
         (`band_port`).",
        violations.join("\n  ")
    );
}

/// The guard catches each shape that drops the listener, and passes each that
/// keeps it.
#[test]
fn the_guard_tells_a_released_port_from_a_held_one() {
    let released = [
        (
            "a port read from the temporary",
            r#"
fn closed_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("a loopback port")
        .local_addr()
        .expect("its address")
        .port()
}
"#,
        ),
        (
            "an address read whole from the temporary, bound by tuple",
            r#"
#[test]
fn a_test() {
    let addr = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap().local_addr().unwrap();
    let url = format!("http://{addr}");
}
"#,
        ),
        (
            "a typed zero",
            r#"
async fn a_helper() {
    let port = tokio::net::TcpListener::bind(("127.0.0.1", 0u16)).await?.local_addr()?.port();
}
"#,
        ),
        (
            "a named listener returned as its port",
            r#"
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.local_addr().unwrap().port()
}
"#,
        ),
        (
            "a named listener returned as a URL",
            r#"
fn closed_url() -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    format!("http://{}", l.local_addr().unwrap())
}
"#,
        ),
        (
            "a held port whose listener is discarded",
            r#"
#[test]
fn a_test() {
    let (_, port) = held_port();
    let url = format!("http://127.0.0.1:{port}");
}
"#,
        ),
        (
            "a held port reached through its module, listener discarded",
            r#"
#[tokio::test]
async fn a_test() {
    let (_, port) = common::held_port();
}
"#,
        ),
        (
            "a named listener in a block expression",
            r#"
#[tokio::test]
async fn a_test() {
    let port = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
}
"#,
        ),
    ];
    for (shape, source) in released {
        assert_eq!(
            violations_in("released.rs", source).len(),
            1,
            "{shape}: {source}"
        );
    }

    let held = [
        (
            "the listener returned with its port",
            r#"
pub fn held_port() -> (std::net::TcpListener, u16) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port = listener.local_addr().expect("its address").port();
    (listener, port)
}
"#,
        ),
        (
            "a listener stored in a field",
            r#"
impl Scratch {
    fn new() -> Self {
        Self {
            held: std::net::TcpListener::bind("127.0.0.1:0").expect("a port to hold"),
        }
    }
    fn held_port(&self) -> String {
        self.held.local_addr().expect("the held address").port().to_string()
    }
}
"#,
        ),
        (
            "a stub server that moves its listener into a thread",
            r#"
fn serve() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || drop(listener.accept()));
    port
}
"#,
        ),
        (
            "a test's own listener, held until the test ends",
            r#"
#[test]
fn a_bind_on_a_held_port_fails() {
    let hog = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = hog.local_addr().unwrap().port();
    assert!(TcpListener::bind(("127.0.0.1", port)).is_err());
}
"#,
        ),
        (
            "a held port whose listener is named",
            r#"
#[test]
fn a_test() {
    let (_held, port) = held_port();
    let url = format!("http://127.0.0.1:{port}");
}
"#,
        ),
        (
            "a held port whose listener is kept under its own name",
            r#"
#[test]
fn a_test() {
    let (held, port) = held_port();
    drop(held);
}
"#,
        ),
        (
            "a port picked without a bind",
            r"
pub fn band_port() -> u16 {
    20000
}
",
        ),
    ];
    for (shape, source) in held {
        assert_eq!(
            violations_in("held.rs", source),
            Vec::<String>::new(),
            "{shape}"
        );
    }
}

/// The unit tests under `src/` are scanned from the inline test module on, or
/// whole for a module a `#[cfg(test)] mod name;` declares, and what the guard
/// reports is the line in the file it names.
#[test]
fn the_guard_reads_the_unit_tests_under_src() {
    let tmp = tempfile::TempDir::new().expect("a scratch directory");
    let src = tmp.path().join("src");
    std::fs::create_dir_all(src.join("parse")).expect("the module directory");
    let write = |path: &Path, text: &str| std::fs::write(path, text).expect("a source file");

    let inline = "fn production() {\n    let (_, port) = held_port();\n}\n\n\
                  #[cfg(test)]\nmod tests {\n    fn a_test() {\n        let (_, port) = held_port();\n    }\n}\n";
    let found = unit_tests_in(&src.join("inline.rs"), inline);
    assert_eq!(found.len(), 1, "only the inline test module is read");
    let reported = violations_in("inline.rs", &found[0].1);
    assert_eq!(reported.len(), 1, "{reported:?}");
    assert!(reported[0].starts_with("inline.rs:8:"), "{reported:?}");

    write(
        &src.join("parse.rs"),
        "fn production() {}\n\n#[cfg(test)]\nmod cases;\n",
    );
    write(
        &src.join("parse").join("cases.rs"),
        "#[test]\nfn a_test() {\n    let (_, port) = held_port();\n}\n",
    );
    let declaring = src.join("parse.rs");
    let found = unit_tests_in(
        &declaring,
        &std::fs::read_to_string(&declaring).expect("read"),
    );
    assert_eq!(found.len(), 1, "the declared module is the one file read");
    assert_eq!(found[0].0, src.join("parse").join("cases.rs"));
    let reported = violations_in("cases.rs", &found[0].1);
    assert!(reported[0].starts_with("cases.rs:3:"), "{reported:?}");

    write(&src.join("lib.rs"), "#[cfg(test)]\nmod beside;\n");
    write(&src.join("beside.rs"), "fn a_test() {}\n");
    let declaring = src.join("lib.rs");
    let found = unit_tests_in(
        &declaring,
        &std::fs::read_to_string(&declaring).expect("read"),
    );
    assert_eq!(
        found[0].0,
        src.join("beside.rs"),
        "a lib.rs names a sibling file"
    );

    let shipped = "fn production() {\n    let (_, port) = held_port();\n}\n";
    assert!(
        unit_tests_in(&src.join("shipped.rs"), shipped).is_empty(),
        "code above no test module is not read"
    );
}
