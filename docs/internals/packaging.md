# How a release is made

One workflow, `.github/workflows/release.yml`, with three jobs that run in
order: decide, build, sign. It is also the only workflow in the repository —
there is no test or lint job.

## 1. release-please decides

Every push to `main` runs release-please. It reads the Conventional Commit
messages since the last tag and, when there is anything releasable, opens or
updates a release pull request that bumps the version in `Cargo.toml` and
rewrites `CHANGELOG.md`.

**Merging that pull request is what cuts a release.** Merging anything else
does not. `release_created` is false on every other push, and both later jobs
are gated on it.

Configuration lives in `release-please-config.json` (`release-type: rust`,
`bump-minor-pre-major: false`) and the current version in
`.release-please-manifest.json`. Neither `CHANGELOG.md` nor the version in
`Cargo.toml` should ever be edited by hand.

## 2. Four targets are built

| Asset | Target | Runner |
|---|---|---|
| `grape-linux` | `x86_64-unknown-linux-gnu` | `ubuntu-latest` |
| `grape-windows.exe` | `x86_64-pc-windows-msvc` | `windows-latest` |
| `grape-macos` | `aarch64-apple-darwin` | `macos-latest` |
| `grape-macos-x86` | `x86_64-apple-darwin` | `macos-latest` |

`fail-fast` is off, so one target failing does not cancel the others. The Linux
runner installs `libasound2-dev` and `pkg-config` and nothing else: `alsa-sys`
is the only crate needing a system development package. `aws-lc-sys` compiles
vendored sources with `cc`, `wayland-sys` dlopens at runtime, and D-Bus is
spoken by pure-Rust `zbus` — none of them need an apt package. The GTK 3 stack
left the tree when the Linux tray moved to `ksni`.

Each job builds `--release` for its target, copies the binary out under the
asset name, and uploads it to the GitHub release.

That build is also the only automated check Windows and macOS ever get: those
targets typecheck at release time and are never tested. See
[contributing.md](contributing.md#what-the-tests-do-not-cover).

## 3. Everything is signed

The `sign` job downloads the release assets, strips the metadata companions
(`.sig`, `.sha256`, `.txt`, `.yml`, `.json`, `.asc`), and signs each remaining
file with the Project-Colony ed25519 key from `secrets.COLONY_SIGNING_KEY_PEM`.
Each signature is verified against the derived public key immediately after
being produced, then uploaded as `<asset>.sig`.

Two deliberate details:

- **A missing secret fails the job.** The step checks for an empty key and
  exits 1 rather than continuing, so a release cannot quietly ship unsigned.
- **`gh release upload` passes `-R "${{ github.repository }}"`.** The job has no
  `actions/checkout`, so there is no git repository for `gh` to infer a target
  from. Without `-R` it dies with "not a git repository" *after* the assets are
  already signed — which is how a release once shipped unsigned while the job
  meant to prevent exactly that reported its failure too late to stop it.

## 4. Colony picks it up

`colony.json` is the launcher manifest:

```json
{
  "$schema": "https://raw.githubusercontent.com/Project-Colony/Project-Colony-Resources/main/generated/colony.schema.json",
  "name": "Grape",
  "category": "multimedia",
  "icon": "assets/icons/icon.png",
  "signed": true
}
```

It validates against the generated schema in Project-Colony-Resources.
`"signed": true` is the flag that makes Colony 0.8.0 and later verify the
`.sig` before installing and refuse an asset that does not match — which is
only meaningful because the signing job cannot be skipped.

## What is not packaged

There is no AUR package, no `.deb`, no `.rpm`, no Flatpak, no Homebrew formula,
and no installer of any kind. There is no `packaging/` directory and no
`PKGBUILD` in the tree. Grape is distributed two ways: through Colony, and as a
bare binary from the GitHub release page.
