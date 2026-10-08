# How a release is made

`.github/workflows/release.yml` runs three jobs in order: decide, build, then
sign and publish. The last one is the shared workflow from
Project-Colony-Resources. Pull requests are checked separately, by `ci.yml`.

## 1. release-please decides

Every push to `main` runs release-please. It reads the Conventional Commit
messages since the last tag and, when there is anything releasable, opens or
updates a release pull request that bumps the version in `Cargo.toml` and
rewrites `CHANGELOG.md`.

**Merging that pull request is what cuts a release.** Merging anything else
does not. `release_created` is false on every other push, and both later jobs
are gated on it.

Configuration lives in `release-please-config.json` (`release-type: rust`,
`bump-minor-pre-major: true`) and the current version in
`.release-please-manifest.json`. The action is given those two files and no
`release-type` input: with one, it ignores both files. Neither `CHANGELOG.md`
nor the version in `Cargo.toml` should ever be edited by hand.

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

Each job builds `--release` for its target with no build cache, copies the
binary out under the asset name, runs it with `--version` (the Intel macOS
binary, which the arm64 runner cannot run, gets an architecture check instead),
and uploads it as a workflow artifact. Nothing reaches the release from a build job, and no build
job sees a key. The Windows job also checks that `grape-windows.exe` carries
the version resource `build.rs` writes: ProductName `Grape` and the release
version as ProductVersion, which SignPath requires before it signs.

A release build is not the first time Windows and macOS compile: `ci.yml`
builds and tests on both for every pull request. Neither workflow exercises the
tray, autostart or hotkeys there; see
[contributing.md](contributing.md#what-the-tests-do-not-cover).

## 3. Everything is signed, then published

The last job calls `sign-and-publish.yml` from Project-Colony-Resources, pinned
by commit, in the same run. It checks the release is still a draft, sends
`grape-windows.exe` to SignPath for Authenticode once a `signpath-project-slug`
is set (it is not yet: SignPath has not accepted the project), then signs every
final file with the Project-Colony ed25519 key from
`secrets.COLONY_SIGNING_KEY_PEM`: `<asset>.sig`, `<asset>.meta` and
`<asset>.meta.sig`. It uploads them to the draft, downloads them again,
verifies them, and only then publishes the release.

The order matters: Authenticode rewrites the `.exe`, so an ed25519 signature
made before it would describe bytes users never download. A missing secret, a
failed check or a refused signing request leaves the release a draft.

If a run fails after the tag exists, finish the draft from the tag:

```bash
gh workflow run release.yml -R Project-Colony/Grape --ref vX.Y.Z -f tag=vX.Y.Z
```

`scripts/sign-release.sh` writes the same three files by hand. No workflow
uses it: it is the manual fallback for when the shared workflow cannot run, and
it needs the release private key, which is not in the repository. It only
signs and checks its own signatures; it does no Authenticode, and uploading the
files and publishing the draft are left to whoever runs it.

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
