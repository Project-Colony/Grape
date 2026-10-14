# Roadmap

What is built, what is next, and what is only an idea. Anything listed as done
is done in the code, not in intent: the things that were previously listed
here as complete but are not have been moved down to *Started and unfinished*.

## Done

**Library**

- Scan of `Artist/Album/` folders, albums sitting at the library root, and
  loose root tracks (`src/library/mod.rs`).
- Year and track number parsed from folder and file names, overridden by tags.
- Album-artist inference for root albums: compilation flag, then a shared
  `ALBUMARTIST`, then a shared `ARTIST`.
- Tag reading through lofty: title, artist, album artist, compilation,
  duration, bitrate, codec, year, genre, embedded cover.
- Cover selection: external file, then embedded picture, then cached copy.
- A versioned library cache under the Colony cache root, outside the music
  folder, with a per-track size+mtime signature, per-folder JSON, a cover cache
  and a metadata cache; garbage-collected at the end of a scan.
- Optional Last.fm album enrichment with a TTL cache and 429/503 backoff.
- Per-album manual genre/year overrides that outrank both tags and Last.fm.

**Playback**

- rodio engine: load, play, pause, seek (seekable decoder, with a
  `skip_duration` fallback).
- Gapless playback: the next track is appended to the sink 500 ms before the
  current one ends.
- Queue with next/previous, shuffle and repeat off/one/all.
- 3- and 5-band peaking equalizer with Flat/Bass/Treble/Vocal/Custom presets,
  clamped to ±12 dB.
- Playback speed 0.5× to 2.0×, and Quiet/Normal/Loud output levels.
- Configurable output sample rate, with a fallback and a visible notice when
  the device or rate cannot be opened.

**Interface**

- Four tabs (Artists, Genres, Albums, Folders) over a three- or two-column
  layout.
- Accent-insensitive search, with four toggles that widen it to genre, year,
  duration and codec.
- Full-screen split queue view, and a full-screen playlist view.
- Mini-player mode.
- Playlists: create, rename, delete, reorder, remove, export to M3U, persisted
  as JSON.
- In-window keyboard control, and three global hotkeys.
- Preferences across General / Appearance / Accessibility / Audio, on
  colony-ui's shared Preferences kit, written atomically and clamped on load.
- colony-ui's typography everywhere: one text scale and a dyslexia font that
  swaps every text at once.
- French and English, with system-locale detection.
- Two custom iced widgets: click-to-seek and anchored overlays.

**Platform and release**

- Colony filesystem layout, with a one-time copy migration off the pre-Colony
  location.
- Session resume: track, position, tab and queue index.
- Themes from Colony's shared palette catalog, with a migration off the old
  flat theme enum.
- Opt-in "Now Playing" notifications, tray icon and autostart, each disabling
  itself when the platform says no.
- Linux tray on `ksni` / StatusNotifierItem, after `tray-icon`'s GTK 3 backend
  turned out to abort the process.
- System media controls: MPRIS 2 on Linux and the System Media Transport
  Controls on Windows.
- Signed releases: every release since v0.3.1 carries ed25519 `.sig`, `.meta`
  and `.meta.sig` files. The release workflow now builds and checks the four
  targets and hands them to the shared Project-Colony sign-and-publish
  workflow, which signs, verifies on the release and only then publishes; a
  failed run leaves a draft rather than shipping unsigned. The next release is
  the first to go through it. Windows Authenticode through SignPath is wired
  and waits for SignPath to accept the project.
- CI on every pull request: rustfmt, clippy with the pedantic and nursery
  lint set, build and `cargo test` on Linux, Windows and macOS, a build at the
  minimum supported Rust, a RustSec advisory check over `Cargo.lock`, and a
  `colony.json` check.

## Started and unfinished

Each of these has a control in Preferences and no implementation behind it.
Either finish it or remove the control; leaving it is the worst of the three.

- **Audio stability mode.** Auto / Stable / Low latency, stored and unused.
- **Time format.** 24 h / 12 h, stored and unused.
- **Hardware acceleration.** Availability is a per-platform constant and the
  setting selects no renderer.
- **Open on Playlists.** The startup screen option opens the same view as
  *Home*; nothing opens the playlists panel at launch.
- **Play history.** `history.json` has a path and a Clear button; nothing
  writes it.

## Next

- **Decode what is scanned.** Opus, AIFF and WMA are indexed and fail on play.
  Either enable the decoders or stop scanning those extensions: silently
  listing a file that cannot be played is the wrong half of both options.
- **Expose the Last.fm key.** There is no field for `metadata_api_key` in
  Preferences; enrichment can only be enabled by hand-editing
  `preferences.json`.
- **Expose sorting.** `SortOption` implements alphabetical, by album, by year
  and by duration, and no control emits it, so the order is permanently by album.
- **Cover the audio path.** Twenty of the twenty-five player tests are
  `#[ignore]` for want of an output device.
- **Report real scan progress.** The scan already runs on the tokio executor
  (`ui/app/playback.rs`), but the banner's bar is a 120 ms cosmetic loop that
  adds 0.02 and wraps (`ui/app/update.rs`). Nothing counts files done against
  files total, so on a large library the bar says nothing.

## Later

- Richer metadata: more online sources, better genre handling.
- Writing tags back to files, and per-track editing.
- A library watcher for automatic rescans, and duplicate detection.
- Smart playlists, and per-album or per-playlist resume.
- Synced lyrics, and scrobbling to Last.fm or ListenBrainz.
- Rebindable keyboard shortcuts.
- An audio visualizer, and preloading or upsampling options.
