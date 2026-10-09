# Dependency exceptions

Advisories that are knowingly accepted, and why. Every entry needs a reason
that survives review and a named trigger for re-checking it.

The `advisories` job in `.github/workflows/ci.yml` runs `cargo audit
--deny warnings` against `Cargo.lock` on every pull request, every push to
`main` and once a week, so an advisory published against a crate already on
`main` shows up without waiting for the next change. It fails on
vulnerabilities and on informational advisories alike (unsound, unmaintained,
yanked), so each entry below is also an `--ignore` in that job, with a
one-line reason next to it. The two lists must match: adding
an exception means adding it in both places, and lifting one means removing it
from both.

`cargo audit` reads the RustSec database directly. Dependabot does not: it
alerts only on advisories that carry a GHSA alias, and it does not surface
informational ones at all, so most of the entries below never appear in the
Security tab.

| Advisory | Crate | Kind | Lifted when |
|---|---|---|---|
| RUSTSEC-2026-0253 | `lru` 0.16.4 | unsound | iced ships a `cryoglyph` on `lru` >= 0.18.2 |
| RUSTSEC-2026-0192 | `ttf-parser` 0.25.1 | unmaintained | `fontdb` and `ab_glyph` move to another font parser |
| RUSTSEC-2024-0436 | `paste` 1.0.15 | unmaintained | `lofty`, `metal`, `pulp` and `rav1e` drop it |

To see where a crate comes from: `cargo tree -i <crate> --target all`.

## RUSTSEC-2026-0253: `lru` 0.16.4

**The defect.** `LruCache::pop()` is not panic safe: if a key's `Drop`
panics during `pop()`, the freed node stays linked, and a later eviction
writes through the dangling pointer. Fixed in 0.18.2. The advisory has no GHSA
alias, so Dependabot never raised it.

**Why it is not exploitable here.** It needs a cache key whose `Drop` can
panic, caught with `catch_unwind`. Grape never touches `lru`, and its one user,
`cryoglyph`, keys its glyph cache on cosmic-text's `CacheKey`, a `Copy` struct
with no `Drop` at all (`cryoglyph-0.1.0/src/text_atlas.rs:24`).

**Why it cannot be upgraded away.**

```
grape -> iced 0.14 -> iced_wgpu 0.14 -> cryoglyph 0.1.0 -> lru 0.16.4
```

`cryoglyph` 0.1.0 is the latest release and requires `lru` 0.16, and iced 0.14
is the latest iced. Nothing in Grape can move it.

**Re-check when:** iced releases a version whose `cryoglyph` takes `lru` 0.18.2
or newer.

## Unmaintained crates with no successor release

An `unmaintained` notice has no patched version by definition, and Grape uses
neither of these directly. `paste` is a proc-macro that runs at compile time
and puts no code of its own in the binary; `ttf-parser` reads fonts, not files
a user opens.

- **RUSTSEC-2026-0192, `ttf-parser` 0.25.1.** Used by `fontdb` (through
  `cosmic-text`, under iced) and by `owned_ttf_parser` / `ab_glyph` (through
  `sctk-adwaita`, under winit). It parses the fonts iced and winit load.
  Re-check when iced and winit release versions whose font stack has moved to
  another parser.
- **RUSTSEC-2024-0436, `paste` 1.0.15.** A proc-macro. Used by `lofty` (a
  direct dependency, still on it in its latest release, 0.25.4), `metal`
  (under `wgpu-hal`), `pulp` (under `exr`) and `rav1e` (under `ravif`); the
  last two arrive through `image`, under iced. Re-check when `lofty` releases
  a version without it; the other three follow wgpu and `image`.

## Lifted

- **RUSTSEC-2024-0429 (`glib` 0.18.5, unsound) and RUSTSEC-2024-0370
  (`proc-macro-error` 1.0.4, unmaintained).** Both came from the gtk3 stack
  behind `tray-icon`'s default `libappindicator` feature, which Grape never
  compiled: Linux uses `ksni`, and Windows and macOS need no feature. With
  default features off, gtk, glib, gdk, atk, pango, cairo-rs and
  libappindicator left `Cargo.lock`, and the ignores went with them.
