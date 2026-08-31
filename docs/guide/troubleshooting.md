# Troubleshooting

Grape logs to stderr, not to a file. Running it from a terminal is the fastest
way to see what it thinks is happening.

## Some files appear in the library but will not play

The scanner and the decoder do not agree on the format list.

The scanner accepts `mp3`, `flac`, `wav`, `ogg`, `m4a`, `aac`, `opus`, `aif`,
`aiff` and `wma`. The decoder is symphonia, as pulled in by rodio's default
features, which covers FLAC, MP3, AAC in MP4, Vorbis in Ogg, and PCM WAV.

So **Opus, AIFF and WMA files are indexed and shown, and fail when you press
play.** There is no decoder compiled in for any of them. This is not a
system-codec question — nothing is loaded from the system, so installing codecs
on your machine changes nothing. Converting those files to FLAC is the
practical fix.

## There is no sound at all

Check the output device in Preferences → Audio. *USB headset* is a guess, not a
picker: it takes the first device whose interface type is USB, or whose type is
headset or headphones, or whose name mentions "usb" or "headset". If that
matches the wrong device — or none — switch back to *System (default)*.

If the sample rate is set to something the device does not accept, Grape opens
the stream at whatever the device offers and shows a notice in the window.
Clearing the sample rate setting removes the guesswork.

*Reset audio engine* in Preferences → Audio rebuilds the output stream without
restarting Grape. Use it after plugging or unplugging an interface.

If crossfade or automix are on and you cannot hear them: they do nothing.
Neither is implemented; see
[configuration.md](configuration.md#settings-that-persist-and-change-nothing).

## Every genre says "Unknown"

Genres come from the tags in the files, and nowhere else by default. Files with
no genre tag are grouped under the "unknown" genre rather than hidden.

Two ways out: edit the genre for an album inline in the track list, which is
stored per album and beats everything else, or set a Last.fm API key so albums
can be enriched from `album.getInfo` (see below).

## Last.fm enrichment returns nothing

In order of likelihood:

- **No API key.** There is no field for it in Preferences. `metadata_api_key`
  has to be written into `preferences.json` by hand, with Grape closed. With an
  empty key the code returns before making any request.
- **A backoff window is open.** A 429 or 503 from Last.fm starts an exponential
  backoff — 30 seconds, doubling, up to an hour — during which nothing is
  requested.
- **A cached miss.** Responses are cached under `.grape_cache/metadata/` for
  the TTL, an empty result included. Force a refresh from the album editor, or
  clear the cache.
- **Last.fm has no entry** for that artist/album spelling.

## The album cover is not the one I put there

The order is: an image file in the album folder named `cover`, `folder`,
`front`, `artwork` or `album` (`.jpg`, `.jpeg`, `.png`, `.webp`), then the
picture embedded in the tags, then a cover cached by an earlier scan.

External files win. If a stale image keeps coming back, it is the cached copy
under `.grape_cache/covers/`; *Clear cache* removes it.

Embedded pictures over 10 MB are skipped and logged.

## The library did not pick up my new album

Grape re-reads a track only when its size or modification time has changed
since the last scan. A file that was moved with its timestamps preserved looks
unchanged.

*Reindex library* rescans ignoring the cache; *Clear cache* deletes the whole
`.grape_cache/` tree and rescans from nothing.

## No tray icon

The tray is off by default. Once enabled, and if it still does not appear:

- **Linux** needs something implementing the StatusNotifierItem protocol —
  a GNOME extension such as AppIndicator support, or KDE's system tray, which
  has it built in. A plain GNOME session with no extension has no tray for
  anything to appear in. Grape speaks StatusNotifierItem through `ksni`.
- **Windows** may be hiding it in the notification-area overflow; the icon is
  there, the setting is Windows's.
- **macOS** puts it in the menu bar.

The menu has one item, Quit, on every platform.

When Grape cannot create the tray it turns the preference back off and saves,
so an unchecked box after a restart means the platform refused.

## The global hotkeys do nothing

They are off until *Advanced shortcuts* is enabled. After that:

- **Wayland** compositors generally do not let an application grab a global
  key. Depending on the compositor, this either fails outright or silently
  never fires.
- **macOS** needs Grape to hold accessibility permission.
- **Windows** will not deliver a hotkey another application has already
  claimed; `Ctrl`+`Alt`+`P` is a popular choice.

The bindings are fixed at `Ctrl`+`Alt`+`P` / `→` / `←` and cannot be changed.

## No "Now Playing" notification

It is behind two switches, both off by default: *Notifications* and *Now
Playing notifications*. Both must be on.

## Launch at startup did not stick

Grape writes the autostart entry itself:

| Platform | Where the entry goes |
|---|---|
| Linux | `$XDG_CONFIG_HOME/autostart/grape.desktop` (`~/.config/autostart/` by default) |
| macOS | `~/Library/LaunchAgents/com.colony.grape.plist`, loaded with `launchctl` |
| Windows | a value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |

All three need the executable to still be at the path recorded when the setting
was enabled. Moving or reinstalling the binary breaks the entry; toggle the
setting off and on to rewrite it.

## My preferences disappeared after an update

They probably moved. Grape now uses the Colony filesystem layout, and the first
launch after that change copies the old profile across from
`~/.config/Colony/Grape` (`%LOCALAPPDATA%\Colony\Grape` on Windows). The
originals are deliberately left in place, so nothing is lost either way — see
[install.md](install.md#upgrading-from-a-pre-colony-install).

If the copy failed, no `.colony-migrated` marker is written and the next launch
tries again.
