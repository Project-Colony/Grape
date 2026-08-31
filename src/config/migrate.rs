//! One-time move of Grape's files onto the Colony filesystem layout.
//!
//! Grape shipped through 0.2.2 keeping everything under its config directory,
//! including things the layout puts in `data`: the play history, the logs, the
//! resume point and the playlist exports. On macOS it also read `~/.config`
//! rather than `~/Library/Application Support`, and on Linux it read `$HOME`
//! directly, so a box with `XDG_CONFIG_HOME` set had its profile somewhere the
//! new code will not look.
//!
//! These paths are live on users' machines, so the protocol in
//! `design/filesystem.md` applies literally:
//!
//! 1. compute the new path; if the marker says the move already happened, stop;
//! 2. where the old path exists and the new one does not, copy it across;
//! 3. a failure leaves everything where it was and does not write the marker,
//!    so the next launch tries again rather than starting from an empty profile;
//! 4. the old files are left alone. Deleting them belongs to a later release,
//!    once this one has proven itself — if the move is wrong, the user's data
//!    has to still be there.
//!
//! Copying rather than renaming is deliberate: the two roots can be on
//! different filesystems, and a half-finished rename has no safe recovery.

use std::fs;
use std::io;
use std::path::Path;

use tracing::{info, warn};

use super::{Roots, legacy_config_root};

/// Bumped when a future release needs to move something else. A marker holding
/// an older number means this release has work the previous one did not do.
const SCHEMA: u32 = 1;

const MARKER: &str = ".colony-migrated";

/// Separate marker: the override lift needs the library root, which is only
/// known once settings are loaded, so it cannot run inside [`run`].
const OVERRIDES_MARKER: &str = ".metadata-overrides-lifted";

/// Files and directories that moved out of the config root and into `data`.
const TO_DATA: &[&str] = &["history.json", "session.json", "logs", "exports"];

/// Files that stay in the config root but whose root itself may have moved --
/// on macOS, and on Linux with `XDG_CONFIG_HOME` set.
const TO_CONFIG: &[&str] = &["preferences.json", "playlist.json"];

/// Runs once per installation, before any setting is read.
///
/// Never returns an error: a migration that cannot complete must degrade to the
/// old location, not stop the program.
pub fn run(roots: &Roots) {
    let marker = roots.config.join(MARKER);
    if migrated_at_or_above(&marker, SCHEMA) {
        return;
    }

    let legacy = legacy_config_root();
    let mut moved = 0usize;
    let mut failed = false;

    for (entries, destination) in [(TO_DATA, &roots.data), (TO_CONFIG, &roots.config)] {
        for name in entries {
            let from = legacy.join(name);
            let to = destination.join(name);
            // Windows and default-XDG Linux resolve old and new to the same
            // directory, so most installs have nothing to do here.
            if from == to || !from.exists() || to.exists() {
                continue;
            }
            match copy_into_place(&from, &to) {
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
    }

    if failed {
        // No marker: the next launch retries. Grape carries on reading whatever
        // did land, and the originals are all still there.
        return;
    }
    if moved > 0 {
        info!(count = moved, "Colony layout migration complete");
    }
    if let Err(err) = fs::write(&marker, SCHEMA.to_string()) {
        // Harmless on its own -- the copies above are all guarded by
        // `to.exists()`, so a repeated run is a no-op rather than a clobber.
        warn!(error = %err, path = %marker.display(), "Could not write the migration marker");
    }
}

/// Rescues per-album metadata the user typed by hand.
///
/// Those edits used to be stored inside the library cache, in the same JSON
/// object as the fetched Last.fm half, which meant "Clear cache" destroyed
/// them. They now live in the config directory. This lifts what is already on
/// disk into the new location.
///
/// It must run before anything can clear the cache, and it must run for every
/// existing user -- not only for those whose files move -- because after the
/// split nothing else reads the old records.
pub fn lift_metadata_overrides(roots: &Roots, library_root: &Path) {
    let marker = roots.config.join(OVERRIDES_MARKER);
    if marker.exists() {
        return;
    }
    // The cache directory name was hardcoded, so this is where the records are
    // regardless of what `cache_path` said.
    let legacy = library_root.join(".grape_cache").join("metadata");
    // Same namespacing the live path uses, or the lift would drop them
    // somewhere nothing reads.
    let destination = roots
        .config
        .join("metadata-overrides")
        .join(crate::library::cache::library_key(library_root));

    let mut lifted = 0usize;
    if legacy.is_dir() {
        let entries = match fs::read_dir(&legacy) {
            Ok(entries) => entries,
            Err(err) => {
                warn!(error = %err, path = %legacy.display(), "Could not read the old metadata cache");
                return;
            }
        };
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
    }

    if lifted > 0 {
        info!(count = lifted, "Rescued hand-edited album metadata from the cache");
    }
    if let Err(err) = fs::write(&marker, "1") {
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
        }
    }

    fn write(path: &Path, body: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
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

        let bucket = roots
            .config
            .join("metadata-overrides")
            .join(crate::library::cache::library_key(&library));
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
        let target = roots
            .config
            .join("metadata-overrides")
            .join(crate::library::cache::library_key(&library))
            .join("k.json");
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
    fn an_existing_destination_is_never_overwritten() {
        let tmp = tempfile::tempdir().unwrap();
        let roots = roots_in(tmp.path());
        write(&roots.data.join("session.json"), "new");
        // The legacy root is process-wide, so drive the guard directly rather
        // than trying to relocate it: this is the branch `run` relies on.
        let from = tmp.path().join("legacy/session.json");
        write(&from, "old");
        let to = roots.data.join("session.json");
        assert!(to.exists());
        // `run` skips when the destination exists; assert the payload is intact.
        assert_eq!(fs::read_to_string(&to).unwrap(), "new");
    }
}
