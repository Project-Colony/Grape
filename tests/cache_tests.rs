use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// Helper to create a test file with specific content
fn create_test_file(dir: &TempDir, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).expect("Failed to create test file");
    file.write_all(content).expect("Failed to write test file");
    path
}

// Helper to create a minimal valid JSON cache index
fn create_test_cache_index(root: &std::path::Path) -> std::io::Result<()> {
    let cache_dir = root.join(".grape_cache");
    fs::create_dir_all(&cache_dir)?;

    let index = serde_json::json!({
        "version": 5,
        "tracks": {}
    });

    let index_path = cache_dir.join("index.json");
    let content = serde_json::to_string_pretty(&index)?;
    fs::write(index_path, content)?;

    Ok(())
}

#[test]
fn test_cache_index_default() {
    // Test that default cache index can be created
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // Try to load from non-existent directory
    let result = grape::library::cache::load_index(root);
    assert!(result.is_ok(), "Should succeed even without cache");

    let index = result.unwrap();
    assert_eq!(index.track_entries().len(), 0, "Should have no entries");
}

#[test]
fn test_cache_index_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // Create a valid cache index
    create_test_cache_index(root).expect("Failed to create cache index");

    // Load it
    let result = grape::library::cache::load_index(root);
    assert!(result.is_ok(), "Should load valid cache index");

    let index = result.unwrap();
    assert_eq!(index.track_entries().len(), 0, "Should have no tracks");
}

#[test]
fn test_track_signature_generation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = create_test_file(&temp_dir, "test.mp3", b"fake mp3 content");

    let signature = grape::library::cache::track_signature(&file_path);
    assert!(
        signature.is_ok(),
        "Should generate signature for existing file"
    );

    let sig = signature.unwrap();
    assert!(sig.modified_secs > 0, "Should have modification time");
    assert!(sig.hash > 0, "Should have hash value");
}

#[test]
fn test_track_signature_nonexistent_file() {
    let nonexistent = PathBuf::from("/tmp/nonexistent_grape_test_file.mp3");

    let signature = grape::library::cache::track_signature(&nonexistent);
    assert!(
        signature.is_err(),
        "Should fail for nonexistent file"
    );
}

#[test]
fn test_track_signature_consistency() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = create_test_file(&temp_dir, "test.mp3", b"consistent content");

    let sig1 = grape::library::cache::track_signature(&file_path)
        .expect("Failed to get signature 1");
    let sig2 = grape::library::cache::track_signature(&file_path)
        .expect("Failed to get signature 2");

    assert_eq!(
        sig1.modified_secs, sig2.modified_secs,
        "Signatures should be consistent"
    );
    assert_eq!(sig1.hash, sig2.hash, "Hashes should be consistent");
    assert_eq!(
        sig1.file_len, sig2.file_len,
        "File lengths should be consistent"
    );
}

#[test]
fn test_track_signature_changes_with_modification() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = create_test_file(&temp_dir, "test.mp3", b"original content");

    let sig1 = grape::library::cache::track_signature(&file_path)
        .expect("Failed to get original signature");

    // Wait a bit to ensure modification time changes
    thread::sleep(Duration::from_millis(100));

    // Modify the file
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(&file_path)
        .expect("Failed to open file for modification");
    file.write_all(b" modified")
        .expect("Failed to modify file");
    drop(file);

    let sig2 = grape::library::cache::track_signature(&file_path)
        .expect("Failed to get modified signature");

    // Signature should be different after modification
    assert_ne!(
        sig1.hash, sig2.hash,
        "Hash should change after file modification"
    );
    assert_ne!(
        sig1.file_len, sig2.file_len,
        "File length should change after modification"
    );
}

#[test]
fn test_track_key_generation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();
    let track_path = root.join("Artist/Album/track.mp3");

    let key = grape::library::cache::track_key(root, &track_path);

    // Key should be relative path with forward slashes
    assert!(key.contains("Artist"));
    assert!(key.contains("Album"));
    assert!(key.contains("track.mp3"));
    assert!(!key.contains('\\'), "Should use forward slashes");
}

#[test]
fn test_track_id_generation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();
    let track_path = root.join("Artist/Album/track.mp3");

    let id = grape::library::cache::track_id(root, &track_path);

    // ID should be a hex hash
    assert!(!id.is_empty(), "ID should not be empty");
    assert!(
        id.chars().all(|c: char| c.is_ascii_hexdigit()),
        "ID should be hexadecimal"
    );
}

