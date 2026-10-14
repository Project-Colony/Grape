# Working on Grape

## Building

```bash
cargo check                  # typecheck, the fast loop
cargo test                   # every test, all of them inside src/
cargo run --profile fast     # optimized dependencies, incremental user code
cargo build --release        # what ships
```

Requires Rust 1.90 or newer, and on Linux `libasound2-dev`, `pkg-config`,
`clang` and `lld`. The reasons are in
[../guide/install.md](../guide/install.md#build-from-source), and the linker one
in particular is worth reading before spending an afternoon on a link error.

### Profiles

Five entries below, and they are not all in the same file: four profiles plus
one per-package override.

| Profile | Where | What it is for |
|---|---|---|
| `dev` | `.cargo/config.toml` | `opt-level = 0`, split debuginfo, incremental |
| `dev.package."*"` | `Cargo.toml` | dependencies at `opt-level = 3` even in dev; without it, indexing and audio init are unusably slow in debug |
| `fast` | `Cargo.toml` | inherits `release`, adds incremental and line-table debuginfo. The one to use while iterating on the UI |
| `release` | `.cargo/config.toml` | `opt-level = 3`, thin LTO, one codegen unit, stripped, `panic = "abort"` |
| `release-small` | `.cargo/config.toml` | `opt-level = "z"`, fat LTO: smaller binary, slower build |

`.cargo/config.toml` also sets `-D warnings` and enables `clippy::pedantic` and
`clippy::nursery` for every build in the workspace, with a documented list of
allows. A new warning anywhere fails the build, which is deliberate.

## What CI checks

`.github/workflows/ci.yml` runs on every pull request and every push to `main`:

| Job | What it runs |
|---|---|
| rustfmt | `cargo fmt --check` against the stable options in `.rustfmt.toml` |
| clippy | `cargo clippy --all-targets` on Linux, Windows and macOS, with the lint set above, so any lint fails it |
| test | `cargo build --all-targets` and `cargo test` on Linux, Windows and macOS; on Windows also the `.exe`'s version resource and GUI subsystem |
| minimum supported Rust | `cargo build` with Rust 1.90, the `rust-version` in `Cargo.toml` |
| advisories | `cargo audit --deny warnings` over `Cargo.lock`, also weekly; exceptions in [dependency-exceptions.md](dependency-exceptions.md) |
| colony.json | the manifest against the published schema |

Clippy runs on all three systems because the tray, media-session and autostart
code is behind `cfg(target_os)`: a lint in the Windows tray backend only shows
up on Windows.

## Checks before committing

The git hooks run rustfmt and clippy before a commit leaves your machine, so a
pull request does not have to go red to find out.

```bash
./scripts/setup-hooks.sh         # rustfmt + clippy + cargo test on every commit
./scripts/setup-hooks-light.sh   # rustfmt + clippy only
```

The light hook is a few seconds; the full one is closer to a minute. If you
install the light one, run the tests yourself. `git commit --no-verify` skips
both, and should stay rare.

Formatting is `.rustfmt.toml`; lints are the `.cargo/config.toml` block above,
with the thresholds that differ from clippy's defaults in `.clippy.toml`.

## The test suite

| | |
|---|---|
| `src/**` (inline `mod tests`) | 90 `#[test]` functions, 2 of them `#[ignore]` (one needs an audio device, one a private D-Bus session): settings normalization and clamping, the theme migration, album-artist inference, cache-path validation, EQ clamping, the file-layout migration and its fallback, the native media-session state, how Preferences open and close, whether Grape's category name fits the category list, and that a language change relabels the tray |
| `src/library/cache/tests.rs` | 20 tests over the library cache round trip and signature invalidation |
| `src/library/metadata/online/tests.rs` | 20 tests over Last.fm response parsing, the TTL, and the backoff |
| `src/player/tests.rs` | 25 tests, **20 of them `#[ignore]`** |

Every test lives in a `#[cfg(test)]` module under `src/`, so `cargo test` runs
them all in one process. A test build points the Colony config, data and cache
roots at a throwaway directory for that process, so no test touches the real
profile.

### What the tests do not cover

Say this plainly, because it is the part that surprises people.

- **The audio path is barely tested.** Twenty of the twenty-five player tests
  need a real output device and are marked `#[ignore]`, so a default
  `cargo test` runs five of them. Playback, seeking, gapless and the EQ are
  verified by hand.
- **Windows and macOS are built and tested, not run.** CI compiles and runs the
  suite on both, and a release starts the Windows and Apple Silicon binaries
  with `--version`, but the LaunchAgent, the HKCU autostart, the `tray-icon`
  backend and the global hotkeys have no automated exercise on either OS. Linux
  is the platform actually run.
- **Last.fm is never contacted by a test.** The online tests cover parsing,
  caching, the TTL and the backoff against fixtures. The live API is not in the
  loop, and the code path needs a user-supplied key to do anything at all.

Running the ignored player tests, on a machine with working audio output:

```bash
cargo test --lib player::tests -- --ignored
```

## Conventions

- **Everything in the repository is English:** code, identifiers, comments,
  commit messages, and these documents. French is a shipped *UI locale*, which
  is a different thing: it lives in `src/ui/i18n.rs` as `STRINGS_FR` and stays
  there.
- **Commits are Conventional Commits.** release-please parses them to decide
  the next version and to write `CHANGELOG.md`; a `fix:` is a patch, a `feat:`
  a minor, and a breaking change (`feat!:`) the major, on 0.x too.
  `CHANGELOG.md` is never edited by hand.
- **A change that makes a document wrong includes fixing the document.** A
  stale page is worse than a missing one, because it is trusted.

## Adding a preference

Enough of them exist that the shape is settled:

1. Add the field to `UserSettings` in `src/config/mod.rs`, with a default.
   Serde is `#[serde(default)]`, so an old preferences file stays readable.
2. Clamp or validate it in `normalized()`, which runs on load.
3. Add a `UiMessage` variant and handle it in `src/ui/state.rs`.
4. Render it in the right `src/ui/app/preferences/` panel.
5. Add both strings to `STRINGS_FR` and `STRINGS_EN` in `src/ui/i18n.rs`.
6. **Make something read it.** Several existing settings stop at step 5:
   the audio stability mode, the time format and others, all catalogued in
   [../guide/configuration.md](../guide/configuration.md#settings-that-persist-and-change-nothing).
   A control that appears to work and does not is worse than no control.

## Repository layout

```
assets/     the logo and application icons, with their sources in assets/brand/
docs/       these pages
scripts/    the git hooks
src/        the program and its tests, see architecture.md
```
