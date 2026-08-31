# Configuration

Preferences open from the logo menu. They are split into four tabs — General,
Appearance, Accessibility, Audio — and every change is written to disk
immediately, through a temporary file that is renamed into place, so an
interrupted write cannot leave a half-written `preferences.json`.

Values are clamped when the file is read, not when it is written. A
hand-edited preferences file with an out-of-range value is repaired on load
rather than rejected.

## General

**Startup.** Launch at startup (per-OS autostart, see below), restore the last
session, which screen to open on, and what closing the window does — quit, or
minimize to the tray.

**Language.** French, English, or follow the system. System detection reads
`LC_ALL`, then `LC_MESSAGES`, then `LANG`, takes the language tag before any
`_`, `.` or `@`, and falls back to French when it recognises nothing.

**Privacy.** *Clear local history* deletes `history.json` from both the Colony
data root and the pre-Colony location, since the migration copies rather than
moves. Worth knowing: nothing in the current code writes that file. The
recently-played list is kept in memory for the session and is not persisted, so
on a fresh install there is nothing for the button to delete.

**Storage.** The library folder — typed, or chosen through a native folder
picker — and whether to scan it at launch. Also the cache path, and the
*Reindex library* and *Clear cache* actions.

**System integration.** Notifications, the "Now Playing" notification, the
tray icon, and the global hotkeys. All four are off by default, and any of
them turns itself back off and rewrites preferences if the platform reports it
unavailable.

**Performance.** *Limit CPU during playback* genuinely does something: it
stretches the UI refresh tick from 225 ms to 1000 ms while a track is playing
and suppresses animation ticks. The progress bar moves less smoothly; the
process does less work.

**Advanced.** *Open logs folder* creates and opens the logs directory. *Reset
preferences* restores every default.

## Appearance

Themes come from Colony's shared palette catalog, addressed as a family plus a
variant — the default is `catppuccin` / `mocha`. Preferences written by an
older Grape used a flat theme name; it is read once, mapped into the new pair,
and dropped on the next save, so nobody loses the theme they picked. With
*follow system theme* on, Grape swaps to the light or dark counterpart of the
same family.

Accent colour can be automatic or chosen. Text scale, and the effects toggles,
are here too.

## Accessibility

Large text, high contrast, reduce transparency, accessible text size, reduce
motion / animations / transitions, highlight keyboard focus, default playback
speed, and pause on focus loss.

Several of these cascade: turning on *large text* raises the text scale and the
accessible text size if they are still at their defaults, *high contrast* turns
on *increase contrast*, and *reduce motion* turns on both *reduce animations*
and *reduce transitions*. The relationship also runs backwards — raising the
text scale by hand flips *large text* on — so the summary switches never
disagree with the fine-grained ones.

*Pause on focus loss* is wired to the window's focus events and works.

## Audio

**Output.** The device choice has exactly two options: *System (default)* and
*USB headset*. This is not a device list. *USB headset* walks the available
output devices and takes the first whose reported interface type is USB, or
whose device type is headset or headphones, or whose name or extended
description mentions "usb" or "headset".

Sample rate is genuinely free: any value from 8 kHz to 192 kHz, and anything
outside that range is discarded on load and falls back to the device default.

*If the device is missing* chooses between switching to the system output and
pausing. Either way a notice appears in the window.

**Playback.** Gapless playback, on by default, works. Crossfade and automix do
not — see below.

**Volume.** *Output level* is Quiet, Normal or Loud, applying a fixed gain of
0.75, 1.0 or 1.25. The volume slider in the player bar and the default-volume
slider here are the same setting.

**Equalizer.** Three bands (100 Hz / 1 kHz / 10 kHz) or five (60 / 230 / 910 /
3600 / 14000 Hz), as a peaking biquad per band per channel at Q = 0.707. The
presets are Flat, Bass, Treble, Vocal and Custom; every gain is clamped to
±12 dB whatever you type. There is a reset button.

