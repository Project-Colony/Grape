# Grape

Grape est un lecteur musique/audio desktop en Rust, inspiré par Dopamine. Le projet vise une
expérience rapide et claire pour explorer une bibliothèque locale et lancer la lecture.

Grape est une application [Colony](https://github.com/Project-Colony/Colony) : les binaires
cross-plateforme (Linux, Windows, macOS Apple Silicon + Intel) sont publiés automatiquement
sur la page [Releases](https://github.com/Project-Colony/Grape/releases) et Colony les récupère
pour vous. License : [MIT](LICENSE).

## État actuel

- **UI desktop Iced** : layout complet (top bar, colonnes, player bar) + vues Playlist/Queue/Préférences.
- **Scan local** : dossier bibliothèque configurable (préférences ou argument CLI) + lecture `Artiste/Album` (ou albums à la racine) + construction du catalogue.
- **Métadonnées audio** : durée, bitrate, codec, année, genre et covers embarquées via `lofty` + parsing des noms de dossiers/fichiers.
- **Cache local** : index piste + cache album/pistes/covers + cache de métadonnées (locales & en ligne) dans un dossier configurable (par défaut `.grape_cache/`).
- **Jaquettes** : priorité aux covers embarquées, fallback sur images locales mises en cache.
- **Métadonnées en ligne** : enrichissement optionnel via Last.fm (API key + TTL) + surcharges manuelles par album.
- **Lecture audio** : module `player` basé sur `rodio`, sortie audio configurable (périphérique + sample rate) + options gapless/crossfade/automix.
- **Égaliseur** : 3 ou 5 bandes, presets (Flat/Bass/Treble/Vocal) + mode custom.
- **Volume** : normalisation + niveaux (Quiet/Normal/Loud) + volume par défaut configurable.
- **File de lecture** : queue basée sur la playlist active + vue dédiée + actions Next/Previous + réordonnancement.
- **Navigation enrichie** : onglets Genres/Folders + recherche/tri appliqués aux listes + filtres (genre, année, durée, codec).
- **Préférences UI** : écrans General/Appearance/Accessibility/Audio avec sections accordéon + persistance locale (certaines options restent déclaratives).
- **Playlists** : création/renommage/suppression + ajout de pistes + réordonnancement/suppression d'items, persistance JSON locale.
- **Internationalisation** : interface bilingue (français/anglais) via le module `i18n`, avec détection automatique de la langue système.
- **Notifications** : notifications natives "Now Playing" (optionnelles, via `notify-rust`), avec rate-limit.
- **Intégration système** : module par OS (Linux/macOS/Windows) couvrant autostart, tray, raccourcis globaux et détection de l'accélération matérielle — opt-in avec fallback automatique.

## Stack technique

- Rust (édition 2021)
- Iced (UI)
- Rodio (audio)
- Lofty (métadonnées audio)

## Structure du dépôt

- `assets/` : visuels et assets UI (logos, fonts, maquettes, captures).
- `docs/` : documentation produit et technique.
- `scripts/` : scripts de développement (hooks git, setup).
- `src/` : code applicatif (UI, bibliothèque, player, intégration système).
- `tasks/roadmap/` : roadmap publique.
- `tests/` : tests d'intégration (cache, player, métadonnées en ligne).

## Démarrage rapide

```bash
cargo run -- /chemin/vers/ma/library
```

Si aucun chemin n'est fourni, Grape utilise le dossier configuré dans les préférences
(par défaut `~/Music`).

### Structure attendue de la bibliothèque

Grape scanne une structure simple de dossiers :

```
Library/
  Artiste/
    2003 - Album Name/
      01 - Titre.mp3
      02 - Autre titre.flac
```

Formats supportés pour le scan : `mp3`, `flac`, `wav`, `ogg`, `m4a`, `aac`, `opus`, `aif`, `aiff`, `wma`.
La prise en charge réelle peut dépendre des codecs disponibles sur la machine.

### Cache local

Après un scan réussi, Grape conserve un cache dans le dossier configuré (par défaut
`.grape_cache/` à la racine de la bibliothèque si le chemin est relatif) :

- `index.json` : index global des signatures de pistes.
- `folders/` : un fichier JSON par dossier d'album.
- `tracks/` : cache des signatures et métadonnées locales par piste.
- `covers/` : jaquettes mises en cache (covers embarquées ou images locales).
- `metadata/` : métadonnées en ligne mises en cache (Last.fm).

Le cache est invalidé par piste en comparant la signature (taille + date de modification).

Les préférences exposent un chemin de cache configurable (action “Vider le cache”). Un
chemin relatif est interprété dans la bibliothèque scannée, un chemin absolu est utilisé tel
quel.

## Documentation

- Vue d'ensemble : `docs/docs.md`
- Source & modules : `src/src.md`
- Scripts de développement : `scripts/README.md`
- Roadmap : `tasks/roadmap/roadmap.md`
- Analyse UI : `tasks/ui_analysis.md`

## Feuille de route (résumé)

- Étendre les métadonnées (sources en ligne, tags avancés, covers hi-res).
- Améliorer la recherche/tri (filtres avancés, multi-critères).
- Améliorer l'indexation (métadonnées enrichies, cache plus fin, genres réels).
