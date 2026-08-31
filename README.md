<div align="center">

<img src="assets/logo.png" alt="Grape" width="180">

**A desktop music player for the music you already own.**

</div>

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-Multimedia-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

Point Grape at a folder of music. It walks the tree, reads the tags, caches what
it found, and gives you artists, albums, genres and folders to browse through —
no account, no server, no network call unless you ask for one. It is a single
binary built on [iced](https://iced.rs) and [rodio](https://github.com/RustAudio/rodio),
distributed through [Colony](https://github.com/Project-Colony/Colony).

> **Status:** the scan → cache → browse → play loop is what Grape is used for
> daily, and it is the part covered by tests: scanning, tag reading, the
> `.grape_cache` round trip, search, filters, playlists, preferences and the
> FR/EN interface. Beyond that, be warned. The audio path is verified by hand —
> 19 of the 23 player tests need a real output device and are `#[ignore]`d — and
> no CI job runs any test at all; the release workflow only compiles. Windows
> and macOS are compile-verified every release but nobody exercises their tray,
> autostart or hotkeys; Linux is the platform actually run. Three audio
> preferences are inert: **crossfade, automix and volume normalization persist
> and change nothing you can hear**. The update settings have no updater behind
> them — Colony delivers updates. Opus, AIFF and WMA files are scanned into the
> library and then fail to decode.

## Why Grape

Most local players ask you to import your music into their world first. Grape
treats the folder on disk as the truth and everything else as a cache you can
delete.

- **Your folder layout is the library.** `Artist/Album/Track` works, albums
  sitting at the root work, loose files at the root work. Nothing is moved,
  renamed or written to.
- **Nothing leaves the machine by default.** The only network call is Last.fm
  album enrichment, and it stays off until you paste in your own API key.
- **Rescans are cheap.** Every track is cached under a size + mtime signature,
  so a second scan only touches what changed.
- **Tags lose to you.** A genre or year you set by hand on an album outranks
  both the file's tags and anything Last.fm returns.

```
┌──────────────────────────────────────────────────────────────┐
│  Artists   Genres   Albums   Folders     [search] [filters]  │
├───────────────┬──────────────────┬───────────────────────────┤
│               │                  │                           │
│   Artists     │     Albums       │         Tracks            │
│               │                  │                           │
├───────────────┴──────────────────┴───────────────────────────┤
│  ◀◀  ▶  ▶▶   ──────●────────────   1:42 / 4:05      ♪  ⚙     │
└──────────────────────────────────────────────────────────────┘
```

## What it does

**Library**

- Scans a root folder given on the command line, or the one saved in
  preferences (default `~/Music`).
- Reads tags with [lofty](https://github.com/Serial-ATA/lofty-rs): title,
  artist, album artist, compilation flag, duration, bitrate, codec, year, genre,
  embedded cover art.
- Works out the album artist for root-level albums: a compilation flag gives
  *Various Artists*, otherwise a shared `ALBUMARTIST`, otherwise a shared
  `ARTIST`.
- Caches everything under `.grape_cache/` next to the library — a track index,
  per-folder entries, cover art and metadata — and invalidates per track on size
  and modification time.
- Picks cover art from the album folder first (`cover`, `folder`, `front`,
  `artwork`, `album` as `.jpg`, `.jpeg`, `.png`, `.webp`), then the embedded
  picture, then whatever the last scan cached.
- Optionally fills in missing album genre and year from Last.fm, with your own
  API key, a TTL cache and backoff when the API pushes back.

**Playback**

- Play, pause, seek, next, previous, shuffle, repeat, playback speed from 0.5×
  to 2×.
- Gapless playback: the next track is queued into the output half a second
  before the current one ends.
- A 3- or 5-band equalizer with Flat, Bass, Treble, Vocal and Custom presets,
  clamped to ±12 dB.
- Quiet / Normal / Loud output levels, and a configurable sample rate that falls
  back to the system default — with a visible notice — when the device refuses
  it.

**Browsing**

- Four tabs — Artists, Genres, Albums, Folders — over a three-column layout,
  with a player bar along the bottom.
- Search with accent-insensitive matching, four sort orders and filters on
  genre, year, duration and codec.
- A full-screen queue split into *now playing* and *up next*, and playlists you
  can create, rename, delete and reorder.
- A mini-player mode that collapses the window down to the player bar alone.
- Keyboard control inside the window (arrows, `Tab`, `Enter`, `Space`, `n`, `p`,
  `+`/`-`, `m`) and three global hotkeys: `Ctrl+Alt+P`, `Ctrl+Alt+←`,
  `Ctrl+Alt+→`.

**Around the edges**

- Opt-in desktop notifications, a tray icon and autostart, each one disabling
  itself when the platform says it is unavailable.
- French and English interface, picked from the system locale.
- Session resume: track, position, tab and queue index come back on the next
  launch.
- Themes from Colony's shared palette catalog, defaulting to Catppuccin Mocha,
  with an automatic light or dark counterpart when following the system theme.

Files that actually decode: **MP3, FLAC, WAV, OGG Vorbis, M4A/AAC**. The scanner
also accepts `.opus`, `.aif`, `.aiff` and `.wma`; those appear in the library and
fail at playback, because the decoders compiled in do not cover them.

## Installation

### Via Colony (recommended)

Install [Colony](https://github.com/Project-Colony/Colony), open the
**Multimedia** category and install Grape. Updates arrive through the launcher,
and because releases are signed, Colony verifies the signature before installing.

### Direct binary download

Grab the asset for your platform from the
[latest release](https://github.com/Project-Colony/Grape/releases/latest). Each
one ships with a matching `.sig` file.

| Platform | Asset |
|---|---|
| Linux (x86_64) | `grape-linux` |
| Windows (x86_64) | `grape-windows.exe` |
| macOS (Apple Silicon) | `grape-macos` |
| macOS (Intel) | `grape-macos-x86` |

There is no installer — download it, make it executable, run it:

```bash
chmod +x grape-linux && ./grape-linux
```

Releases are cut by release-please when its release pull request is merged, not
on every merge to `main`.

### Build from source

```bash
git clone https://github.com/Project-Colony/Grape
cd Grape
cargo build --release
```

Requires Rust 1.90 or newer. On Linux you also need `libasound2-dev` and
`pkg-config` for the audio backend, plus `clang` and `lld` — the repository pins
the Linux linker to `clang -fuse-ld=lld`, and without them the build fails at
link time in a way that looks like a Rust problem and is not.

Run it against a specific folder:

```bash
cargo run --release -- /path/to/library
```

With no argument, Grape uses the library folder from preferences.

## Documentation

Everything is in [docs/](docs/); start at the index,
[docs/README.md](docs/README.md).

| | |
|---|---|
| [docs/guide/install.md](docs/guide/install.md) | getting it running, and where it puts its files |
| [docs/guide/usage.md](docs/guide/usage.md) | the interface, end to end |
| [docs/guide/configuration.md](docs/guide/configuration.md) | every preference, including the ones that do nothing |
| [docs/guide/troubleshooting.md](docs/guide/troubleshooting.md) | no sound, no tray, no genres, no hotkeys |
| [docs/internals/architecture.md](docs/internals/architecture.md) | how the modules fit together |
| [docs/internals/contributing.md](docs/internals/contributing.md) | building, testing, and what the hooks check |
| [docs/internals/packaging.md](docs/internals/packaging.md) | how a release is built and signed |
| [docs/project/roadmap.md](docs/project/roadmap.md) | what is done and what is next |

## License

[GPL-3.0-or-later](LICENSE) © 2026 MotherSphere

You may redistribute and modify Grape under the terms of the GNU General Public
License as published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.
