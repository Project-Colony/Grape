# Documentation Grape

Cette documentation couvre l'état actuel du projet, l'architecture et les choix UI.

## Objectifs produit

- Explorer rapidement une bibliothèque locale.
- Lancer la lecture sans latence visible.
- Proposer une interface claire et moderne.

## Architecture (vue rapide)

- **Entrée** : `src/main.rs`
  - Initialise le logging.
  - Démarre l'UI via `ui::run`, qui pilote le scan initial.
- **Library crate** : `src/lib.rs`
  - Expose les modules publics (`config`, `eq`, `library`, `notifications`, `player`, `playlist`, `system_integration`, `ui`) pour les tests d'intégration et la réutilisation.
- **Bibliothèque** : `src/library.rs` + `src/library/`
  - Scan de dossiers, structure `Artist/Album/Track`.
  - Parsing des noms de dossiers/fichiers pour année et numéro de piste.
  - Lecture des métadonnées audio via `library::metadata` (crate `lofty`).
  - Détection de covers embarquées + fallback sur images locales.
  - Enrichissement optionnel via `library::metadata::online` (Last.fm).
- **Cache** : `src/library/cache.rs`
  - Dossier configurable (par défaut `.grape_cache/` en racine de la bibliothèque si chemin relatif).
  - Index global des signatures de pistes + JSON par dossier d'album.
  - Cache covers + cache metadata locales + cache metadata (Last.fm).
  - Cache des pistes (signatures + métadonnées locales).
  - Invalidation par signature (taille + date de modification).
  - Surcharges manuelles des métadonnées en ligne (par album) persistées dans `metadata/`.
- **Lecture audio** : `src/player.rs`
  - Player `rodio` (load/play/pause/seek).
  - Sortie audio configurable (périphérique + sample rate) + fallback automatique.
  - Traitement EQ (3/5 bandes + presets) et normalisation de volume.
- **Playlists & queue** : `src/playlist.rs`
  - Modèle de playlist + sérialisation JSON (`~/.config/Colony/Grape/playlist.json`, Windows : `%LOCALAPPDATA%/Colony/Grape/playlist.json`).
  - Queue de lecture (`PlaybackQueue`) basée sur la playlist active + vue dédiée.
- **Notifications** : `src/notifications.rs`
  - Notification native "Now Playing" via `notify-rust` (opt-in).
  - Respecte les préférences utilisateur (`notifications_enabled`, `now_playing_notifications`).
  - Fallback silencieux sur les plateformes non supportées (wasm32).
- **Intégration système** : `src/system_integration/`
  - `mod.rs` : orchestration (`SystemIntegration`, `SystemIntegrationAvailability`, `SystemAction`).
  - `common.rs` : tray (`tray-icon`) + raccourcis globaux (`global-hotkey`).
  - `linux.rs`, `macos.rs`, `windows.rs` : autostart et détection des capacités par OS.
  - `unsupported.rs` : fallback pour les plateformes non couvertes.
  - Fonctionnalités : autostart, tray system (Quit), raccourcis globaux (Ctrl+Alt+P/→/←), détection accélération matérielle.
  - Toutes les intégrations sont opt-in avec fallback automatique si non disponibles.
- **UI** : `src/ui/`
  - `mod.rs` : point d'entrée (`ui::run`), expose `GrapeApp`.
  - `state.rs` : état UI centralisé (`UiState`).
  - `message.rs` : enum des messages UI (`UiMessage`).
  - `style.rs` : styles Iced centralisés.
  - `i18n.rs` : internationalisation français/anglais avec détection automatique de la langue système.
  - `app/` : logique applicative découpée en sous-modules :
    - `mod.rs` : `GrapeApp` (struct principale Iced).
    - `view.rs` : rendu de la vue principale.
    - `update.rs` : gestion des messages.
    - `selection.rs` : logique de sélection (artiste/album/piste).
    - `playback.rs` : gestion de la lecture.
    - `filters.rs` : filtres UI (genre, année, durée, codec).
    - `preferences/` : vues Préférences découpées (`general.rs`, `appearance.rs`, `accessibility.rs`, `audio.rs`, `helpers.rs`).
  - `components/` : composants Iced réutilisables (voir section suivante).
- **Préférences** : `src/config.rs`
  - Paramètres persistés dans `~/.config/Colony/Grape/preferences.json` (Windows : `%LOCALAPPDATA%/Colony/Grape/preferences.json`) (+ `history.json`, `logs/`).
  - Sections General/Appearance/Accessibility/Audio avec options UI (thèmes, accents, densité, accessibilité, audio avancé).
  - Actions locales (reindex, clear cache, reset audio) exposées dans l'UI.
  - Configuration du cache + TTL métadonnées + clé API Last.fm.
  - Options d'intégration système (notifications, tray, raccourcis globaux, accélération matérielle).
  - Certaines options restent déclaratives (updates, privacy, performance).
- **Égaliseur** : `src/eq.rs`
  - Modèle 3 ou 5 bandes + presets et gains clampés.
- **Tests d'intégration** : `tests/`
  - `cache_tests.rs` : tests du cache local.
  - `player_tests.rs` : tests du module de lecture.
  - `metadata_online_tests.rs` : tests de l'enrichissement Last.fm.

## UI : layout et composants

La maquette actuelle est structurée ainsi :

