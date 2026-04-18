# Roadmap

## Phase 1 — MVP (terminée)

- Scan local fonctionnel (`library.rs`).
- UI desktop Iced (layout + navigation).
- Métadonnées audio via `lofty` (durées, tags, covers embarquées).
- Cache JSON `.grape_cache/` (index piste + cache par dossier).
- Jaquettes en cache local (covers).
- Lecture audio connectée à l'UI (sélection + play/pause/seek).
- Queue de lecture (Next/Previous) et états shuffle/repeat.
- Vue dédiée de la queue + actions (vider/réordonner/supprimer).
- Préférences UI (General/Appearance/Accessibility/Audio) + persistance locale.
- Thèmes multiples + accents + densité d'interface.
- Accessibilité (taille de texte, contraste, réduction d'animations, sous-titres).
- Audio avancé (output device, gapless/crossfade, EQ presets).
- Playlists persistées (JSON local) + vue dédiée + édition (réordonnancement/suppression).
- EQ et options de sortie audio (périphérique + sample rate).
- Internationalisation français/anglais (`i18n.rs`) avec détection automatique.
- Filtres UI (genre, année, durée, codec) appliqués aux listes.
- Widgets Iced custom (SeekArea, AnchoredOverlay).
- Notifications natives "Now Playing" (opt-in, `notify-rust`).
- Intégration système par OS : autostart, tray, raccourcis globaux, détection accélération matérielle.
- Library crate (`lib.rs`) + tests d'intégration (cache, player, métadonnées en ligne).

## Phase 2 — Expérience

- Recherche avancée multi-critères.
- Cache d'indexation plus fin (JSON/SQLite) par piste.
- Genres réels via métadonnées + sources en ligne.
- Actions Préférences avancées (logs détaillés).

## Phase 3 — Qualité audio & finition

- Jaquettes et métadonnées enrichies (embed + online).
- Accessibilité, thèmes, polish UX.

## Backlog — Idées d'ajouts

- Paroles synchronisées + affichage "Now Playing".
- Smart playlists (règles) + reprise de lecture.
- Édition des tags audio (écriture dans les fichiers).
- Watcher de bibliothèque + détection de duplicatas.
- Raccourcis clavier configurables + mini-player.