**Advanced.** Audio debug logs. The audio stability mode is inert — see below.

## Settings that persist and change nothing

These are rendered, stored and reloaded, and nothing in the program reads them
back. Listing them is the honest thing to do; a toggle that appears to work is
worse than a missing one.

| Setting | Why it does nothing |
|---|---|
| **Crossfade** (0–12 s) | the value is stored and clamped; the audio path never reads it |
| **Automix** | a toggle with no consumer anywhere in the playback code |
| **Normalize volume** | copied into the audio config and never read. Toggling it does force a track reload — the config compares unequal — but nothing about the sound changes |
| **Auto-check updates**, **update channel**, **auto-install updates** | there is no updater in Grape. Updates arrive through Colony |
| **Hardware acceleration** | nothing probes the GPU and nothing selects a renderer. Availability is a per-platform constant, `true` everywhere Grape builds |
| **Audio stability mode** | Auto / Stable / Low latency is stored and never consulted |
| **Time format** | 24 h / 12 h is stored; nothing formats a time from it |
| **Interface density**, **transparency & blur** | both only feed the label in the Appearance preview card; neither changes the layout or the rendering |

## Actions that need confirming

*Reindex library*, *Clear cache* and *Reset audio engine* each ask once before
running: the first press replaces the button with a Confirm / Cancel pair.

- **Reindex library** rescans from disk, ignoring the cache.
- **Clear cache** deletes the whole `.grape_cache/` tree and rescans.
- **Reset audio engine** tears down and rebuilds the output stream.

## Last.fm enrichment

Grape can fill in an album's genre and year from Last.fm's `album.getInfo`.
It is off unless you supply your own API key.

**There is no field for the key in Preferences.** `metadata_api_key` and
`metadata_cache_ttl_hours` are read from `preferences.json` and can only be set
by editing that file while Grape is closed. With an empty key the whole path
returns immediately and no request is made.

Responses are cached under `.grape_cache/metadata/` and reused until the TTL
expires (24 hours by default, capped at one year). A 429 or 503 starts an
exponential backoff, from 30 seconds up to an hour, so a rate-limited account
stops hammering the API. The HTTP client gives up after 8 seconds, and the
caller caps each attempt at 15 seconds on top of that. It makes three attempts
in all, waiting 500 ms, then 1 s, between them.

Whatever you set by hand in the album editor outranks both the tags and
anything Last.fm returned.

## Where the files are

Grape follows the Colony layout — `<root>/Colony/Grape/`, the roots listed in
[install.md](install.md#where-grape-keeps-its-files).

| File | Root | What it holds |
|---|---|---|
| `preferences.json` | config | everything on this page |
| `playlist.json` | config | your playlists |
| `history.json` | data | play history — read by *Clear local history*, written by nothing |
| `session.json` | data | the resume point: track, position, tab, queue index |
| `logs/` | data | created so *Open logs folder* has somewhere to open |
| `exports/` | data | M3U playlists exported from the playlist view |
| `.colony-migrated` | config | the marker saying the one-time move already ran |

Logs go to stderr. Nothing writes a log file into `logs/`; run Grape from a
terminal to see its output.

## The library cache

`.grape_cache/` sits at the library root by default. The path is configurable:
relative paths resolve against the library folder, absolute paths are used as
given, and a relative path containing `..` is rejected and reset to the
default.

```
.grape_cache/
├── index.json     the signature index, and the cache format version
├── folders/       one JSON per album folder
├── tracks/        per-track signatures and tag data
├── covers/        cover images copied out of album folders and tags
└── metadata/      Last.fm responses, and your manual per-album overrides
```

A track is re-read only when its size or modification time no longer matches
the recorded signature. Entries no longer referenced by any album are dropped
at the end of a scan. Bumping the cache format version invalidates the index
wholesale.

Deleting the tree by hand is safe — everything in it can be rebuilt by
rescanning. The one thing worth knowing is that your manual genre/year
overrides live in `metadata/` and go with it.
