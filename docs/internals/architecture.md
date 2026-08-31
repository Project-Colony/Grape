# Architecture

Grape is one binary with one thread of control: iced owns the event loop,
everything else is a plain module it calls into. There is no actor system and
no background service; four operations are dispatched onto the tokio executor,
and everything else runs inline on the UI thread.

## Module map

```
src/
├── main.rs                     binary: logging, argv, hands off to ui::run
├── lib.rs                      library: re-exports the modules for the test suite
├── config/
│   ├── mod.rs                  UserSettings, the Colony roots, session state
│   └── migrate.rs              one-time copy off the pre-Colony layout
├── library/
│   ├── mod.rs                  the scanner and the Catalog
│   ├── cache.rs                the .grape_cache/ format
│   └── metadata/
│       ├── mod.rs              tag reading, via lofty
│       └── online.rs           Last.fm album.getInfo, cached
├── player.rs                   rodio: stream, sink, decode, EQ, gain, speed
├── eq.rs                       the band model — frequencies, gains, presets
├── playlist.rs                 Playlist, PlaylistManager, PlaybackQueue
├── notifications.rs            "Now Playing" desktop notification
├── system_integration/
│   ├── mod.rs                  SystemIntegration, availability, SystemAction
│   ├── common.rs               global hotkeys, via global-hotkey
│   ├── tray/
│   │   ├── mod.rs              picks a backend at compile time
│   │   ├── sni.rs              Linux: ksni / StatusNotifierItem
│   │   └── native.rs           Windows and macOS: tray-icon
│   ├── linux.rs                autostart: XDG .desktop
│   ├── macos.rs                autostart: LaunchAgent
│   ├── windows.rs              autostart: HKCU Run
│   └── unsupported.rs          everything reports unavailable
└── ui/
    ├── mod.rs                  ui::run
    ├── state.rs                UiState and every sub-state
    ├── message.rs              UiMessage
    ├── style.rs                Colony palette → iced styling tokens
    ├── i18n.rs                 UiStrings, one static per locale
    ├── app/
    │   ├── mod.rs              GrapeApp, new(), subscription()
    │   ├── view.rs             the layout
    │   ├── update.rs           message handling
    │   ├── selection.rs        artist / album / genre / folder / track
    │   ├── playback.rs         transport, gapless, the confirmed actions
    │   ├── filters.rs          search matching, sorting, the async fetch
    │   └── preferences/        mod, general, appearance, accessibility, audio, helpers
    └── components/             the Iced widgets, listed below
```

`main.rs` declares the same modules privately that `lib.rs` exposes publicly.
That duplication is what lets `tests/` link against a library while the binary
stays a binary.

### Components

Most are compositions of iced widgets: `artists_panel`, `albums_grid`,
`genres_panel`, `folders_panel`, `songs_panel` (which carries the inline album
genre/year editor), `player_bar`, `playlist_view`, `queue_view`,
`audio_settings` (the equalizer).

Two are real custom widgets, implementing iced's `Widget` trait rather than
composing:

- **`seek_area`** — click-anywhere-to-seek on the progress bar, which needs
  hit-testing a bare rectangle rather than a button's bounds.
- **`anchored_overlay`** — positions an overlay against an anchor element, for
  the logo menu and the speed popup.

## From a file on disk to a playing track

```
   Music/Artist/Album/01 - Title.flac
              │
   scan_library ── size + mtime ──▶ .grape_cache/index.json
              │                         │ unchanged? reuse the cached entry
              │◀────────────────────────┘
              │ changed, or absent
        lofty tags ──▶ Track { title, artist, album_artist, compilation,
              │                duration, bitrate, codec, year, genre, cover }
              │
          Catalog ──▶ UiState ──▶ view()
              │
      click ──┴──▶ Player::load ──▶ symphonia decode ──▶ EQ ──▶ gain
                                                                 │
                                            rodio sink (speed, volume) ──▶ device
```

