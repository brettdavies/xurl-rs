# xurl-rs

Rust port of xurl — CLI for authenticated X/Twitter API requests.

## Porting Skill
The rust-porting skill is available at .claude/skills/rust-port/
Follow its 7-phase workflow.

## Source
Original xurl (Go): ~/github-stars/xdevplatform/xurl/

## go2rust Scaffold
Run: ~/dev/go2rust/target/debug/go2rust ~/github-stars/xdevplatform/xurl -o ./scaffold --report
The scaffold is a starting point — it needs idiomification and fixing.

## Research
- Source analysis: ~/obsidian-vault/OpenClaw/research/rust-porting/xurl-case-study/
- Go patterns: ~/dev/rust-porting-skill/references/go-patterns.md
- Crate selection: ~/dev/rust-porting-skill/references/crate-selection.md

## Quality Bar
- Absolute Parity with xurl (identical outputs for identical inputs)
- Clippy clean, edition 2024
- No unwrap() in production code
- Comprehensive tests (unit + integration + differential conformance)
