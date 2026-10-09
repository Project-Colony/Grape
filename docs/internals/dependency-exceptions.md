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
| RUSTSEC-2024-0429 | `glib` 0.18.5 | unsound | `tray-icon` drops its gtk3 dependency |
| RUSTSEC-2026-0192 | `ttf-parser` 0.25.1 | unmaintained | `fontdb` and `ab_glyph` move to another font parser |
| RUSTSEC-2024-0436 | `paste` 1.0.15 | unmaintained | `lofty`, `metal`, `pulp` and `rav1e` drop it |
| RUSTSEC-2024-0370 | `proc-macro-error` 1.0.4 | unmaintained | `tray-icon` drops its gtk3 dependency |

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

## RUSTSEC-2024-0429 / GHSA-wrw7-89jp-8q8g: `glib` 0.18.5

**Status:** accepted, Dependabot alert dismissed as `not_used`.

**The defect.** `VariantStrIter::impl_get` passed an out-pointer as `&p`
instead of `&mut p` to the variadic `g_variant_get_child`. Under optimization
the write is discarded, so `CStr::from_ptr` receives NULL and dereferences it.
It is a crash, not remote code execution. Fixed in glib 0.20.0.

**Why it cannot be upgraded away.** `glib` arrives transitively:

```
grape -> tray-icon -> muda / libappindicator -> gtk 0.18.2 -> glib 0.18.5
```

The released gtk3-rs is frozen at 0.18.2 with a hard `glib 0.18` bound, so no
version of `tray-icon` escapes it. (The gtk3-rs repository is *not* archived:
there is a 0.19.0-alpha, but the published crate has not moved, and the
genuinely dead link is `libappindicator` 0.9.0, last published 2023-10-01 with
a null `repository` field.)

**Why it is not exploitable here.** The vulnerable iterator is only
constructible through the public `glib::Variant::array_iter_str`
(`glib-0.18.5/src/variant.rs:843-854`); `VariantStrIter::new` is `pub(crate)`
(`variant_iter.rs:109`). Grepping every vendored package in the dependency
graph for `array_iter_str` / `VariantStrIter` returns matches only inside
`glib` itself, and the only ones outside `#[cfg(test)]` are its own definition.
Grape never calls glib directly.

**It is also no longer compiled on Linux.** Since the Linux tray moved to
`ksni`, `cargo tree -e normal --target x86_64-unknown-linux-gnu` contains no
gtk, glib, gdk, atk, pango, cairo-rs or libappindicator at all. The crate
remains in `Cargo.lock` (the lockfile is the union over every target, and
`tray-icon` still serves Windows and macOS), which is why lockfile-based
tooling (Dependabot, `cargo audit`, `cargo deny`) keeps reporting it and why a
dismissal and an ignore, rather than a code change, are what close it.

**Re-check when:** `tray-icon` drops its gtk3 dependency (upstream has been
attempting a ksni-based Linux backend since 2024: tauri-apps/tray-icon#201,
muda#239), or a gtk3 advisory lands that is reachable rather than unsound-only,
or Grape stops shipping a Windows/macOS tray.

## Unmaintained crates with no successor release

An `unmaintained` notice has no patched version by definition, and Grape uses
none of these three directly. Two are proc-macros that run at compile time and
put no code of their own in the binary; the third reads fonts, not files a
user opens.

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
- **RUSTSEC-2024-0370, `proc-macro-error` 1.0.4.** A proc-macro, used by
  `glib-macros` and `gtk3-macros`: the same gtk3 stack as `glib` above, never
  compiled on Linux. Re-check together with RUSTSEC-2024-0429.

The gtk3-rs `unmaintained` notices this page used to list (RUSTSEC-2024-0412,
-0413, -0415, -0416, -0418, -0419, -0420) were withdrawn from the RustSec
database on 2026-08-14 and no longer fire.
