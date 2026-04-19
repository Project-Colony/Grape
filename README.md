# Grape

**Desktop music player in Rust — fast, minimalist, OS-agnostic.**

Grape scans your local library, reads file metadata, displays cover art and
albums, and plays your music without friction. Inspired by
[Dopamine](https://github.com/digimezzo/dopamine), built on
[iced](https://iced.rs) + [rodio](https://github.com/RustAudio/rodio).

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-Multimedia-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

---

## Installation

### Via Colony (recommended)

Grape is a [Colony](https://github.com/Project-Colony/Colony) app. Install Colony,
open the **Multimedia** category — Grape will appear there and be updated
automatically on every new release.

### Direct binary

Grab the portable executable for your platform from the
[Releases](https://github.com/Project-Colony/Grape/releases/latest) page:

| Platform                | Asset                  |
|-------------------------|------------------------|
| Linux (x86_64)          | `grape-linux`          |
| Windows (x86_64)        | `grape-windows.exe`    |
| macOS (Apple Silicon)   | `grape-macos`          |
| macOS (Intel)           | `grape-macos-x86`      |

No installer — download, `chmod +x` on Unix, run. A new release is published
automatically on every merged change.

### From source

```bash
git clone https://github.com/Project-Colony/Grape.git
cd Grape
cargo run --release -- /path/to/my/library
```

With no argument, Grape uses the folder configured in preferences (default `~/Music`).

---

## Features

**Library**
- Local scan based on an `Artist/Album/` structure (or albums at the root).
- Tag reading via [lofty](https://github.com/Serial-ATA/lofty-rs): title, artist,
  album-artist, compilation, duration, bitrate, codec, year, genre.
- Metadata pipeline with standard detection (ALBUMARTIST → ARTIST → "Various Artists"
  for compilations).
- Optional enrichment via Last.fm, with TTL cache and per-album manual overrides.
- Versioned local cache (`.grape_cache/`) with signature-based invalidation (mtime + size).
- Cover art: embedded covers first, fallback to local images in the album folder.

**Playback**
- Audio engine on [rodio](https://github.com/RustAudio/rodio), configurable output
  (device + sample rate).
- Gapless playback, crossfade, automix.
- 3/5-band equalizer (Flat / Bass / Treble / Vocal presets + custom).
- Volume normalization, Quiet/Normal/Loud levels.

**Navigation**
- Artists, Albums, Tracks, Genres, Folders views.
- Playlists: create, rename, delete, drag-to-reorder.
- Split-pane playback queue (now-playing on the left, upcoming queue on the right).
- Search and multi-criteria filters (genre, year, duration, codec).

**System integration**
- Native "Now Playing" notifications (opt-in, via
  [notify-rust](https://github.com/hoodie/notify-rust)).
- Tray icon, global shortcuts, autostart — detected with graceful fallback per OS.
- Hardware acceleration detection.

**Misc**
- Bilingual FR / EN interface with automatic system-language detection.
- Persistent preferences (audio, appearance, accessibility).

---

## Supported audio formats

`mp3` · `flac` · `wav` · `ogg` · `m4a` · `aac` · `opus` · `aif` / `aiff` · `wma`

Actual support depends on the codecs available on your system.

---

## Development

```bash
cargo check                  # fast typecheck
cargo test --lib             # unit tests
cargo run --profile fast     # dev build with optimized deps (fast compile, fast run)
cargo run --release          # release build
```

The `fast` profile combines `opt-level = 3` on dependencies with minimal debug
info on user code: perfect for iterating on the UI without waiting for a full
release build.

### Repository layout

```
assets/   visuals, fonts, logos
docs/     product and technical documentation
scripts/  git hooks, setup
src/      code (iced UI, library, player, system integration)
tests/    integration tests (cache, player, metadata)
```

### Local cache

After a scan, Grape writes to `.grape_cache/` (relative to the library by default,
configurable as an absolute path in preferences):

- `index.json` — global signature index.
- `folders/` — one JSON per album folder.
- `tracks/` — signatures + per-track metadata.
- `covers/` — cached cover art.
- `metadata/` — cached Last.fm responses.

A "Clear cache" button in preferences wipes the tree.

---

## Roadmap

See [`tasks/roadmap/roadmap.md`](tasks/roadmap/roadmap.md).

## License

[MIT](LICENSE) © 2026 MotherSphere
