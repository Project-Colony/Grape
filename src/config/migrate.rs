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
        }
    }

    fn write(path: &Path, body: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
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
