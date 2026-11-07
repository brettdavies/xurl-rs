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
//! Every `.rs` under each workspace member's `tests/` is scanned,
//! subdirectories included, so the helpers in `tests/common/` and
//! `tests/support/` are held to it too.

mod common;

use common::{member_dirs, rust_files, workspace_name};
use regex::Regex;

/// A bind on port 0: a `"host:0"` literal, or a `(host, 0)` tuple.
fn port_zero_bind() -> Regex {
    Regex::new(r#"bind\(\s*(?:"[^"]*:0"|\(\s*[^,()]+\s*,\s*0(?:u16)?\s*\))\s*\)"#)
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

#[test]
fn no_test_releases_a_port_it_bound_before_using_it() {
    let mut violations = Vec::new();
    for member in member_dirs() {
        for path in rust_files(&member.join("tests")) {
            if path.file_name().is_some_and(|n| n == "port_race_guard.rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            violations.extend(violations_in(&workspace_name(&path), &source));
        }
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
