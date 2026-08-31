# Dependency exceptions

Advisories that are knowingly accepted, and why. Every entry needs a reason
that survives review and a named trigger for re-checking it.

## RUSTSEC-2024-0429 / GHSA-wrw7-89jp-8q8g — `glib` 0.18.5

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
version of `tray-icon` escapes it. (The gtk3-rs repository is *not* archived —
there is a 0.19.0-alpha — but the published crate has not moved, and the
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
remains in `Cargo.lock` — the lockfile is the union over every target, and
`tray-icon` still serves Windows and macOS — which is why lockfile-based
tooling (Dependabot, `cargo audit`, `cargo deny`) keeps reporting it and why a
dismissal, rather than a code change, is what closes the alert.

**Re-check when:** `tray-icon` drops its gtk3 dependency (upstream has been
attempting a ksni-based Linux backend since 2024 — tauri-apps/tray-icon#201,
muda#239), or a gtk3 advisory lands that is reachable rather than unsound-only,
or Grape stops shipping a Windows/macOS tray.

## Informational advisories on the same stack

These fire in `cargo audit` / `cargo deny` but not in Dependabot, which does
not surface RustSec informational advisories. All are `unmaintained` notices on
the gtk3-rs crates reached only through `tray-icon`'s Windows/macOS-irrelevant
Linux backend, and none has a patched version in existence:
RUSTSEC-2024-0412 (`gdk`), -0413 (`atk`), -0415 (`gtk`), -0416 (`atk-sys`),
-0418 (`gdk-sys`), -0419 (`gtk3-macros`), -0420 (`gtk-sys`). RUSTSEC-2024-0370
(`proc-macro-error`) arrives with them.

## RUSTSEC-2026-0253 — `lru` 0.16.4

Not dismissible by us: `lru` is pinned by `iced` 0.14 via
`cryoglyph -> iced_wgpu`. Re-check when iced updates it.