#[test]
fn test_track_id_consistency() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();
    let track_path = root.join("Artist/Album/track.mp3");

    let id1 = grape::library::cache::track_id(root, &track_path);
    let id2 = grape::library::cache::track_id(root, &track_path);

    assert_eq!(id1, id2, "Same path should produce same ID");
}

#[test]
fn test_track_id_different_for_different_paths() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();
    let track1 = root.join("Artist1/Album/track.mp3");
    let track2 = root.join("Artist2/Album/track.mp3");

    let id1 = grape::library::cache::track_id(root, &track1);
    let id2 = grape::library::cache::track_id(root, &track2);

    assert_ne!(
        id1, id2,
        "Different paths should produce different IDs"
    );
}

#[test]
fn test_ensure_cover_cache_dir() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let result = grape::library::cache::ensure_cover_cache_dir(root);
    assert!(result.is_ok(), "Should create cover cache dir");

    let cover_dir = result.unwrap();
    assert!(
        cover_dir.exists(),
        "Cover cache directory should exist after creation"
    );
    assert!(
        cover_dir.is_dir(),
        "Cover cache path should be a directory"
    );
}

#[test]
fn test_ensure_metadata_cache_dir() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let result = grape::library::cache::ensure_metadata_cache_dir(root);
    assert!(result.is_ok(), "Should create metadata cache dir");

    let metadata_dir = result.unwrap();
    assert!(
        metadata_dir.exists(),
        "Metadata cache directory should exist after creation"
    );
    assert!(
        metadata_dir.is_dir(),
        "Metadata cache path should be a directory"
    );
}

#[test]
fn test_cache_directory_structure() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // Create both cache directories
    grape::library::cache::ensure_cover_cache_dir(root)
        .expect("Failed to create cover cache dir");
    grape::library::cache::ensure_metadata_cache_dir(root)
        .expect("Failed to create metadata cache dir");

    // Verify structure
    let cache_root = root.join(".grape_cache");
    assert!(cache_root.exists(), "Cache root should exist");
    assert!(
        cache_root.join("covers").exists(),
        "Covers directory should exist"
    );
    assert!(
        cache_root.join("metadata").exists(),
        "Metadata directory should exist"
    );
}

#[test]
fn test_cache_index_with_tracks() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();
    let cache_dir = root.join(".grape_cache");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");

    // Create index with sample track entry
    let index = serde_json::json!({
        "version": 5,
        "tracks": {
            "Artist/Album/track1.mp3": {
                "id": "abc123",
                "modified_secs": 1234567890,
                "hash": 9876543210u64,
                "file_len": 5000000
            }
        }
    });

    let index_path = cache_dir.join("index.json");
    fs::write(
        index_path,
        serde_json::to_string_pretty(&index).expect("Failed to serialize"),
    )
    .expect("Failed to write index");

    // Load and verify
    let loaded = grape::library::cache::load_index(root).expect("Failed to load index");
    assert_eq!(
        loaded.track_entries().len(),
        1,
        "Should have one track entry"
    );

    let entry = loaded
        .track_entries()
        .get("Artist/Album/track1.mp3")
        .expect("Should have the track entry");
    assert_eq!(entry.id(), "abc123");
}

#[test]
fn test_load_album_nonexistent() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();
    let album_path = root.join("Artist/Album");

    let result = grape::library::cache::load_album(root, &album_path);
    assert!(result.is_ok(), "Should succeed even if album not cached");

    let cached = result.unwrap();
    assert!(cached.is_none(), "Should return None for non-cached album");
}

#[test]
fn test_finalize_cache_cleanup() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // Create initial cache index with tracks
    let mut index = grape::library::cache::load_index(root).expect("Failed to load index");

    // No tracks or folders used
    let used_keys = HashSet::new();
    let used_track_keys = HashSet::new();
    let used_track_ids = HashSet::new();
    let used_metadata_keys = HashSet::new();
    let used_cover_filenames = HashSet::new();

    // Finalize should clean up and save
    let result = grape::library::cache::finalize(
        root,
        &mut index,
        &used_keys,
        &used_track_keys,
        &used_track_ids,
        &used_metadata_keys,
        &used_cover_filenames,
    );
    assert!(result.is_ok(), "Finalize should succeed");

    // Index file should exist
    let index_path = root.join(".grape_cache/index.json");
    assert!(index_path.exists(), "Index file should be created");
}

