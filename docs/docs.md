# Grape documentation

This documentation covers the current project state, architecture, and UI choices.

## Product goals

- Browse a local library quickly.
- Start playback with no perceptible latency.
- Provide a clean, modern interface.

## Architecture (quick view)

- **Entry point**: `src/main.rs`
  - Initializes logging.
  - Starts the UI via `ui::run`, which drives the initial scan.
- **Library crate**: `src/lib.rs`
  - Exposes public modules (`config`, `eq`, `library`, `notifications`, `player`, `playlist`, `system_integration`, `ui`) for integration tests and reuse.
- **Library**: `src/library.rs` + `src/library/`
  - Folder scan, `Artist/Album/Track` structure.
  - Folder/file-name parsing for year and track number.
  - Audio metadata reading via `library::metadata` (using the `lofty` crate).
  - Embedded cover detection + fallback to local images.
  - Optional online enrichment via `library::metadata::online` (Last.fm).
- **Cache**: `src/library/cache.rs`
  - Configurable folder (default `.grape_cache/` at the library root if relative).
  - Global track-signature index + per-album-folder JSON.
  - Cover cache + local metadata cache + online metadata cache (Last.fm).
  - Track cache (signatures + local metadata).
  - Signature-based invalidation (size + modification time).
  - Manual online-metadata overrides (per album) persisted in `metadata/`.
- **Audio playback**: `src/player.rs`
  - `rodio` player (load/play/pause/seek).
  - Configurable audio output (device + sample rate) with automatic fallback.
  - EQ processing (3/5 bands + presets) and volume normalization.
- **Playlists & queue**: `src/playlist.rs`
  - Playlist model + JSON serialization (`~/.config/Colony/Grape/playlist.json`, Windows: `%LOCALAPPDATA%/Colony/Grape/playlist.json`).
  - Playback queue (`PlaybackQueue`) backed by the active playlist + dedicated view.
- **Notifications**: `src/notifications.rs`
  - Native "Now Playing" notification via `notify-rust` (opt-in).
  - Honors user preferences (`notifications_enabled`, `now_playing_notifications`).
  - Silent fallback on unsupported platforms (wasm32).
- **System integration**: `src/system_integration/`
  - `mod.rs`: orchestration (`SystemIntegration`, `SystemIntegrationAvailability`, `SystemAction`).
  - `common.rs`: tray (`tray-icon`) + global shortcuts (`global-hotkey`).
  - `linux.rs`, `macos.rs`, `windows.rs`: autostart and per-OS capability detection.
  - `unsupported.rs`: fallback for uncovered platforms.
  - Capabilities: autostart, system tray (Quit), global shortcuts (Ctrl+Alt+P/→/←), hardware-acceleration detection.
  - All integrations are opt-in with automatic fallback when unavailable.
- **UI**: `src/ui/`
  - `mod.rs`: entry point (`ui::run`), exposes `GrapeApp`.
  - `state.rs`: central UI state (`UiState`).
  - `message.rs`: UI message enum (`UiMessage`).
  - `style.rs`: centralized Iced styles.
  - `i18n.rs`: French/English internationalization with automatic system-language detection.
  - `app/`: application logic split into submodules:
    - `mod.rs`: `GrapeApp` (main Iced struct).
    - `view.rs`: main view rendering.
    - `update.rs`: message handling.
    - `selection.rs`: selection logic (artist/album/track).
    - `playback.rs`: playback control.
    - `filters.rs`: UI filters (genre, year, duration, codec).
    - `preferences/`: Preferences views split into (`general.rs`, `appearance.rs`, `accessibility.rs`, `audio.rs`, `helpers.rs`).
  - `components/`: reusable Iced components (see next section).
- **Preferences**: `src/config.rs`
  - Settings persisted in `~/.config/Colony/Grape/preferences.json` (Windows: `%LOCALAPPDATA%/Colony/Grape/preferences.json`) (+ `history.json`, `logs/`).
  - General/Appearance/Accessibility/Audio sections with UI options (themes, accents, density, accessibility, advanced audio).
  - Local actions (reindex, clear cache, reset audio) exposed in the UI.
  - Cache configuration + metadata TTL + Last.fm API key.
  - System-integration options (notifications, tray, global shortcuts, hardware acceleration).
  - Some options remain declarative (updates, privacy, performance).
- **Equalizer**: `src/eq.rs`
  - 3- or 5-band model + presets and clamped gains.
- **Integration tests**: `tests/`
  - `cache_tests.rs`: local-cache tests.
  - `player_tests.rs`: playback-module tests.
  - `metadata_online_tests.rs`: Last.fm enrichment tests.

