# Scripts de Développement Grape

Ce dossier contient des scripts utiles pour le développement de Grape.

## Hooks Git Pre-Commit

### Version Complète (Recommandée)

```bash
./scripts/setup-hooks.sh
```

Cette version exécute avant chaque commit :
- ✅ Vérification du formatage (rustfmt)
- ✅ Analyse statique (clippy)
- ✅ Tests (cargo test)

**Temps d'exécution** : ~30-60 secondes selon la machine

### Version Légère (Rapide)

```bash
./scripts/setup-hooks-light.sh
```

Cette version exécute seulement :
- ✅ Vérification du formatage (rustfmt)
- ✅ Analyse statique (clippy)
- ❌ Pas de tests

**Temps d'exécution** : ~5-10 secondes

⚠️ **Important** : N'oubliez pas de lancer les tests manuellement !

### Désactiver Temporairement les Hooks

Si vous avez besoin de commit rapidement sans passer par les vérifications :

```bash
git commit --no-verify -m "Votre message"
```

⚠️ **À utiliser avec parcimonie** : Les vérifications sont là pour une bonne raison !

## Commandes Utiles

### Formatage du Code

```bash
# Formater tout le code
cargo fmt --all

# Vérifier le formatage sans modifier
cargo fmt --all -- --check
```

### Analyse Statique

```bash
# Lancer clippy
cargo clippy --all-targets --all-features

# Mode strict (comme en CI)
cargo clippy --all-targets --all-features -- -D warnings
```

### Tests

```bash
# Lancer tous les tests
cargo test --all-features

# Tests avec output détaillé
cargo test --all-features -- --nocapture

# Tests d'un module spécifique
cargo test --test player_tests
cargo test --test cache_tests
cargo test --test metadata_online_tests

# Tests avec couverture
cargo install cargo-tarpaulin
cargo tarpaulin --all-features
```

### Build

```bash
# Build debug
cargo build

# Build release (optimisé)
cargo build --release

# Build ultra-optimisé pour taille minimale
cargo build --profile release-small
```

## Notes de Performance

**Tous ces outils sont pour le développement uniquement.**

Ils n'ajoutent **AUCUNE** consommation CPU/RAM au runtime de l'application :
- Les hooks ne s'exécutent que pendant le développement (git commit)
- rustfmt/clippy ne modifient pas le binaire final
- Les profils de build sont optimisés pour la performance

Le binaire final reste aussi léger et rapide qu'avant ! 🚀