```
Top bar  → navigation + recherche + boutons fenêtre
Colonnes → Artistes | Albums | Titres (ou Genres/Folders selon l'onglet)
Footer   → player bar (transport + progression)
```

Composants Iced :

- `ArtistsPanel` (`src/ui/components/artists_panel.rs`)
- `AlbumsGrid` (`src/ui/components/albums_grid.rs`)
- `GenresPanel` (`src/ui/components/genres_panel.rs`)
- `FoldersPanel` (`src/ui/components/folders_panel.rs`)
- `SongsPanel` (`src/ui/components/songs_panel.rs`)
  - Éditeur de métadonnées album (genre/année) dans la liste de pistes
- `PlayerBar` (`src/ui/components/player_bar.rs`)
- `SeekArea` (`src/ui/components/seek_area.rs`) — widget Iced custom pour le clic-to-seek sur la barre de progression
- `AnchoredOverlay` (`src/ui/components/anchored_overlay.rs`) — widget Iced custom pour positionner les menus/overlays ancrés à un élément
- `PlaylistView` (`src/ui/components/playlist_view.rs`)
- `QueueView` (`src/ui/components/queue_view.rs`)
- `AudioSettings` (`src/ui/components/audio_settings.rs`) pour l'égaliseur

## État UI

- `ActiveTab` : Artists / Genres / Albums / Folders.
- `SelectionState` : artiste, album, genre, dossier, piste.
- `PlaybackState` : position, durée, lecture, shuffle, repeat.
- `SearchState` : query + tri (`SortOption`) + filtres actifs (genre, année, durée, codec).
- `UiState` : menu, playlist ouverte, queue ouverte, préférences ouvertes, états combinés.
- `UserSettings` : préférences (apparence, accessibilité, audio, stockage, intégration système, etc.).
- `PreferencesSection` : sections accordéon (startup, language, updates, privacy, storage, system integration, performance, advanced).

## Données du catalogue

- Les artistes et albums sont chargés depuis le scan local.
- Les métadonnées proviennent de `lofty` (durées, codec, bitrate, genre, année).
- Les jaquettes embarquées sont prioritaires, sinon copie locale dans le cache.
- Les onglets Genres/Folders sont alimentés par des résumés dérivés du catalogue.
- L'enrichissement en ligne (Last.fm) est optionnel via clé API + surcharges manuelles par album.

## Assets

Le dossier `assets/` est dédié aux éléments visuels (logos, fonts, captures, icônes).

## Limitations actuelles

- Les genres restent « Unknown » si les tags audio sont absents.
- L'égaliseur est limité aux bandes préconfigurées (3 ou 5) avec des gains entre -12 dB et +12 dB.
- Si un périphérique audio sélectionné n'est pas disponible, la sortie repasse sur le système.
- Certaines options Préférences sont uniquement déclaratives (updates, privacy, performance).

## Intégrations système : limites par OS

### Windows

- **Démarrage** : l'entrée est créée/supprimée via le registre `HKCU\...\Run` (nécessite un
  exécutable accessible en lecture).  
- **Tray** : l'icône utilise un menu minimal (Quitter) ; l'affichage dépend des paramètres de
  zone de notification Windows.  
- **Raccourcis globaux** : repose sur les APIs Windows disponibles ; un focus exclusif par une
  autre app peut empêcher le déclenchement.  

### macOS

- **Démarrage** : création d'un LaunchAgent (`~/Library/LaunchAgents/com.colony.grape.plist`).
  Selon la configuration, macOS peut demander une autorisation pour lancer automatiquement.  
- **Tray** : l'icône est affichée dans la barre de menu, avec un menu minimal (Quitter).  
- **Raccourcis globaux** : nécessite que l'app ait l'autorisation d'accessibilité si macOS bloque
  les raccourcis globaux.  

### Linux

- **Démarrage** : création d'un fichier `.desktop` dans `~/.config/autostart/`.  
- **Tray** : dépend du support de l'extension tray (certaines distributions/Wayland la désactivent
  par défaut).  
- **Raccourcis globaux** : sur Wayland, les raccourcis globaux peuvent être désactivés ou limités
  par le compositor.  

## Prochaines étapes suggérées

- Enrichir les métadonnées (genres réels, sources en ligne supplémentaires).
- Étendre les préférences (actions système avancées, logs détaillés).
- Améliorer la découverte (recommandations locales, recherche multi-critères).

## Pistes d'ajouts à considérer

- **Gestion avancée de la bibliothèque** : watcher de fichiers pour re-scan automatique, détection
  des duplicatas, et outils de réparation des tags.
- **Édition de métadonnées** : édition inline des tags piste par piste, écriture des tags dans les
  fichiers, support des champs avancés (compositeur, BPM, disc number).
- **Lecture intelligente** : smart playlists (règles), historique d'écoute, "Reprendre la lecture"
  par album/playlist, radio locale basée sur les tags.
- **Découverte & social** : paroles synchronisées, intégrations scrobbling (Last.fm, ListenBrainz),
  affichage des top tracks/artistes locaux, recommandations hors-ligne.
- **Expérience utilisateur** : raccourcis clavier configurables, mini-player, mode plein écran
  "Now Playing", notifications enrichies.
- **Performance & qualité audio** : préchargement configurable, visualiseur audio, gestion multi-sorties,
  crossfade par piste, options d'upsampling.