## UI: layout and components

The current layout is structured as follows:

```
Top bar  → navigation + search + window controls
Columns  → Artists | Albums | Tracks (or Genres/Folders depending on the tab)
Footer   → player bar (transport + progress)
```

Iced components:

- `ArtistsPanel` (`src/ui/components/artists_panel.rs`)
- `AlbumsGrid` (`src/ui/components/albums_grid.rs`)
- `GenresPanel` (`src/ui/components/genres_panel.rs`)
- `FoldersPanel` (`src/ui/components/folders_panel.rs`)
- `SongsPanel` (`src/ui/components/songs_panel.rs`)
  - Inline album metadata editor (genre/year) inside the track list
- `PlayerBar` (`src/ui/components/player_bar.rs`)
- `SeekArea` (`src/ui/components/seek_area.rs`) — custom Iced widget for click-to-seek on the progress bar
- `AnchoredOverlay` (`src/ui/components/anchored_overlay.rs`) — custom Iced widget to position menus/overlays anchored to an element
- `PlaylistView` (`src/ui/components/playlist_view.rs`)
- `QueueView` (`src/ui/components/queue_view.rs`)
- `AudioSettings` (`src/ui/components/audio_settings.rs`) for the equalizer

## UI state

- `ActiveTab`: Artists / Genres / Albums / Folders.
- `SelectionState`: artist, album, genre, folder, track.
- `PlaybackState`: position, duration, playing, shuffle, repeat.
- `SearchState`: query + sort (`SortOption`) + active filters (genre, year, duration, codec).
- `UiState`: menu, open playlist, open queue, open preferences, combined states.
- `UserSettings`: preferences (appearance, accessibility, audio, storage, system integration, etc.).
- `PreferencesSection`: accordion sections (startup, language, updates, privacy, storage, system integration, performance, advanced).

## Catalog data

- Artists and albums are loaded from the local scan.
- Metadata comes from `lofty` (duration, codec, bitrate, genre, year).
- Embedded cover art takes priority, otherwise a local copy is cached.
- The Genres/Folders tabs are fed by summaries derived from the catalog.
- Online enrichment (Last.fm) is optional, via API key + per-album manual overrides.

## Assets

The `assets/` folder is dedicated to visual elements (logos, fonts, screenshots, icons).

## Current limitations

- Genres stay "Unknown" when audio tags are missing.
- The equalizer is limited to the preconfigured bands (3 or 5) with gains between -12 dB and +12 dB.
- When a selected audio device is unavailable, output falls back to the system default.
- Some Preferences options are declarative only (updates, privacy, performance).

## System integration: per-OS limits

### Windows

- **Autostart**: the entry is created/removed via the `HKCU\...\Run` registry key (requires
  a readable executable).
- **Tray**: the icon uses a minimal menu (Quit); display depends on the Windows notification
  area settings.
- **Global shortcuts**: relies on available Windows APIs; exclusive focus held by another
  app can prevent triggering.

### macOS

- **Autostart**: creates a LaunchAgent (`~/Library/LaunchAgents/com.colony.grape.plist`).
  Depending on the setup, macOS may ask for authorization to launch automatically.
- **Tray**: the icon is shown in the menu bar with a minimal menu (Quit).
- **Global shortcuts**: requires the app to have accessibility permission if macOS blocks
  global shortcuts.

### Linux

- **Autostart**: creates a `.desktop` file in `~/.config/autostart/`.
- **Tray**: depends on tray-extension support (some distributions/Wayland disable it by default).
- **Global shortcuts**: on Wayland, global shortcuts may be disabled or restricted by the
  compositor.

## Suggested next steps

- Enrich metadata (real genres, additional online sources).
- Extend preferences (advanced system actions, detailed logs).
- Improve discovery (local recommendations, multi-criteria search).

## Ideas to consider

- **Advanced library management**: file watcher for automatic re-scans, duplicate detection,
  and tag-repair tooling.
- **Metadata editing**: inline per-track tag editing, writing tags back to files, support for
  advanced fields (composer, BPM, disc number).
- **Smart playback**: smart playlists (rules), listening history, "Resume playback" per
  album/playlist, local radio based on tags.
- **Discovery & social**: synced lyrics, scrobbling integrations (Last.fm, ListenBrainz),
  local top tracks/artists display, offline recommendations.
- **User experience**: configurable keyboard shortcuts, mini-player, full-screen "Now Playing"
  mode, richer notifications.
- **Performance & audio quality**: configurable preloading, audio visualizer, multi-output
  management, per-track crossfade, upsampling options.
