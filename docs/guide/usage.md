# Using Grape

## The window

```
┌──────────────────────────────────────────────────────────────┐
│ Grape ▾   Artists  Genres  Albums  Folders   [search] ≡ — ▫ ✕ │  top bar
├──────────────┬───────────────────────────┬───────────────────┤
│ Artists      │ Albums                    │ Tracks            │  library
│              │                           │                   │
├──────────────┴───────────────────────────┴───────────────────┤
│ ◀ ▶  cover · title · artist   ──●────────  vol  speed  queue │  player bar
└──────────────────────────────────────────────────────────────┘
```

The top bar carries the logo (which opens the menu), the four tabs, the search
field, and the window controls — Grape draws its own title bar.

Below it, the library is three columns on the Artists, Genres and Albums tabs
and two on Folders. The proportions are fixed: 2 / 5 / 3 for the three-column
layouts, 7 / 3 for Folders. Selecting in a left column narrows the columns to
its right.

The player bar is always at the bottom, in every view except the full-screen
Queue and Playlist views.

## The four tabs

| Tab | Left column | Middle | Right |
|---|---|---|---|
| **Artists** | every artist | that artist's albums | the selected album's tracks |
| **Genres** | every genre found in tags | albums in that genre | tracks |
| **Albums** | artists | every album | tracks |
| **Folders** | the folder tree as it is on disk | — | tracks in the selected folder |

Genres come from the tags. An album whose tracks carry no genre tag is filed
under the "unknown" genre rather than dropped.

## Search

Typing in the search field narrows the lists as you type. Matching is
accent-insensitive: the query and the text are both NFKD-normalized first, so
`beyonce` finds *Beyoncé*.

By default a query is matched against **titles and artist names**. The four
toggles in the logo menu — Genre, Year, Duration, Codec — each add one more
field to the match. They widen the search, they do not narrow it: with Duration
on, `3:45` also finds anything of that length, and everything that matched
before still matches.

Duration accepts several spellings of the same thing: total seconds, total
minutes, `mm:ss`, or `h:mm:ss`.

Sorting is not exposed. The panel headers show what the ordering is (`A–Z`,
`By name`), but there is no control to change it and no preference for it —
albums and tracks are ordered by album, then by track number.

## Playing

Click a track to play it. The player bar handles the rest: play/pause,
previous, next, shuffle, repeat (off / one / all), a click-anywhere progress
bar, volume, and a playback-speed popup from 0.5× to 2.0×.

When gapless playback is on — it is, by default — Grape appends the next queued
track to the audio sink once the current one is within 500 ms of ending, so the
two run together with no gap.

## The queue

Open the queue from the logo menu. It is a full-screen split: what is playing
on the left, what is coming on the right.

Each queued row has move-up, move-down and remove buttons. There is no pointer
dragging in the queue.

`Escape` closes it.

## Playlists

Playlists are created, renamed and deleted from the Playlist view, and stored
as JSON in the Colony config directory (`playlist.json`). They survive
restarts.

Reordering works by two clicks rather than a drag: press the `⠿` handle on the
row you want to move, then press the `⤵` button that appears on the row you
want to move it to. Pressing the handle again cancels.

## The mini player

Press `m`, or the mini-player button on the player bar, to collapse the window
down to the player bar alone — no library, no tabs. `Escape` or `m` again
brings the full window back.

## Keyboard

In the main window, when nothing has taken the keystroke first:

| Key | What it does |
|---|---|
| `↑` `↓` | move within the focused column |
| `←` `→` | move between entries |
| `Tab` / `Shift`+`Tab` | next / previous column |
| `Enter` | activate the selection |
| `Space` | play / pause |
| `n` | next track |
| `p` | previous track |
| `+` or `=` | volume up |
| `-` | volume down |
| `m` | toggle the mini player |
| `Escape` | close the open menu, queue, playlist, preferences or mini player |

These are fixed; there is no key-rebinding UI.

### Global hotkeys

Three shortcuts work while another application has focus, once **Advanced
shortcuts** is enabled in Preferences:

| Key | What it does |
|---|---|
| `Ctrl`+`Alt`+`P` | play / pause |
| `Ctrl`+`Alt`+`→` | next track |
| `Ctrl`+`Alt`+`←` | previous track |

They are opt-in and turn themselves back off if the platform reports them
unavailable. See
[troubleshooting.md](troubleshooting.md#the-global-hotkeys-do-nothing).

## Editing album genre and year

The track list has an inline editor for the selected album's genre and year.
What you set there is stored per album in the cache and outranks both the file
tags and anything Last.fm returned — it is the last word, and it survives a
rescan.
