---
title: "refactor: convert completions from flag to subcommand"
type: refactor
status: completed
date: 2026-03-20
---

# refactor: convert completions from flag to subcommand

Convert the hidden `--generate-completion <SHELL>` flag to a visible `xr completions <SHELL>` subcommand, matching
bird's proven pattern. Also hoist `Version` to Tier 1 (early return before config/auth init) for consistency.

## Acceptance Criteria

- [x] `xr completions bash` generates bash completion script to stdout (same for zsh, fish, powershell, elvish)
- [x] `xr completions` (no arg) exits 2 with clap usage error listing valid shells
- [x] `xr completions notashell` exits 2 with clap error
- [x] `completions` is visible in `xr --help` output
- [x] `--generate-completion` flag is fully removed from `Cli` struct
- [x] `xr version` also works without config/auth init (Tier 1 early return)
- [x] SIGPIPE handling added globally at top of `main()` — `xr completions bash | head` exits cleanly
- [x] `unreachable!()` arms in command dispatch for `Completions` and `Version` with explanatory comments
- [x] Pre-baked `completions/` scripts regenerated (all 5 shells: bash, zsh, fish, powershell, elvish)
- [x] All existing completion tests updated to use `["completions", "bash"]` invocation style
- [x] New tests: missing argument, subcommand names in output, no config dir created, works without auth
- [x] README updated: `--generate-completion <shell>` -> `xr completions <shell>`
- [x] `long_about` in `Cli` struct updated to include `completions` example

## Context

**Reference implementation:** bird (`/home/brett/dev/bird/src/cli.rs`, `/home/brett/dev/bird/src/main.rs:587-590`)

**Documented solution:**
`docs/solutions/architecture-patterns/shell-completions-main-dependency-gating.md` — covers the exact pattern including
SIGPIPE, three-tier gating, `unreachable!()` arms, and test coverage.

**Breaking change:** `--generate-completion` was `hide = true` with near-zero external usage. Remove immediately without
deprecation period. Document in CHANGELOG.

## MVP

### Cargo.toml — add libc (unix-only)

```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

### src/main.rs

```rust
use clap::{CommandFactory, Parser};
use cli::Cli;
use cli::Commands;
// ... existing imports ...

fn main() {
    // Restore default SIGPIPE handling (Rust masks it, causing panics on closed pipes)
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();

    // --- Tier 1: Meta-commands (need only parsed args) ---
    if let Some(ref cmd) = cli.command {
        match cmd {
            Commands::Completions { shell } => {
                let mut cmd = Cli::command();
                clap_complete::generate(*shell, &mut cmd, "xr", &mut std::io::stdout());
                return;
            }
            Commands::Version => {
                println!("xr {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            _ => {}
        }
    }

    // --- Tier 3: Everything else (needs config + auth) ---
    let out = OutputConfig::new(cli.output.clone(), cli.quiet);

    match cli::commands::run(cli, &out) {
        Ok(()) => std::process::exit(EXIT_SUCCESS),
        Err(e) => {
            let code = exit_code_for_error(&e);
            out.print_error(&e, code);
            std::process::exit(code);
        }
    }
}
```

### src/cli/mod.rs — add Completions variant, remove generate_completion field

```rust
// Remove this field from Cli struct:
// pub generate_completion: Option<clap_complete::Shell>,

// Add to Commands enum:
// ── Meta ───────────────────────────────────────────────────────
/// Generate shell completion script
Completions {
    /// Shell to generate completions for
    #[arg(value_enum)]
    shell: clap_complete::Shell,
},
```

### src/cli/commands/mod.rs — unreachable arms

```rust
Commands::Completions { .. } => {
    unreachable!("completions is handled before config init in main()")
}
Commands::Version => {
    unreachable!("version is handled before config init in main()")
}
```

### tests/completion_tests.rs — updated invocation

```rust
#[test]
fn test_completion_bash_generates_output() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xr"))
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_completion_no_argument_exits_two() {
    Command::cargo_bin("xr")
        .unwrap()
        .arg("completions")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn test_completions_bash_contains_subcommand_names() {
    Command::cargo_bin("xr")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("post"))
        .stdout(predicate::str::contains("auth"));
}
```

## Sources

- Bird reference: `/home/brett/dev/bird/src/cli.rs`, `/home/brett/dev/bird/src/main.rs:587-590`
- Documented solution: `docs/solutions/architecture-patterns/shell-completions-main-dependency-gating.md`
- Related brainstorm: `docs/brainstorms/2026-03-17-install-completions-brainstorm.md` (tarball/install story —
  completions distribution)
