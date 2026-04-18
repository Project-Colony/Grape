# Source — Vue d'ensemble

Ce dossier contient le code applicatif Rust du lecteur Grape.

## Entrée

- `main.rs`
  - Initialise le logging.
  - Lance l'application Iced via `ui::run` (scan déclenché côté UI).
- `lib.rs`
  - Expose les modules publics (`config`, `eq`, `library`, `notifications`, `player`, `playlist`, `system_integration`, `ui`) pour les tests d'intégration.

## Modules

- `config.rs`
  - Préférences utilisateur (thèmes, accents, densité, accessibilité, audio avancé, stockage, intégration système).
  - Chargement/sauvegarde JSON dans `~/.config/Colony/Grape/preferences.json` (Windows : `%LOCALAPPDATA%/Colony/Grape/preferences.json`) (+ history/logs).
- `library.rs`
  - Scan du disque et construction d'un `Catalog`.
  - Convention : dossiers `Artiste/Album` + fichiers audio.
  - Parsing des années/numéros depuis les noms de dossier/fichier.
  - Métadonnées audio (durée, codec, genre, année, cover embarquée).
  - Enrichissement optionnel en ligne (Last.fm) + surcharges manuelles par album.
  - Détection des jaquettes et cache local.
- `library/cache.rs`
  - Cache JSON dans le dossier configuré (par défaut `.grape_cache/` si chemin relatif).
  - Index global de signatures de pistes + cache par dossier d'album.
  - Cache des covers + metadata locales + metadata online + overrides utilisateur.
  - Invalidation par signature (taille + date de modification).
- `library/metadata.rs`
  - Lecture des métadonnées audio via `lofty`.
- `library/metadata/online.rs`
  - Enrichissement album (genre/année) via Last.fm + cache.
- `player.rs`
  - Abstraction de lecture audio (`rodio`).
  - Sortie audio configurable (périphérique + sample rate) + EQ/normalisation.
  - Méthodes : `load`, `play`, `pause`, `seek`.
- `eq.rs`
  - Modèle d'égaliseur 3/5 bandes (presets + custom).
- `playlist.rs`
  - Modèle de playlist + sérialisation JSON (`~/.config/Colony/Grape/playlist.json`, Windows : `%LOCALAPPDATA%/Colony/Grape/playlist.json`).
  - File de lecture `PlaybackQueue` utilisée par Next/Previous + vue queue dédiée.
- `notifications.rs`
  - Notification native "Now Playing" via `notify-rust`.
  - Opt-in via préférences (`notifications_enabled`, `now_playing_notifications`).
  - Fallback silencieux sur wasm32.
- `system_integration/`
  - `mod.rs` : orchestration (`SystemIntegration`, `SystemIntegrationAvailability`, `SystemAction`).
  - `common.rs` : tray (`tray-icon`) + raccourcis globaux (`global-hotkey`).
  - `linux.rs`, `macos.rs`, `windows.rs` : autostart et détection par OS.
  - `unsupported.rs` : fallback pour plateformes non couvertes.
  - Fonctionnalités : autostart, tray (Quit), raccourcis globaux (Ctrl+Alt+P/→/←), détection accélération matérielle.

## UI (`ui/`)

- `mod.rs` : point d'entrée (`ui::run`), expose `GrapeApp`.
- `state.rs` : état UI centralisé (`UiState`).
- `message.rs` : enum des messages UI (`UiMessage`).
- `style.rs` : styles Iced centralisés.
- `i18n.rs` : internationalisation français/anglais, détection automatique de la langue système.

### Logique applicative (`ui/app/`)

- `mod.rs` : `GrapeApp` (struct principale Iced).
- `view.rs` : rendu de la vue principale.
- `update.rs` : gestion des messages.
- `selection.rs` : logique de sélection (artiste/album/piste).
- `playback.rs` : gestion de la lecture.
- `filters.rs` : filtres UI (genre, année, durée, codec).
- `preferences/` : vues Préférences découpées :
  - `mod.rs`, `general.rs`, `appearance.rs`, `accessibility.rs`, `audio.rs`, `helpers.rs`.

### Composants (`ui/components/`)

- `ArtistsPanel` (`artists_panel.rs`)
- `AlbumsGrid` (`albums_grid.rs`)
- `GenresPanel` (`genres_panel.rs`)
- `FoldersPanel` (`folders_panel.rs`)
- `SongsPanel` (`songs_panel.rs`) — inclut l'éditeur de métadonnées album
- `PlayerBar` (`player_bar.rs`)
- `SeekArea` (`seek_area.rs`) — widget custom pour le clic-to-seek
- `AnchoredOverlay` (`anchored_overlay.rs`) — widget custom pour les menus/overlays ancrés
- `PlaylistView` (`playlist_view.rs`)
- `QueueView` (`queue_view.rs`)
- `AudioSettings` (`audio_settings.rs`) — égaliseur

## Tests d'intégration (`../tests/`)

- `cache_tests.rs` : tests du cache local.
- `player_tests.rs` : tests du module de lecture.
- `metadata_online_tests.rs` : tests de l'enrichissement Last.fm.

## Notes

- La lecture audio est branchée à la sélection de pistes dans l'UI.
- La playlist est connectée au modèle et persistée localement.
- Les préférences modifient l'état UI et sont persistées.
- Les intégrations système sont opt-in et chargées uniquement si activées.
- L'interface supporte le français et l'anglais (détection automatique).
