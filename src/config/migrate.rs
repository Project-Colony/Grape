//! Moves Grape's files onto the Colony filesystem layout.
//!
//! Grape shipped through 0.2.2 keeping everything under its config directory,
//! including things the layout puts in `data`: the play history, the logs, the
//! resume point and the playlist exports. On macOS it also read `~/.config`
//! rather than `~/Library/Application Support`, and on Linux it read `$HOME`
//! directly, so a box with `XDG_CONFIG_HOME` set had its profile somewhere the
//! new code will not look.
//!
//! Schema 1 moved each file to the right root, directly inside it. Schema 2
//! gives each one a sub-directory as well, because config and data are the
//! same directory on Windows and macOS: only a sub-directory keeps them apart
//! there. [`ENTRIES`] lists every location.
//!
//! These paths are live on users' machines, so the protocol in
//! `design/filesystem.md` applies literally:
//!
//! 1. compute the new path; if the marker says the move already happened, stop;
//! 2. where an older path exists and the new one does not, copy it across;
//! 3. a failure leaves everything where it was and does not write the marker,
//!    so the next launch tries again. Until then [`resolve`] keeps Grape on the
//!    old file rather than on an empty profile;
//! 4. the old files are left alone. Deleting them belongs to a later release,
//!    once this one has proven itself: if the move is wrong, the user's data
//!    has to still be there.
//!
//! Copying rather than renaming is deliberate: the two roots can be on
//! different filesystems, and a half-finished rename has no safe recovery.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tracing::{info, warn};

use super::Roots;

/// Bumped when a future release needs to move something else. A marker holding
/// an older number means this release has work the previous one did not do.
const SCHEMA: u32 = 2;

const MARKER: &str = ".colony-migrated";

/// Separate marker: the override lift needs the library root, which is only
/// known once settings are loaded, so it cannot run inside [`run`]. It holds
/// the key of every library already lifted, one per line.
const OVERRIDES_MARKER: &str = ".metadata-overrides-lifted";

/// One thing Grape keeps outside the cache.
#[derive(Clone, Copy, Debug)]
pub(super) struct Entry {
    in_data: bool,
    /// The schema-2 sub-directory of its root; empty for an entry that is a
    /// directory of its own.
    dir: &'static str,
    name: &'static str,
}

impl Entry {
    const fn config(dir: &'static str, name: &'static str) -> Self {
        Self { in_data: false, dir, name }
    }

    const fn data(dir: &'static str, name: &'static str) -> Self {
        Self { in_data: true, dir, name }
    }

    /// Every place this entry has lived, newest first: schema 2, schema 1
    /// (directly in its root), then the pre-Colony config root.
    pub(super) fn locations(self, roots: &Roots) -> [PathBuf; 3] {
        let root = if self.in_data { &roots.data } else { &roots.config };
        [
            root.join(self.dir).join(self.name),
            root.join(self.name),
            roots.legacy.join(self.name),
        ]
    }
}

pub(super) const PREFERENCES: Entry = Entry::config("preferences", "preferences.json");
pub(super) const PLAYLISTS: Entry = Entry::config("preferences", "playlist.json");
pub(super) const METADATA_OVERRIDES: Entry = Entry::config("preferences", "metadata-overrides");
pub(super) const HISTORY: Entry = Entry::data("history", "history.json");
pub(super) const SESSION: Entry = Entry::data("session", "session.json");
pub(super) const LOGS: Entry = Entry::data("", "logs");
pub(super) const EXPORTS: Entry = Entry::data("", "exports");

const ENTRIES: [Entry; 7] =
    [PREFERENCES, PLAYLISTS, METADATA_OVERRIDES, HISTORY, SESSION, LOGS, EXPORTS];

/// Where `entry` is read and written.
///
/// Its schema-2 path once the migration has completed. Until then, the newest
/// of its locations that exists, so a copy that failed leaves Grape on the old
/// file instead of on defaults, and writes land where the next attempt will
/// copy them from. Nothing anywhere means a fresh install: schema 2.
pub(super) fn resolve(roots: &Roots, entry: Entry) -> PathBuf {
    let [new, older @ ..] = entry.locations(roots);
    if roots.migrated || new.exists() {
        return new;
    }
    older.into_iter().find(|path| path.exists()).unwrap_or(new)
}

