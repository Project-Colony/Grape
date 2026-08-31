# Roadmap

What is built, what is next, and what is only an idea. Anything listed as done
is done in the code, not in intent — the things that were previously listed
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
- Cover selection — external file, then embedded picture, then cached copy.
- Versioned `.grape_cache/` with a per-track size+mtime signature, per-folder
  JSON, a cover cache and a metadata cache; garbage-collected at the end of a
  scan.
- Optional Last.fm album enrichment with a TTL cache and 429/503 backoff.
- Per-album manual genre/year overrides that outrank both tags and Last.fm.

**Playback**

- rodio engine: load, play, pause, seek (seekable decoder, with a
  `skip_duration` fallback).
- Gapless playback — the next track is appended to the sink 500 ms before the
  current one ends.
- Queue with next/previous, shuffle and repeat off/one/all.
- 3- and 5-band peaking equalizer with Flat/Bass/Treble/Vocal/Custom presets,
  clamped to ±12 dB.
- Playback speed 0.5×–2.0×, and Quiet/Normal/Loud output levels.
- Configurable output sample rate, with a fallback and a visible notice when
  the device or rate cannot be opened.

**Interface**

- Four tabs — Artists, Genres, Albums, Folders — over a three- or two-column
  layout.
- Accent-insensitive search, with four toggles that widen it to genre, year,
  duration and codec.
- Full-screen split queue view, and a full-screen playlist view.
- Mini-player mode.
- Playlists: create, rename, delete, reorder, remove, export to M3U, persisted
  as JSON.
- In-window keyboard control, and three global hotkeys.
- Preferences across General / Appearance / Accessibility / Audio, written
  atomically and clamped on load.
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
- Signed releases: four targets, ed25519 signatures verified at sign time, and
  a release that fails rather than shipping unsigned.

## Started and unfinished

Each of these has a control in Preferences and no implementation behind it.
Either finish it or remove the control; leaving it is the worst of the three.

- **Crossfade.** A 0–12 s slider that is stored, clamped and never read.
- **Automix.** A toggle with no consumer.
- **Volume normalization.** Copied into the audio config and never applied.
- **Audio stability mode.** Auto / Stable / Low latency, stored and unused.
- **Time format.** 24 h / 12 h, stored and unused.
- **Interface density** and **transparency & blur.** Both only label themselves
  in the Appearance preview; neither changes the layout or the rendering.
- **Update settings.** Auto-check, channel and auto-install have no updater
  behind them. Updates come through Colony — the honest fix is probably to
  delete the section.
- **Hardware acceleration.** Availability is a per-platform constant and the
  setting selects no renderer.
- **Play history.** `history.json` has a path and a Clear button; nothing
  writes it.

## Next

- **Decode what is scanned.** Opus, AIFF and WMA are indexed and fail on play.
  Either enable the decoders or stop scanning those extensions — silently
  listing a file that cannot be played is the wrong half of both options.
- **Expose the Last.fm key.** There is no field for `metadata_api_key` in
  Preferences; enrichment can only be enabled by hand-editing
  `preferences.json`.
- **Expose sorting.** `SortOption` implements alphabetical, by album, by year
  and by duration, and no control emits it — the order is permanently by album.
- **Run the tests in CI.** `.github/workflows/` has only `release.yml`. The
  hooks under `scripts/` are opt-in and the only thing running clippy, rustfmt
  or `cargo test` today.
- **Cover the audio path.** Nineteen of the twenty-three player tests are
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