### Scanning

`scan_library` walks the root and recognises three shapes: `Artist/Album/`
folders, an album sitting directly at the root, and loose files at the root.
Year and track number are parsed from `2022 - Album` and `01 - Title`, then
overwritten by tags wherever tags exist — the folder name is only a fallback.

Albums found at the root have no artist directory to name them, so
`infer_album_artist` derives one from the tags: `Various Artists` if the tracks
carry a compilation flag (`COMPILATION` / `TCMP`), else a shared `ALBUMARTIST`,
else a shared `ARTIST`, else `None` and the caller's localized "unknown artist"
label. Five unit tests pin the rule down, alongside the ones covering
`scan_library` itself, the folder and file-name parsers, and the genre/year
resolvers — 21 in `src/library/mod.rs` in total.

### The cache

`.grape_cache/` lives beside the library by default, and holds `index.json`
plus four directories: `folders/`, `tracks/`, `covers/` and `metadata/`.

Invalidation is per track, not per folder or per library. The index stores a
signature — file size and modification time — and a track is re-read only when
its signature no longer matches. Everything else is reused as-is, which is why
a rescan of an unchanged library costs a directory walk and a few stat calls.

`CACHE_VERSION` (currently 5) is the escape hatch: bumping it makes the whole
index unusable and forces a full re-read, which is the right move whenever the
serialized shape changes. At the end of a scan, `finalize` drops entries no
album referenced, so deleting music eventually reclaims the cache too.

The cache path is user-configurable. Relative paths resolve against the library
root; absolute paths are used as given; a relative path containing `..` is
rejected and reset, so the setting cannot be pointed at an arbitrary directory
by a hand-edited or migrated preferences file.

### Cover selection

`select_album_cover` tries, in order:

1. an image file in the album folder — `cover`, `folder`, `front`, `artwork`,
   `album`, in that priority, with `.jpg` / `.jpeg` / `.png` / `.webp`;
2. a picture embedded in a track's tags, skipping anything over 10 MB;
3. a cover cached by an earlier scan that is still on disk.

**External files beat embedded ones.** Older documentation claimed the reverse.
Whatever is chosen is copied into `.grape_cache/covers/` under a name derived
from the source path and its mtime, so iced loads a stable file rather than
holding image bytes in the catalog.

### Metadata precedence

Three sources, and the order never changes:

```
user override  ▸  file tags  ▸  Last.fm
```

The user's per-album genre/year edit is written to `.grape_cache/metadata/` and
applied by `apply_user_metadata_override` after the scan builds the album, so
it wins over the tags. `merge_album_online_metadata` fills only what is still
missing, which means Last.fm can never overwrite something you set or something
the file already said.

### Playback

`Player::load` opens the file, builds a decoder, wraps it in
`AudioProcessingSource` and appends it to a rodio sink. The wrapper is a
per-sample chain: the peaking-biquad EQ per channel, then the fixed
Quiet/Normal/Loud gain. Playback speed and the volume slider are not in that
chain — they are set on the sink itself, which is why changing either takes
effect without reloading the track.

Seeking prefers a seekable decoder and calls `try_seek`. When that is not
available or fails, it falls back to reopening the file and calling
`skip_duration`, which is correct but pays for decoding everything it skips.
The seekable decoder is dropped explicitly before the fallback so the file
handle is released first.

Gapless playback is not a crossfade and not a preload thread: on each playback
tick, when the current track is within 500 ms of its end, the next queued track
is appended to the same sink. rodio plays the queued source the instant the
first one runs out.

Output device selection has two states. `System` takes the default. `UsbHeadset`
enumerates cpal devices and takes the first whose interface type is USB, or
whose device type is headset or headphones, or whose name or extended
description mentions "usb" or "headset". When the requested device or sample
rate cannot be opened, `open_stream` falls back and records an `AudioFallback`,
which `GrapeApp::new` drains and turns into a visible notice.

