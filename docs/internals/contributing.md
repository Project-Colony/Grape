# Working on Grape

## Building

```bash
cargo check                  # typecheck, the fast loop
cargo test --lib             # the unit tests inside src/
cargo test                   # unit plus integration tests
cargo run --profile fast     # optimized dependencies, incremental user code
cargo build --release        # what ships
```

Requires Rust 1.90 or newer, and on Linux `libasound2-dev`, `pkg-config`,
`clang` and `lld`. The reasons are in
[../guide/install.md](../guide/install.md#build-from-source), and the linker one
in particular is worth reading before spending an afternoon on a link error.

### Profiles

Five entries below, and they are not all in the same file — four profiles plus
one per-package override.

| Profile | Where | What it is for |
|---|---|---|
| `dev` | `.cargo/config.toml` | `opt-level = 0`, split debuginfo, incremental |
| `dev.package."*"` | `Cargo.toml` | dependencies at `opt-level = 3` even in dev — without it, indexing and audio init are unusably slow in debug |
| `fast` | `Cargo.toml` | inherits `release`, adds incremental and line-table debuginfo. The one to use while iterating on the UI |
| `release` | `.cargo/config.toml` | `opt-level = 3`, thin LTO, one codegen unit, stripped, `panic = "abort"` |
| `release-small` | `.cargo/config.toml` | `opt-level = "z"`, fat LTO — smaller binary, slower build |

`.cargo/config.toml` also sets `-D warnings` and enables `clippy::pedantic` and
`clippy::nursery` for every build in the workspace, with a documented list of
allows. A new warning anywhere fails the build, which is deliberate.

## Checks before committing

There is no CI that runs them, so the hooks are how they get run.

```bash
./scripts/setup-hooks.sh         # rustfmt + clippy + cargo test on every commit
./scripts/setup-hooks-light.sh   # rustfmt + clippy only
```

The light hook is a few seconds; the full one is closer to a minute. If you
install the light one, run the tests yourself. `git commit --no-verify` skips
both, and should stay rare.

Formatting is `rustfmt.toml`; lints are the `.cargo/config.toml` block above.

## The test suite

| | |
|---|---|
| `src/**` | 50 `#[test]` functions — settings normalization and clamping, the theme migration, album-artist inference, cache-path validation, EQ clamping, the migration marker |
| `tests/cache_tests.rs` | 20 tests over the `.grape_cache/` round trip and signature invalidation |
| `tests/metadata_online_tests.rs` | 20 tests over Last.fm response parsing, the TTL, and the backoff |
| `tests/player_tests.rs` | 23 tests, **19 of them `#[ignore]`** |

### What the tests do not cover

Say this plainly, because it is the part that surprises people.

- **The audio path is barely tested.** Nineteen of the twenty-three player
  tests need a real output device and are marked `#[ignore]`, so a default
  `cargo test` runs four of them. Playback, seeking, gapless and the EQ are
  verified by hand.
- **Nothing runs the tests automatically.** `.github/workflows/` contains
  `release.yml` and nothing else — no test, clippy or fmt job. CI compiles four
  targets when a release is cut, and never runs a test. The git hooks in
  `scripts/` are the entire safety net, and they are opt-in.
- **Windows and macOS are compile-verified only.** The release matrix builds
  both, so those paths typecheck every release, but the LaunchAgent, the HKCU
  autostart, the `tray-icon` backend and the global hotkeys have no automated
  exercise on either OS. Linux is the platform actually run.
- **Last.fm is never contacted by a test.** The online tests cover parsing,
  caching, the TTL and the backoff against fixtures. The live API is not in the
  loop, and the code path needs a user-supplied key to do anything at all.

Running the ignored player tests, on a machine with working audio output:

```bash
cargo test --test player_tests -- --ignored
```

## Conventions

- **Everything in the repository is English** — code, identifiers, comments,
  commit messages, and these documents. French is a shipped *UI locale*, which
  is a different thing: it lives in `src/ui/i18n.rs` as `STRINGS_FR` and stays
  there.
  through the binary: the startup failure message in `src/main.rs` is still
  French.
- **Commits are Conventional Commits.** release-please parses them to decide
  the next version and to write `CHANGELOG.md`; a `fix:` is a patch, a `feat:`
  a minor. `CHANGELOG.md` is never edited by hand.
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
6. **Make something read it.** Several existing settings stop at step 5 —
   crossfade, automix, volume normalization, the update settings and others,
   all catalogued in
   [../guide/configuration.md](../guide/configuration.md#settings-that-persist-and-change-nothing).
   A control that appears to work and does not is worse than no control.

## Repository layout

```
assets/     logos, application icons, the bundled JetBrains Mono Nerd Font
docs/       these pages
scripts/    the git hooks
src/        the program — see architecture.md
tests/      integration tests
```
