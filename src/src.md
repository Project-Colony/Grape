# Source — Overview

This folder contains the Rust application code for the Grape music player.

## Entry

- `main.rs`
  - Initializes logging.
  - Starts the Iced application via `ui::run` (scan triggered on the UI side).
- `lib.rs`
  - Exposes public modules (`config`, `eq`, `library`, `notifications`, `player`, `playlist`, `system_integration`, `ui`) for integration tests.

## Modules

- `config.rs`
  - User preferences (themes, accents, density, accessibility, advanced audio, storage, system integration).
  - JSON load/save in `~/.config/Colony/Grape/preferences.json` (Windows: `%LOCALAPPDATA%/Colony/Grape/preferences.json`) (+ history/logs).
- `library.rs`
  - Disk scan and `Catalog` construction.
  - Convention: `Artist/Album` folders + audio files.
  - Year/track-number parsing from folder/file names.
  - Audio metadata (duration, codec, genre, year, embedded cover).
  - Optional online enrichment (Last.fm) + per-album manual overrides.
  - Cover detection and local cache.
- `library/cache.rs`
  - JSON cache in the configured folder (default `.grape_cache/` if the path is relative).
  - Global track-signature index + per-album-folder cache.
  - Covers cache + local metadata + online metadata + user overrides.
  - Signature-based invalidation (size + modification time).
- `library/metadata.rs`
  - Audio metadata reading via `lofty`.
- `library/metadata/online.rs`
  - Per-album enrichment (genre/year) via Last.fm + cache.
- `player.rs`
  - Audio playback abstraction (`rodio`).
  - Configurable audio output (device + sample rate) + EQ/normalization.
  - Methods: `load`, `play`, `pause`, `seek`.
- `eq.rs`
  - 3/5-band equalizer model (presets + custom).
- `playlist.rs`
  - Playlist model + JSON serialization (`~/.config/Colony/Grape/playlist.json`, Windows: `%LOCALAPPDATA%/Colony/Grape/playlist.json`).
  - `PlaybackQueue` used by Next/Previous + a dedicated queue view.
- `notifications.rs`
  - Native "Now Playing" notification via `notify-rust`.
  - Opt-in via preferences (`notifications_enabled`, `now_playing_notifications`).
  - Silent fallback on wasm32.
- `system_integration/`
  - `mod.rs`: orchestration (`SystemIntegration`, `SystemIntegrationAvailability`, `SystemAction`).
  - `common.rs`: tray (`tray-icon`) + global shortcuts (`global-hotkey`).
  - `linux.rs`, `macos.rs`, `windows.rs`: autostart and per-OS detection.
  - `unsupported.rs`: fallback for uncovered platforms.
  - Features: autostart, tray (Quit), global shortcuts (Ctrl+Alt+P/→/←), hardware-acceleration detection.

## UI (`ui/`)

- `mod.rs`: entry point (`ui::run`), exposes `GrapeApp`.
- `state.rs`: central UI state (`UiState`).
- `message.rs`: UI message enum (`UiMessage`).
- `style.rs`: centralized Iced styles.
- `i18n.rs`: French/English internationalization with automatic system-language detection.

### Application logic (`ui/app/`)

- `mod.rs`: `GrapeApp` (main Iced struct).
- `view.rs`: main view rendering.
- `update.rs`: message handling.
- `selection.rs`: selection logic (artist/album/track).
- `playback.rs`: playback control.
- `filters.rs`: UI filters (genre, year, duration, codec).
- `preferences/`: Preferences views split into:
  - `mod.rs`, `general.rs`, `appearance.rs`, `accessibility.rs`, `audio.rs`, `helpers.rs`.

### Components (`ui/components/`)

- `ArtistsPanel` (`artists_panel.rs`)
- `AlbumsGrid` (`albums_grid.rs`)
- `GenresPanel` (`genres_panel.rs`)
- `FoldersPanel` (`folders_panel.rs`)
- `SongsPanel` (`songs_panel.rs`) — includes the album metadata editor
- `PlayerBar` (`player_bar.rs`)
- `SeekArea` (`seek_area.rs`) — custom widget for click-to-seek
- `AnchoredOverlay` (`anchored_overlay.rs`) — custom widget for anchored menus/overlays
- `PlaylistView` (`playlist_view.rs`)
- `QueueView` (`queue_view.rs`)
- `AudioSettings` (`audio_settings.rs`) — equalizer

## Integration tests (`../tests/`)

- `cache_tests.rs`: local-cache tests.
- `player_tests.rs`: playback-module tests.
- `metadata_online_tests.rs`: Last.fm enrichment tests.

## Notes

- Audio playback is wired to track selection in the UI.
- The playlist is connected to the model and persisted locally.
- Preferences drive UI state and are persisted.
- System integrations are opt-in and only loaded when enabled.
- The interface supports French and English (automatic detection).