## What runs off the UI thread

Grape enables iced's `tokio` feature, so `Task::perform` has a multi-threaded
runtime under it. Four things use it: the native folder picker
(`ui/app/update.rs`), the M3U export (`ui/app/update.rs`), the library scan
(`ui/app/playback.rs::begin_scan`) and the Last.fm request
(`ui/app/filters.rs::request_album_metadata`).

The scan is the important one: `scan_library` is entirely synchronous code, but
it is handed to the executor rather than run inline, so the window stays live
while it walks the tree. What the banner shows is not progress — `ScanTick`
fires every 120 ms and adds 0.02 to the bar, wrapping from 0.95 back to 0.2
(`ui/app/update.rs`), an indeterminate animation. Nothing reports files done
against files total.

The Last.fm request makes three attempts in all, each capped at 15 s by the
caller and at 8 s by the reqwest client itself, with a 500 ms doubling delay
between them, and delivers its result as an ordinary `UiMessage`.

Everything else — tag reading, cache I/O, playlist writes — is synchronous on
the UI thread, and is fast enough that it does not matter.

## Configuration and the Colony layout

`config::roots()` resolves the config and data directories once per process via
`colony_ui::paths`, and calls `migrate::run` on the way. The accessors are
infallible: a program that cannot resolve a home directory should still start
against a degraded path rather than refuse to run, so the fallback is the
pre-Colony layout.

`migrate::run` is guarded by a `.colony-migrated` marker holding a schema
number. It copies rather than moves — the two roots can be on different
filesystems, and a half-finished rename has no safe recovery — and leaves the
originals in place. A failure writes no marker, so the next launch retries.

`UserSettings::normalized()` is the single place ranges are enforced, and it
runs on load rather than on save. It also holds the theme migration: a
preferences file written by an older Grape carries a flat `theme_mode` string,
which is read once, mapped to a family/variant pair, and dropped on the next
save.

## Theming

`ui/style.rs` does not own any colours. It calls `colony_ui::set_active_theme`
with the family and variant from preferences, then reads
`colony_ui::active_palette()` back and converts it into the token struct the
components use. High contrast and the accent choice are pushed into colony-ui
by two further calls in the same function (`set_high_contrast`,
`set_active_accent`). Text scale, accessible text size, the focus ring and
reduce-transparency never leave Grape — they are copied straight into the token
struct.

The consequence for anyone adding a theme: the catalog is in
Project-Colony-Resources, not here. Grape only stores two strings and asks for
whatever they name, and validates on load that the family still exists.

## System integration

`SystemIntegration::sync` reconciles preferences against what the platform
actually offers. Every capability is opt-in, and `apply_availability` turns a
preference back off and rewrites the file when the platform reports the feature
unavailable — so an unchecked box after a restart is the platform's answer, not
a bug.

The tray backend is chosen at compile time. Linux uses `ksni` because
`tray-icon`'s Linux backend goes through libappindicator and GTK 3, which needs
a GTK main loop that iced/winit never runs; building a tray with it aborts the
process. Both speak `org.kde.StatusNotifierItem`, so this is a change of
implementation, not of protocol. Windows and macOS keep `tray-icon`.

`hardware_acceleration` is worth naming for what it is not: `availability()`
returns a hard-coded `true` on Linux, macOS and Windows and `false` only on the
unsupported stub. Nothing probes a GPU, and the setting selects no renderer —
its only effect is helping decide whether the `SystemIntegration` object is
kept alive at all.

## Tests

`tests/cache_tests.rs` (20), `tests/metadata_online_tests.rs` (20) and
`tests/player_tests.rs` (23, of which 19 are `#[ignore]` because they need a
real audio device), plus 50 `#[test]` functions inside `src/`. What that
coverage does and does not prove is in
[contributing.md](contributing.md#what-the-tests-do-not-cover).
