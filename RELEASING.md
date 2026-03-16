# Releasing xurl

## Release Process

### 1. Bump Version

Update `Cargo.toml`:
```toml
[package]
version = "0.2.0"
```

### 2. Run Tests

```bash
cargo test
cargo clippy -- -W clippy::all -W clippy::pedantic
cargo build --release
```

### 3. Publish to crates.io

```bash
cargo publish --dry-run    # Verify first
cargo publish              # Publish
```

### 4. Tag the Release

```bash
git tag v0.2.0
git push origin main --tags
```

### 5. GitHub Release

Create a release on GitHub with:
- Tag: v0.2.0
- Title: xurl v0.2.0
- Release notes from CHANGELOG

### 6. Update Homebrew Formula

Update `Formula/xurl.rb`:
1. Change the `url` to the new tag
2. Update the `sha256` hash:
   ```bash
   curl -sL https://github.com/brettdavies/xurl-rs/archive/refs/tags/v0.2.0.tar.gz | shasum -a 256
   ```

### 7. Regenerate Completions

```bash
cargo build --release
./target/release/xurl --generate-completion bash > completions/xurl.bash
./target/release/xurl --generate-completion zsh > completions/_xurl
./target/release/xurl --generate-completion fish > completions/xurl.fish
```

## CI (Future)

GitHub Actions will automate:
- `cargo test` on every PR
- `cargo publish` on tag push
- Binary builds for Linux, macOS, Windows
- Homebrew formula auto-update