/// Runs once per installation, before any setting is read, and says whether
/// every entry is now at its schema-2 path.
///
/// Never returns an error: a migration that cannot complete must degrade to the
/// old location, not stop the program.
pub fn run(roots: &Roots) -> bool {
    let marker = roots.config.join(MARKER);
    if migrated_at_or_above(&marker, SCHEMA) {
        return true;
    }

    let mut moved = 0usize;
    let mut failed = false;
    for entry in ENTRIES {
        let [to, older @ ..] = entry.locations(roots);
        if to.exists() {
            continue;
        }
        // Schema 1 before the legacy root: it is the newer copy. On Windows
        // and default-XDG Linux some of these are the same directory, and
        // `logs/` and `exports/` did not move between schema 1 and 2.
        let Some(from) = older.iter().find(|from| **from != to && from.exists()) else {
            continue;
        };
        match copy_into_place(from, &to) {
            Ok(()) => {
                info!(from = %from.display(), to = %to.display(), "Migrated to the Colony layout");
                moved += 1;
            }
            Err(err) => {
                warn!(
                    error = %err,
                    from = %from.display(),
                    to = %to.display(),
                    "Could not migrate; keeping the old location for now"
                );
                failed = true;
            }
        }
    }

    if failed {
        // No marker: the next launch retries. Grape carries on with whatever
        // did land, and the originals are all still there.
        return false;
    }
    if moved > 0 {
        info!(count = moved, "Colony layout migration complete");
    }
    if let Err(err) = fs::write(&marker, SCHEMA.to_string()) {
        // Harmless on its own: the copies above are all guarded by
        // `to.exists()`, so a repeated run is a no-op rather than a clobber.
        warn!(error = %err, path = %marker.display(), "Could not write the migration marker");
    }
    true
}

/// Rescues per-album metadata the user typed by hand in `library_root`.
///
/// Those edits used to be stored inside the library cache, in the same JSON
/// object as the fetched Last.fm half, which meant "Clear cache" destroyed
/// them. They now live in the config directory. This lifts what is already on
/// disk into the new location.
///
/// It must run for every library the user opens, before anything reads that
/// library's overrides, because after the split nothing else reads the old
/// records. Each library is lifted once.
pub fn lift_metadata_overrides(roots: &Roots, library_root: &Path) {
    // The cache directory name was hardcoded, so this is where the records are
    // regardless of what `cache_path` said.
    let legacy = library_root.join(".grape_cache").join("metadata");
    if !legacy.is_dir() {
        return;
    }
    // Same namespacing the live path uses, or the lift would drop them
    // somewhere nothing reads.
    let key = crate::library::cache::library_key(library_root);
    let marker = roots.config.join(OVERRIDES_MARKER);
    let lifted_before = fs::read_to_string(&marker).unwrap_or_default();
    if lifted_before.lines().any(|line| line == key) {
        return;
    }
    let destination = resolve(roots, METADATA_OVERRIDES).join(&key);

    let entries = match fs::read_dir(&legacy) {
        Ok(entries) => entries,
        Err(err) => {
            warn!(error = %err, path = %legacy.display(), "Could not read the old metadata cache");
            return;
        }
    };
    let mut lifted = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        // Read the one field that mattered, without needing the rest of the
        // old record's shape to still parse.
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) else {
            continue;
        };
        let Some(user_override) = value.get("user_override") else {
            continue;
        };
        if user_override.is_null() {
            continue;
        }
        let Some(name) = path.file_name() else { continue };
        let target = destination.join(name);
        if target.exists() {
            continue;
        }
        if let Err(err) = fs::create_dir_all(&destination) {
            warn!(error = %err, "Could not create the metadata override directory");
            return;
        }
        match serde_json::to_string_pretty(user_override)
            .map_err(io::Error::other)
            .and_then(|payload| fs::write(&target, payload))
        {
            Ok(()) => lifted += 1,
            Err(err) => {
                warn!(error = %err, path = %target.display(), "Could not lift a metadata override");
                // No marker: retry next launch rather than orphaning the rest.
                return;
            }
        }
    }

    if lifted > 0 {
        info!(count = lifted, "Rescued hand-edited album metadata from the cache");
    }
    let mut keys: Vec<&str> = lifted_before.lines().collect();
    keys.push(&key);
    if let Err(err) = fs::write(&marker, keys.join("\n") + "\n") {
        warn!(error = %err, "Could not write the metadata override marker");
    }
}

