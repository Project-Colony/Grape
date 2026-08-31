# Roadmap

## Phase 1 — MVP (complete)

- Working local scan (`library.rs`).
- Iced desktop UI (layout + navigation).
- Audio metadata via `lofty` (durations, tags, embedded covers).
- `.grape_cache/` JSON cache (track index + per-folder cache).
- Local cover-art cache.
- Audio playback wired to the UI (selection + play/pause/seek).
- Playback queue (Next/Previous) and shuffle/repeat states.
- Dedicated queue view + actions (clear/reorder/remove).
- UI preferences (General/Appearance/Accessibility/Audio) + local persistence.
- Multiple themes + accents + interface density.
- Accessibility (text size, contrast, reduced motion, subtitles).
- Advanced audio (output device, gapless/crossfade, EQ presets).
- Persisted playlists (local JSON) + dedicated view + editing (reorder/remove).
- EQ and audio-output options (device + sample rate).
- French/English internationalization (`i18n.rs`) with automatic detection.
- UI filters (genre, year, duration, codec) applied to lists.
- Custom Iced widgets (SeekArea, AnchoredOverlay).
- Native "Now Playing" notifications (opt-in, `notify-rust`).
- Per-OS system integration: autostart, tray, global shortcuts, hardware-acceleration detection.
- Library crate (`lib.rs`) + integration tests (cache, player, online metadata).

## Phase 2 — Experience

- Advanced multi-criteria search.
- Finer indexing cache (JSON/SQLite) per track.
- Real genres via metadata + online sources.
- Advanced Preferences actions (detailed logs).

## Phase 3 — Audio quality & polish

- Richer cover art and metadata (embedded + online).
- Accessibility, themes, UX polish.

## Backlog — Ideas

- Synced lyrics + "Now Playing" display.
- Smart playlists (rules) + resume playback.
- Audio tag editing (write to files).
- Library watcher + duplicate detection.
- Configurable keyboard shortcuts + mini-player.
