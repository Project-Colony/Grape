# Installing Grape

Grape is a single binary. There is no installer, no service, no account, and
nothing to configure before the first launch.

## Via Colony (recommended)

Grape is published on [Colony](https://github.com/Project-Colony/Colony) under
**Multimedia**. Install Colony, find Grape in that category, install it, and
updates arrive through the launcher.

`colony.json` sets `"signed": true`, so Colony 0.8.0 and later verifies the
ed25519 signature attached to the release asset before installing it and
refuses an asset that does not match.

## Direct binary download

Every release ships exactly four assets, each with a detached `.sig` beside it:

| Platform | Asset | Target triple |
|---|---|---|
| Linux x86_64 | `grape-linux` | `x86_64-unknown-linux-gnu` |
| Windows x86_64 | `grape-windows.exe` | `x86_64-pc-windows-msvc` |
| macOS (Apple Silicon) | `grape-macos` | `aarch64-apple-darwin` |
| macOS (Intel) | `grape-macos-x86` | `x86_64-apple-darwin` |

```bash
chmod +x grape-linux && ./grape-linux
```

Releases are cut by release-please: a release exists when its release pull
request is merged, not on every merge to `main`. See
[../internals/packaging.md](../internals/packaging.md).

## Build from source

```bash
git clone https://github.com/Project-Colony/Grape
cd Grape
cargo build --release
```

Requires **Rust 1.90 or newer**. That floor is not a preference — it is what
`ordered-float` 5.5.0 demands, reached through `iced` → `iced_wgpu` →
`wgpu-hal`, and `Cargo.toml` records it as `rust-version`.

Two prerequisites are easy to miss:

- **Linux system packages: `libasound2-dev` and `pkg-config`.** `alsa-sys` is
  the only dependency that needs a development package; it finds libasound
  through pkg-config. Nothing else does — the GTK 3 stack left the tree when
  the Linux tray moved to `ksni`, and D-Bus is spoken by pure-Rust `zbus`.
- **`clang` and `lld`.** `.cargo/config.toml` pins the Linux linker to
  `clang` with `-fuse-ld=lld`. On a machine without them the build fails at
  link time, which reads like a Rust problem and is not.

That same file also caps the build at 4 jobs and applies `-D warnings` plus
`clippy::pedantic` and `clippy::nursery` to everything, so a warning anywhere
fails the build.

### Running it

```bash
cargo run --release -- /path/to/library    # scan this folder
cargo run --release                        # scan the folder from preferences
```

The first positional argument overrides the configured library folder for that
run. With no argument Grape uses `library_folder` from preferences, which
defaults to `$HOME/Music` (`%USERPROFILE%\Music` on Windows).

## What the scanner expects

Grape reads three shapes and needs no naming discipline beyond them:

```
Library/
├── Artist/
│   └── 2022 - Album/          ← Artist/Album/Track, the common case
│       ├── 01 - Title.flac
│       └── cover.jpg
├── Some Compilation/          ← an album sitting directly at the root
│   └── 01 - Title.mp3
└── Loose Track.mp3            ← a file at the root, with no album folder
```

Year and track number are parsed from `2022 - Album` and `01 - Title`, then
overridden by the file's tags wherever tags exist. Tags win; the folder name is
only a fallback.

An album found directly at the root is filed under an album artist inferred
from its tags: `Various Artists` if the tracks are flagged as a compilation
(`COMPILATION` / `TCMP`), otherwise a shared `ALBUMARTIST`, otherwise a shared
`ARTIST`, otherwise the localized "unknown artist" label.

Cover art is picked in this order, and the first hit wins: an image file in the
album folder named `cover`, `folder`, `front`, `artwork` or `album` (`.jpg`,
`.jpeg`, `.png`, `.webp`), then a picture embedded in a track's tags, then a
cover cached by an earlier scan that is still on disk.

Recognised extensions: `mp3`, `flac`, `wav`, `ogg`, `m4a`, `aac`, `opus`,
`aif`, `aiff`, `wma`. Three of those scan into the library but will not decode
— see [troubleshooting.md](troubleshooting.md#some-files-appear-in-the-library-but-will-not-play).

## Where Grape keeps its files

Grape follows the Colony filesystem layout: `<root>/Colony/Grape/`.

| | Linux | Windows | macOS |
|---|---|---|---|
| config | `~/.config/Colony/Grape/` | `%LOCALAPPDATA%\Colony\Grape\` | `~/Library/Application Support/Colony/Grape/` |
| data | `~/.local/share/Colony/Grape/` | `%LOCALAPPDATA%\Colony\Grape\` | `~/Library/Application Support/Colony/Grape/` |

Config holds `preferences.json` and `playlist.json` — what you chose. Data
holds `history.json`, `session.json` and `logs/` — what the program produced.

The library cache is separate and lives beside the music: `.grape_cache/` at
the library root by default, configurable in preferences.

### Upgrading from a pre-Colony install

Grape used to keep everything under `~/.config/Colony/Grape` (and read `$HOME`
directly, ignoring `XDG_CONFIG_HOME`, and used `~/.config` on macOS instead of
`~/Library/Application Support`). The first launch after the move **copies**
those files into the Colony roots and writes a `.colony-migrated` marker.

It copies rather than moves: the originals stay where they were, deliberately,
for one release. If the migration fails partway it writes no marker, so the
next launch tries again instead of starting from an empty profile. On Windows
and on a default-XDG Linux box the two locations resolve to the same directory
and there is nothing to do.
