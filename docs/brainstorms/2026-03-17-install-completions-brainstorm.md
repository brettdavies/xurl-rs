# Brainstorm: Full Install Story — Tarballs, Completions, install.sh

## What We're Building

A complete GitHub Releases distribution channel for xurl-rs that gives users a `curl | sh` one-liner install experience with pre-built binaries, bundled shell completions, and SHA256 checksum verification.

### Current state

- GitHub Releases ship **bare binaries** (no tarballs, no completions, no checksums)
- Pre-baked completions exist for bash/zsh/fish only — missing PowerShell and Elvish
- No install script exists
- `cargo-binstall` metadata may reference tarballs that don't exist
- No CI automation to keep completions in sync with CLI changes

### Target state

- Releases ship `.tar.gz` (Linux/macOS) and `.zip` (Windows) archives containing:
  - Binary (`xr` or `xr.exe`)
  - Shell completions for all 5 shells (bash, zsh, fish, PowerShell, Elvish)
  - `LICENSE-MIT`, `LICENSE-APACHE`
  - `README.md`
- SHA256 checksums file published alongside archives
- `install.sh` attached as a release asset
- `cargo-binstall` metadata updated to match new tarball naming
- All 5 completions pre-baked in `completions/` and kept in sync via CI

### User experience

```bash
# One-liner install
curl -sSL https://github.com/brettdavies/xurl-rs/releases/latest/download/install.sh | sh

# Or with version pinning
curl -sSL https://github.com/brettdavies/xurl-rs/releases/download/v1.0.5/install.sh | sh
```

## Why This Approach

### Three independent distribution channels

```text
Tag push (v1.0.5)
  ├─ crates.io  → source crate (Trusted Publishing)         ✅ working
  ├─ GitHub Release → tarballs + checksums + install.sh      ❌ this brainstorm
  │   └─ cargo-binstall reads these tarballs
  └─ Homebrew dispatch → tap builds its own bottles          🔄 in progress (tap repo)
```

Each channel is independent:

- **crates.io** serves Rust developers (`cargo install xurl-rs`)
- **GitHub Releases** serves everyone via direct download or `curl | sh`
- **Homebrew tap** serves macOS/Linux Homebrew users with pre-built bottles (handled by tap repo)

### cargo-dist evaluation first

Rather than immediately writing custom CI, we'll evaluate `cargo-dist` on a throwaway branch. cargo-dist auto-generates:

- Tarballs with binaries + completions
- SHA256 checksums
- `install.sh` and `install.ps1`
- `cargo-binstall` metadata

**Integration points to evaluate:**

- Can Trusted Publishing (OIDC) for crates.io coexist with cargo-dist's publish step?
- Can the Homebrew tap dispatch job be added alongside cargo-dist's generated workflow?
- Does cargo-dist's tarball naming work with the tap's bottle infrastructure?

**Fallback:** If cargo-dist doesn't fit, add tarball packaging (~4 steps), checksums, and a custom install.sh (~200 lines based on starship/just patterns) to the existing release.yml.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Pre-baked completions | All 5 shells (bash, zsh, fish, PowerShell, Elvish) | Near-zero cost via clap_complete; complete coverage |
| Archive format | .tar.gz (Linux/macOS), .zip (Windows) | Standard convention |
| install.sh hosting | GitHub Release asset per version | Version-pinned, no separate hosting, clean URL |
| CI approach | Evaluate cargo-dist first, fallback to custom | Minimize maintenance if cargo-dist fits |
| Checksums | SHA256 per-artifact + unified checksums file | Industry standard; enables verification in install.sh |
| Homebrew integration | No change — tap handles its own bottles | Channels are independent; dispatch trigger already works |
| Completion sync | CI step to regenerate and fail if stale | Prevents drift between CLI changes and pre-baked completions |

## Open Questions

1. **cargo-dist compatibility:** Does cargo-dist support adding custom jobs (Trusted Publishing, Homebrew dispatch) to its generated workflow without them being overwritten on `cargo dist init` re-runs? — To be answered during the cargo-dist evaluation spike.

## Resolved Questions

- **Where do Homebrew bottles live?** In the homebrew-tap repo as GitHub Release assets. The xurl-rs repo only dispatches the trigger.
- **Does this conflict with the bottle plan?** No. GitHub Releases (tarballs for direct install) and Homebrew bottles are independent distribution channels.
- **cargo-binstall metadata:** No `[package.metadata.binstall]` exists in Cargo.toml. Clean slate — add it fresh when tarballs are set up.
- **install.sh scope:** Print instructions for completions only. Don't auto-install to system directories. The tarball includes the files for manual setup.
- **Windows install:** Yes, include `install.ps1` for PowerShell users (`irm ... | iex` pattern). cargo-dist generates this automatically.