#[test]
fn test_track_entry_signature_matching() {
    use grape::library::cache::{TrackEntry, TrackSignature};

    let entry = TrackEntry {
        id: "test123".to_string(),
        modified_secs: 1234567890,
        hash: 9876543210,
        file_len: Some(5000000),
    };

    // Matching signature
    let matching_sig = TrackSignature {
        modified_secs: 1234567890,
        hash: 9876543210,
        file_len: Some(5000000),
    };
    assert!(
        entry.matches_signature(&matching_sig),
        "Should match identical signature"
    );

    // Different modification time
    let diff_mod_sig = TrackSignature {
        modified_secs: 9999999999,
        hash: 9876543210,
        file_len: Some(5000000),
    };
    assert!(
        !entry.matches_signature(&diff_mod_sig),
        "Should not match different modification time"
    );

    // Different hash
    let diff_hash_sig = TrackSignature {
        modified_secs: 1234567890,
        hash: 1111111111,
        file_len: Some(5000000),
    };
    assert!(
        !entry.matches_signature(&diff_hash_sig),
        "Should not match different hash"
    );

    // Different file length
    let diff_len_sig = TrackSignature {
        modified_secs: 1234567890,
        hash: 9876543210,
        file_len: Some(9999999),
    };
    assert!(
        !entry.matches_signature(&diff_len_sig),
        "Should not match different file length"
    );
}

#[test]
fn test_track_entry_signature_matching_without_file_len() {
    use grape::library::cache::{TrackEntry, TrackSignature};

    // Entry without file length (legacy)
    let entry = TrackEntry {
        id: "test123".to_string(),
        modified_secs: 1234567890,
        hash: 9876543210,
        file_len: None,
    };

    // Signature with file length should still match if other fields match
    let sig_with_len = TrackSignature {
        modified_secs: 1234567890,
        hash: 9876543210,
        file_len: Some(5000000),
    };
    assert!(
        entry.matches_signature(&sig_with_len),
        "Should match even with missing file_len field"
    );
}

#[cfg(test)]
mod legacy_cache_tests {
    use super::*;

    #[test]
    fn test_load_legacy_cache_format() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();
        let cache_dir = root.join(".grape_cache");
        fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");

        // Create legacy format index (version with "entries" field)
        let legacy_index = serde_json::json!({
            "version": 4,
            "entries": {
                "folder1": {
                    "tracks": {
                        "Artist/Album/track1.mp3": {
                            "modified_secs": 1234567890,
                            "hash": 9876543210u64,
                        }
                    }
                }
            }
        });

        let index_path = cache_dir.join("index.json");
        fs::write(
            index_path,
            serde_json::to_string_pretty(&legacy_index).expect("Failed to serialize"),
        )
        .expect("Failed to write legacy index");

        // Load should migrate to new format
        let loaded =
            grape::library::cache::load_index(root).expect("Failed to load legacy index");

        // Should have migrated the tracks
        assert!(
            !loaded.track_entries().is_empty(),
            "Should have migrated tracks from legacy format"
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_cache_workflow() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();

        // 1. Create cache directories
        grape::library::cache::ensure_cover_cache_dir(root)
            .expect("Failed to create cover cache");
        grape::library::cache::ensure_metadata_cache_dir(root)
            .expect("Failed to create metadata cache");

        // 2. Load initial index
        let mut index =
            grape::library::cache::load_index(root).expect("Failed to load initial index");
        assert_eq!(index.track_entries().len(), 0);

        // 3. Create a test file and generate signature
        let file_path = create_test_file(&temp_dir, "test_track.mp3", b"test audio data");
        let _signature = grape::library::cache::track_signature(&file_path)
            .expect("Failed to generate signature");

        // 4. Finalize (should save the index)
        let used_keys = HashSet::new();
        let used_track_keys = HashSet::new();
        let used_track_ids = HashSet::new();
        let used_metadata_keys = HashSet::new();
        let used_cover_filenames = HashSet::new();
        grape::library::cache::finalize(
            root,
            &mut index,
            &used_keys,
            &used_track_keys,
            &used_track_ids,
            &used_metadata_keys,
            &used_cover_filenames,
        )
        .expect("Failed to finalize");

        // 5. Reload index and verify it was saved
        let reloaded =
            grape::library::cache::load_index(root).expect("Failed to reload index");
        assert_eq!(
            reloaded.track_entries().len(),
            index.track_entries().len(),
            "Reloaded index should match saved index"
        );
    }
}
