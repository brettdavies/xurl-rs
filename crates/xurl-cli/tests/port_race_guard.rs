//! Fails when a test picks a port by binding port 0 and letting the socket go
//! before the port is used.
//!
//! Between the drop and the use the port is free, and the OS can hand it to
//! any socket that binds port 0 meanwhile: a mock server in a test running
//! beside this one then answers a request meant for an address nothing
//! listens on, or the code under test cannot bind the port it was handed.
//! Such a test passes alone and fails under parallel load.
//!
//! Two shapes drop the socket: a bind on port 0 whose port is read in the
//! calls chained onto it, which leaves the listener a temporary, and a
//! function returning a bare `u16` whose body binds port 0, whose listener
//! drops on return.
//! Every `.rs` under each workspace member's `tests/` is scanned,
//! subdirectories included, so the helpers in `tests/common/` and
//! `tests/support/` are held to it too.

mod common;

use common::{member_dirs, rust_files, workspace_name};

/// A bind on port 0: a `"host:0"` literal, or a `("host", 0)` tuple.
fn port_zero_bind() -> regex::Regex {
    regex::Regex::new(r#"bind\(\s*(?:"[^"]*:0"|\(\s*"[^"]*"\s*,\s*0\s*\))\s*\)"#)
        .expect("the pattern compiles")
}

/// The calls chained onto an expression: `.method(args)` with one level of
/// nested parentheses in the arguments, `.await`, and `?`. A port read in this
/// chain is read from a listener nothing holds.
fn method_chain() -> regex::Regex {
    regex::Regex::new(r"^(?:\s*(?:\.\w+\((?:[^()]|\([^()]*\))*\)|\.await|\?))*")
        .expect("the pattern compiles")
}

/// The 1-based line `offset` falls on.
fn line_of(source: &str, offset: usize) -> usize {
    source[..offset].matches('\n').count() + 1
}

/// The body of the function whose opening brace is at `open`, by brace depth.
fn body_from(source: &str, open: usize) -> &str {
    let mut depth = 0usize;
    for (i, c) in source[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open..=open + i];
                }
            }
            _ => {}
        }
    }
    &source[open..]
}

/// Each place in `source` that drops a port-0 socket before its port is used.
fn violations_in(file: &str, source: &str) -> Vec<String> {
    let bind = port_zero_bind();
    let mut found = Vec::new();

    let chain = method_chain();
    for m in bind.find_iter(source) {
        let calls = chain.find(&source[m.end()..]).map_or("", |c| c.as_str());
        if calls.contains(".port()") {
            found.push(format!(
                "{file}:{}: binds port 0 and reads its port from the temporary listener, so \
                 the listener is dropped before the port is used",
                line_of(source, m.start())
            ));
        }
    }

    let returns_port =
        regex::Regex::new(r"fn (\w+)[^{;]*->\s*u16\s*\{").expect("the pattern compiles");
    for m in returns_port.captures_iter(source) {
        let whole = m.get(0).expect("the match");
        let body = body_from(source, whole.end() - 1);
        if bind.is_match(body) {
            found.push(format!(
                "{file}:{}: fn {} binds port 0 and returns the bare port, so its listener is \
                 dropped on return",
                line_of(source, whole.start()),
                &m[1]
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
         - for an address nothing answers, http://127.0.0.1:1, which no test server binds;\n  \
         - for a run that must not bind, the listener held for the whole test with its port \
         (`held_port`);\n  \
         - for code under test that binds, port 0 handed to it and the port it bound read from \
         its own output (`listen_while`).\n\
         A test of binding a port it must name first draws one below the ephemeral range \
         (`band_port`).",
        violations.join("\n  ")
    );
}

/// The guard catches both shapes, and passes the safe helpers.
#[test]
fn the_guard_tells_a_released_port_from_a_held_one() {
    let released_inline = r#"
fn closed_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("a loopback port")
        .local_addr()
        .expect("its address")
        .port()
}
"#;
    let released_after_let = r#"
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.local_addr().unwrap().port()
}
"#;
    let held = r#"
pub fn held_port() -> (std::net::TcpListener, u16) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port = listener.local_addr().expect("its address").port();
    (listener, port)
}
pub fn band_port() -> u16 {
    20000
}
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
"#;
    assert_eq!(violations_in("inline.rs", released_inline).len(), 2);
    assert_eq!(violations_in("after_let.rs", released_after_let).len(), 1);
    assert_eq!(
        violations_in("held.rs", held),
        Vec::<String>::new(),
        "a held listener, a held field, and a port picked without a bind pass"
    );
}