fn migrated_at_or_above(marker: &Path, schema: u32) -> bool {
    fs::read_to_string(marker)
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .is_some_and(|found| found >= schema)
}

/// Copies a file or a directory tree, creating the parent chain first.
///
/// Writes into a temporary sibling and renames it into place, so an interrupted
/// copy cannot leave a half-written file at the destination that the guard in
/// `run` would then mistake for a completed migration.
fn copy_into_place(from: &Path, to: &Path) -> io::Result<()> {
    let parent = to
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "destination has no parent"))?;
    fs::create_dir_all(parent)?;

    let staging = parent.join(format!(
        ".{}.migrating",
        to.file_name().and_then(|n| n.to_str()).unwrap_or("entry")
    ));
    if staging.exists() {
        remove_any(&staging)?;
    }

    let result = if from.is_dir() {
        copy_dir(from, &staging)
    } else {
        fs::copy(from, &staging).map(|_| ())
    };
    if let Err(err) = result {
        let _ = remove_any(&staging);
        return Err(err);
    }

    match fs::rename(&staging, to) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = remove_any(&staging);
            Err(err)
        }
    }
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn remove_any(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else if path.exists() {
        fs::remove_file(path)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn roots_in(base: &Path) -> Roots {
        Roots {
            config: base.join("config"),
            data: base.join("data"),
            cache: base.join("cache"),
            legacy: base.join("legacy"),
            migrated: false,
        }
    }

    fn write(path: &Path, body: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
    }

    /// Where a library's lifted overrides land on a fresh schema-2 layout.
    fn bucket(roots: &Roots, library: &Path) -> PathBuf {
        roots
            .config
            .join("preferences/metadata-overrides")
            .join(crate::library::cache::library_key(library))
    }

    fn old_override(library: &Path, name: &str, genre: &str) {
        write(
            &library.join(".grape_cache/metadata").join(name),
            &format!(
                r#"{{"fetched_at":0,"metadata":{{}},"user_override":{{"genre":"{genre}",
                   "year":0,"genre_overridden":true,"year_overridden":false,"edited_at":1}}}}"#
            ),
        );
    }

    #[test]
    fn lifts_a_user_override_out_of_the_old_cache_record() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        fs::create_dir_all(&roots.config).unwrap();
        let library = tmp.path().join("Music");
        let old = library.join(".grape_cache").join("metadata");
        // The record as it was written before the split: the user's edit in the
        // same object as the fetched half.
        write(
            &old.join("abc123.json"),
            r#"{"fetched_at":17,"metadata":{"genre":"Jazz","year":1959},
                "user_override":{"genre":"Modal Jazz","year":1959,
                "genre_overridden":true,"year_overridden":false,"edited_at":42},
                "backoff_until":0,"backoff_secs":0}"#,
        );
        // One with nothing hand-typed must not produce a file.
        write(&old.join("def456.json"), r#"{"fetched_at":9,"metadata":{},"user_override":null}"#);

        lift_metadata_overrides(&roots, &library);

        let bucket = bucket(&roots, &library);
        let lifted = bucket.join("abc123.json");
        let body = fs::read_to_string(&lifted).expect("the hand-typed edit must survive");
        assert!(body.contains("Modal Jazz"));
        assert!(!body.contains("fetched_at"), "only the user's half moves");
        assert!(
            !bucket.join("def456.json").exists(),
            "a record with no override must not produce a file"
        );
        assert!(old.join("abc123.json").exists(), "the original stays for one release");
        assert!(roots.config.join(OVERRIDES_MARKER).exists());
    }

    #[test]
    fn the_lift_runs_once_and_does_not_clobber() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        fs::create_dir_all(&roots.config).unwrap();
        let library = tmp.path().join("Music");
        write(
            &library.join(".grape_cache/metadata/k.json"),
            r#"{"fetched_at":0,"metadata":{},"user_override":{"genre":"Old","year":0,
               "genre_overridden":true,"year_overridden":false,"edited_at":1}}"#,
        );
        lift_metadata_overrides(&roots, &library);
        let target = bucket(&roots, &library).join("k.json");
        fs::write(&target, r#"{"genre":"Edited since"}"#).unwrap();

        lift_metadata_overrides(&roots, &library);

        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            r#"{"genre":"Edited since"}"#,
            "a later edit must not be overwritten by a second run"
        );
    }

    #[test]
    fn copies_a_file_and_leaves_the_original() {
        let tmp = tempfile::tempdir().unwrap();
        let from = tmp.path().join("a.json");
        let to = tmp.path().join("nested/a.json");
        write(&from, "payload");

        copy_into_place(&from, &to).unwrap();

        assert_eq!(fs::read_to_string(&to).unwrap(), "payload");
        assert!(from.exists(), "the original must survive the release that moves it");
    }

    #[test]
    fn copies_a_directory_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let from = tmp.path().join("logs");
        write(&from.join("inner/deep.txt"), "line");
        let to = tmp.path().join("data/logs");

        copy_into_place(&from, &to).unwrap();

        assert_eq!(fs::read_to_string(to.join("inner/deep.txt")).unwrap(), "line");
    }

    #[test]
    fn leaves_no_staging_entry_behind_on_success() {
        let tmp = tempfile::tempdir().unwrap();
        let from = tmp.path().join("s.json");
        write(&from, "x");
        let to = tmp.path().join("out/s.json");

        copy_into_place(&from, &to).unwrap();

        let leftovers: Vec<PathBuf> = fs::read_dir(to.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.contains("migrating")))
            .collect();
        assert!(leftovers.is_empty(), "staging entries must not survive: {leftovers:?}");
    }

    #[test]
    fn marker_is_honoured_only_at_or_above_the_current_schema() {
        let tmp = tempfile::tempdir().unwrap();
        let marker = tmp.path().join(MARKER);
        assert!(!migrated_at_or_above(&marker, SCHEMA), "absent marker means work to do");
        fs::write(&marker, "0").unwrap();
        assert!(!migrated_at_or_above(&marker, SCHEMA), "an older schema must re-run");
        fs::write(&marker, SCHEMA.to_string()).unwrap();
        assert!(migrated_at_or_above(&marker, SCHEMA));
        fs::write(&marker, "99").unwrap();
        assert!(migrated_at_or_above(&marker, SCHEMA), "a newer schema has already done this work");
        fs::write(&marker, "not a number").unwrap();
        assert!(!migrated_at_or_above(&marker, SCHEMA), "an unreadable marker must not skip the move");
    }

    #[test]
    fn a_fresh_install_starts_on_schema_2() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        fs::create_dir_all(&roots.config).unwrap();

        assert!(run(&roots));

        assert_eq!(read(&roots.config.join(MARKER)), SCHEMA.to_string());
        assert_eq!(resolve(&roots, PREFERENCES), roots.config.join("preferences/preferences.json"));
        assert_eq!(resolve(&roots, HISTORY), roots.data.join("history/history.json"));
    }

    #[test]
    fn schema_1_files_are_copied_into_their_sub_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        write(&roots.config.join(MARKER), "1");
        write(&roots.config.join("preferences.json"), "prefs");
        write(&roots.config.join("playlist.json"), "lists");
        write(&roots.config.join("metadata-overrides/k/a.json"), "edit");
        write(&roots.data.join("history.json"), "history");
        write(&roots.data.join("session.json"), "session");
        write(&roots.data.join("logs/grape.log"), "log");
        // An older legacy copy must lose to the schema-1 one.
        write(&roots.legacy.join("preferences.json"), "stale");

        assert!(run(&roots));

        assert_eq!(read(&roots.config.join("preferences/preferences.json")), "prefs");
        assert_eq!(read(&roots.config.join("preferences/playlist.json")), "lists");
        assert_eq!(read(&roots.config.join("preferences/metadata-overrides/k/a.json")), "edit");
        assert_eq!(read(&roots.data.join("history/history.json")), "history");
        assert_eq!(read(&roots.data.join("session/session.json")), "session");
        assert_eq!(read(&roots.data.join("logs/grape.log")), "log");
        assert_eq!(read(&roots.config.join("preferences.json")), "prefs", "originals stay");
        assert_eq!(read(&roots.config.join(MARKER)), SCHEMA.to_string());
    }

    #[test]
    fn legacy_files_are_copied_straight_to_schema_2() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        fs::create_dir_all(&roots.config).unwrap();
        write(&roots.legacy.join("preferences.json"), "prefs");
        write(&roots.legacy.join("history.json"), "history");
        write(&roots.legacy.join("exports/mix.m3u"), "#EXTM3U");

        assert!(run(&roots));

        assert_eq!(read(&roots.config.join("preferences/preferences.json")), "prefs");
        assert_eq!(read(&roots.data.join("history/history.json")), "history");
        assert_eq!(read(&roots.data.join("exports/mix.m3u")), "#EXTM3U");
        assert!(roots.legacy.join("preferences.json").exists(), "originals stay");
    }

    #[test]
    fn a_failed_copy_keeps_using_the_old_file() {
        let tmp = tempfile::tempdir().unwrap();
        let mut roots = roots_in(tmp.path());
        write(&roots.config.join("preferences.json"), "prefs");
        write(&roots.legacy.join("history.json"), "history");
        // A file where the sub-directory has to go: every copy into it fails.
        write(&roots.config.join("preferences"), "in the way");

        roots.migrated = run(&roots);

        assert!(!roots.migrated);
        assert!(!roots.config.join(MARKER).exists(), "no marker, so the next launch retries");
        assert_eq!(resolve(&roots, PREFERENCES), roots.config.join("preferences.json"));
        // An entry that did copy is used from its new place.
        assert_eq!(resolve(&roots, HISTORY), roots.data.join("history/history.json"));
        // Nothing anywhere: written straight to the new place.
        assert_eq!(resolve(&roots, SESSION), roots.data.join("session/session.json"));

        fs::remove_file(roots.config.join("preferences")).unwrap();
        roots.migrated = run(&roots);

        assert!(roots.migrated, "the retry completes once the obstacle is gone");
        assert_eq!(read(&resolve(&roots, PREFERENCES)), "prefs");
    }

    #[test]
    fn resolve_falls_back_to_the_legacy_root_last() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        write(&roots.legacy.join("playlist.json"), "lists");

        assert_eq!(resolve(&roots, PLAYLISTS), roots.legacy.join("playlist.json"));
    }

    #[test]
    fn config_and_data_do_not_collide_in_one_root() {
        // Windows and macOS: config and data are the same directory.
        let tmp = tempfile::tempdir().unwrap();
        let mut roots = roots_in(tmp.path());
        roots.data = roots.config.clone();
        roots.migrated = true;

        let mut paths: Vec<PathBuf> = ENTRIES.iter().map(|entry| resolve(&roots, *entry)).collect();
        paths.push(roots.config.join(MARKER));
        paths.push(roots.config.join(OVERRIDES_MARKER));
        for (i, a) in paths.iter().enumerate() {
            for b in &paths[i + 1..] {
                assert!(
                    !a.starts_with(b) && !b.starts_with(a),
                    "{} and {} collide",
                    a.display(),
                    b.display()
                );
            }
        }
    }

    #[test]
    fn every_library_gets_its_overrides_lifted() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        // Written by the previous release, which lifted one library only.
        write(&roots.config.join(OVERRIDES_MARKER), "1");
        let first = tmp.path().join("Music");
        let second = tmp.path().join("Other Music");
        old_override(&first, "a.json", "Jazz");
        old_override(&second, "b.json", "Folk");

        lift_metadata_overrides(&roots, &first);
        lift_metadata_overrides(&roots, &second);

        assert!(read(&bucket(&roots, &first).join("a.json")).contains("Jazz"));
        assert!(read(&bucket(&roots, &second).join("b.json")).contains("Folk"));
        let marker = read(&roots.config.join(OVERRIDES_MARKER));
        for library in [&first, &second] {
            let key = crate::library::cache::library_key(library);
            assert!(marker.lines().any(|line| line == key), "{key} missing from {marker:?}");
        }
    }

    #[test]
    fn a_library_without_an_old_cache_leaves_no_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        fs::create_dir_all(&roots.config).unwrap();

        // What a half-typed library path in the preferences looks like.
        lift_metadata_overrides(&roots, &tmp.path().join("Mu"));

        assert!(!roots.config.join(OVERRIDES_MARKER).exists());
    }
}
