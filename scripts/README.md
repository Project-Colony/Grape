# Grape Development Scripts

This folder contains scripts useful for Grape development.

## Pre-Commit Git Hooks

### Full Version (Recommended)

```bash
./scripts/setup-hooks.sh
```

Before every commit this version runs:
- Format check (rustfmt)
- Static analysis (clippy)
- Tests (cargo test)

**Runtime**: ~30-60 seconds depending on the machine.

### Light Version (Fast)

```bash
./scripts/setup-hooks-light.sh
```

This version runs only:
- Format check (rustfmt)
- Static analysis (clippy)
- No tests

**Runtime**: ~5-10 seconds.

**Important**: don't forget to run the tests manually!

### Temporarily Disable the Hooks

When you need to commit quickly without going through the checks:

```bash
git commit --no-verify -m "Your message"
```

Use sparingly — the checks are there for a reason.

## Useful Commands

### Code Formatting

```bash
# Format everything
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check
```

### Static Analysis

```bash
# Run clippy
cargo clippy --all-targets --all-features

# Strict mode (as in CI)
cargo clippy --all-targets --all-features -- -D warnings
```

### Tests

```bash
# Run all tests
cargo test --all-features

# Verbose output
cargo test --all-features -- --nocapture

# Tests for a specific module
cargo test --test player_tests
cargo test --test cache_tests
cargo test --test metadata_online_tests

# Coverage
cargo install cargo-tarpaulin
cargo tarpaulin --all-features
```

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Ultra-optimized build for minimal size
cargo build --profile release-small
```

## Performance Notes

**All of these tools are for development only.**

They add **zero** CPU/RAM overhead at application runtime:
- Hooks run only during development (git commit)
- rustfmt/clippy don't modify the final binary
- Build profiles are optimized for performance

The shipped binary stays as light and fast as ever.
