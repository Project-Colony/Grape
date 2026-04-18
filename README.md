# Grape

**Lecteur musique desktop en Rust — rapide, minimaliste, agnostique du système d'exploitation.**

Grape scanne votre bibliothèque locale, lit les métadonnées des fichiers, affiche jaquettes et
albums, et joue votre musique sans friction. Inspiré par [Dopamine](https://github.com/digimezzo/dopamine),
construit sur [iced](https://iced.rs) + [rodio](https://github.com/RustAudio/rodio).

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-Multimedia-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

---

## Installation

### Via Colony (recommandé)

Grape est une application [Colony](https://github.com/Project-Colony/Colony). Installez Colony
puis ouvrez la catégorie **Multimédia** — Grape s'y trouvera et sera mis à jour automatiquement
à chaque nouvelle release.

### Binaire direct

Récupérez l'exécutable portable pour votre plateforme sur la page
[Releases](https://github.com/Project-Colony/Grape/releases/latest) :

| Plateforme              | Asset                  |
|-------------------------|------------------------|
| Linux (x86_64)          | `grape-linux`          |
| Windows (x86_64)        | `grape-windows.exe`    |
| macOS (Apple Silicon)   | `grape-macos`          |
| macOS (Intel)           | `grape-macos-x86`      |

Aucun installateur : téléchargez, `chmod +x` sur Unix, lancez. Une nouvelle release est
publiée automatiquement à chaque commit.

### Depuis le source

```bash
git clone https://github.com/Project-Colony/Grape.git
cd Grape
cargo run --release -- /chemin/vers/ma/library
```

Sans argument, Grape utilise le dossier configuré dans les préférences (défaut `~/Music`).

---

## Fonctionnalités

**Bibliothèque**
- Scan local basé sur la structure `Artiste/Album/` (ou albums posés à la racine).
- Lecture des tags via [lofty](https://github.com/Serial-ATA/lofty-rs) : titre, artiste,
  album-artist, compilation, durée, bitrate, codec, année, genre.
- Pipeline metadata avec détection standard (ALBUMARTIST → ARTIST → "Various Artists"
  pour les compilations).
- Enrichissement optionnel via Last.fm, avec cache TTL et surcharges manuelles par album.
- Cache local versionné (`.grape_cache/`) avec invalidation par signature (mtime + taille).
- Jaquettes : priorité aux covers embarquées, fallback sur images locales du dossier album.

**Lecture**
- Moteur audio [rodio](https://github.com/RustAudio/rodio), sortie configurable (device +
  sample rate).
- Gapless playback, crossfade, automix.
- Égaliseur 3/5 bandes (presets Flat / Bass / Treble / Vocal + custom).
- Normalisation de volume, niveaux Quiet/Normal/Loud.

**Navigation**
- Vues Artistes, Albums, Pistes, Genres, Dossiers.
- Playlists : création, renommage, suppression, réordonnancement par drag.
- File de lecture en split-pane (morceau en cours à gauche, suite de la queue à droite).
- Recherche et filtres multi-critères (genre, année, durée, codec).

**Intégration système**
- Notifications natives "Now Playing" (opt-in, via [notify-rust](https://github.com/hoodie/notify-rust)).
- Icône de tray, raccourcis globaux, autostart — détectés et fallback gracieux selon l'OS.
- Détection de l'accélération matérielle.

**Divers**
- Interface bilingue FR / EN avec détection automatique de la langue système.
- Préférences persistantes (audio, appearance, accessibility).

---

## Formats audio supportés

`mp3` · `flac` · `wav` · `ogg` · `m4a` · `aac` · `opus` · `aif` / `aiff` · `wma`

La prise en charge réelle dépend des codecs disponibles sur votre système.

---

## Développement

```bash
cargo check                  # typecheck rapide
cargo test --lib             # tests unitaires
cargo run --profile fast     # build dev avec deps optimisées (compile vite, exec rapide)
cargo run --release          # build release
```

Le profil `fast` combine `opt-level = 3` sur les deps et debug-info minimal sur le code user :
parfait pour itérer sur l'UI sans attendre le build release complet.

### Structure du dépôt

```
assets/   visuels, fonts, logos
docs/     documentation produit et technique
scripts/  hooks git, setup
src/      code (UI iced, bibliothèque, player, intégration système)
tests/    tests d'intégration (cache, player, métadonnées)
```

### Cache local

Après un scan, Grape écrit dans `.grape_cache/` (relatif à la bibliothèque par défaut,
configurable en chemin absolu dans les préférences) :

- `index.json` — index global des signatures.
- `folders/` — un JSON par dossier d'album.
- `tracks/` — signatures + métadonnées par piste.
- `covers/` — jaquettes cachées.
- `metadata/` — réponses Last.fm mises en cache.

Un bouton "Vider le cache" dans les préférences nettoie l'arborescence.

---

## Roadmap

Voir [`tasks/roadmap/roadmap.md`](tasks/roadmap/roadmap.md).

## License

[MIT](LICENSE) © 2026 MotherSphere
