use std::thread;
use std::time::Duration;

// Note: We don't test the actual Last.fm API calls as they require a real API key
// and network access. Instead, we test the parsing, caching, and data structures.

#[test]
fn test_online_metadata_creation() {
    use grape::library::metadata::online::OnlineMetadata;

    let metadata = OnlineMetadata {
        genre: Some("Rock".to_string()),
        year: Some(2020),
    };

    assert_eq!(metadata.genre, Some("Rock".to_string()));
    assert_eq!(metadata.year, Some(2020));
}

#[test]
fn test_online_metadata_default() {
    use grape::library::metadata::online::OnlineMetadata;

    let metadata = OnlineMetadata::default();

    assert_eq!(metadata.genre, None);
    assert_eq!(metadata.year, None);
}

#[test]
fn test_online_metadata_equality() {
    use grape::library::metadata::online::OnlineMetadata;

    let m1 = OnlineMetadata {
        genre: Some("Rock".to_string()),
        year: Some(2020),
    };

    let m2 = OnlineMetadata {
        genre: Some("Rock".to_string()),
        year: Some(2020),
    };

    let m3 = OnlineMetadata {
        genre: Some("Jazz".to_string()),
        year: Some(2020),
    };

    assert_eq!(m1, m2);
    assert_ne!(m1, m3);
}

#[test]
fn test_online_metadata_clone() {
    use grape::library::metadata::online::OnlineMetadata;

    let metadata = OnlineMetadata {
        genre: Some("Electronic".to_string()),
        year: Some(2023),
    };

    let cloned = metadata.clone();
    assert_eq!(metadata, cloned);
}

#[test]
fn test_user_metadata_override_default() {
    use grape::library::metadata::online::UserMetadataOverride;

    let override_data = UserMetadataOverride::default();

    assert_eq!(override_data.genre, None);
    assert_eq!(override_data.year, None);
    assert!(!override_data.genre_overridden);
    assert!(!override_data.year_overridden);
    assert_eq!(override_data.edited_at, 0);
}

#[test]
fn test_load_user_metadata_override_nonexistent() {
    use grape::library::metadata::online::load_user_metadata_override;

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let result = load_user_metadata_override(root, "Test Artist", "Test Album");
    assert!(result.is_ok(), "Should succeed even if no override exists");

    let override_data = result.unwrap();
    assert!(
        override_data.is_none(),
        "Should return None when no override exists"
    );
}

#[test]
fn test_store_and_load_user_metadata_override() {
    use grape::library::metadata::online::{
        load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
    };

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let override_data = UserMetadataOverride {
        genre: Some("Custom Genre".to_string()),
        year: Some(2021),
        genre_overridden: true,
        year_overridden: true,
        edited_at: 0, // Will be set by store function
    };

    // Store the override
    let store_result =
        store_user_metadata_override(root, "Test Artist", "Test Album", override_data.clone());
    assert!(store_result.is_ok(), "Should store override successfully");

    // Load it back
    let load_result = load_user_metadata_override(root, "Test Artist", "Test Album");
    assert!(load_result.is_ok(), "Should load override successfully");

    let loaded = load_result.unwrap();
    assert!(loaded.is_some(), "Should have loaded the override");

    let loaded = loaded.unwrap();
    assert_eq!(loaded.genre, Some("Custom Genre".to_string()));
    assert_eq!(loaded.year, Some(2021));
    assert!(loaded.genre_overridden);
    assert!(loaded.year_overridden);
    assert!(loaded.edited_at > 0, "edited_at should be set");
}

#[test]
fn test_store_user_metadata_override_updates_edited_at() {
    use grape::library::metadata::online::{
        load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
    };

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let override_data1 = UserMetadataOverride {
        genre: Some("Genre 1".to_string()),
        year: Some(2020),
        genre_overridden: true,
        year_overridden: false,
        edited_at: 0,
    };

    // Store first time
    store_user_metadata_override(root, "Artist", "Album", override_data1)
        .expect("Failed to store first time");

    let first = load_user_metadata_override(root, "Artist", "Album")
        .expect("Failed to load first")
        .expect("Should exist");
    let first_edited_at = first.edited_at;

    // Wait a bit
    thread::sleep(Duration::from_millis(50));

    // Store again with different data
    let override_data2 = UserMetadataOverride {
        genre: Some("Genre 2".to_string()),
        year: Some(2021),
        genre_overridden: true,
        year_overridden: true,
        edited_at: 0,
    };

    store_user_metadata_override(root, "Artist", "Album", override_data2)
        .expect("Failed to store second time");

    let second = load_user_metadata_override(root, "Artist", "Album")
        .expect("Failed to load second")
        .expect("Should exist");

    assert_eq!(second.genre, Some("Genre 2".to_string()));
    assert_eq!(second.year, Some(2021));
    assert!(
        second.edited_at >= first_edited_at,
        "Second edited_at should be >= first"
    );
}

#[test]
fn test_multiple_overrides_different_albums() {
    use grape::library::metadata::online::{
        load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
    };

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // Store override for album 1
    let override1 = UserMetadataOverride {
        genre: Some("Rock".to_string()),
        year: Some(2020),
        genre_overridden: true,
        year_overridden: false,
        edited_at: 0,
    };
    store_user_metadata_override(root, "Artist", "Album 1", override1)
        .expect("Failed to store album 1");

    // Store override for album 2
    let override2 = UserMetadataOverride {
        genre: Some("Jazz".to_string()),
        year: Some(2021),
        genre_overridden: true,
        year_overridden: true,
        edited_at: 0,
    };
    store_user_metadata_override(root, "Artist", "Album 2", override2)
        .expect("Failed to store album 2");

    // Load both and verify they're different
    let loaded1 = load_user_metadata_override(root, "Artist", "Album 1")
        .expect("Failed to load album 1")
        .expect("Album 1 should exist");

    let loaded2 = load_user_metadata_override(root, "Artist", "Album 2")
        .expect("Failed to load album 2")
        .expect("Album 2 should exist");

    assert_eq!(loaded1.genre, Some("Rock".to_string()));
    assert_eq!(loaded1.year, Some(2020));

    assert_eq!(loaded2.genre, Some("Jazz".to_string()));
    assert_eq!(loaded2.year, Some(2021));
}

#[test]
fn test_override_partial_fields() {
    use grape::library::metadata::online::{
        load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
    };

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // Override only genre, not year
    let override_data = UserMetadataOverride {
        genre: Some("Custom Genre".to_string()),
        year: None,
        genre_overridden: true,
        year_overridden: false,
        edited_at: 0,
    };

    store_user_metadata_override(root, "Artist", "Album", override_data)
        .expect("Failed to store");

    let loaded = load_user_metadata_override(root, "Artist", "Album")
        .expect("Failed to load")
        .expect("Should exist");

    assert_eq!(loaded.genre, Some("Custom Genre".to_string()));
    assert_eq!(loaded.year, None);
    assert!(loaded.genre_overridden);
    assert!(!loaded.year_overridden);
}

#[cfg(test)]
mod cache_key_tests {
    use std::fs;

    // We can't directly test metadata_cache_key as it's private,
    // but we can test that storing different artists/albums creates different cache files

    #[test]
    fn test_different_albums_create_different_cache_files() {
        use grape::library::metadata::online::{
            store_user_metadata_override, UserMetadataOverride,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();

        let override_data = UserMetadataOverride::default();

        // Store for two different albums
        store_user_metadata_override(root, "Artist", "Album 1", override_data.clone())
            .expect("Failed to store album 1");
        store_user_metadata_override(root, "Artist", "Album 2", override_data)
            .expect("Failed to store album 2");

        // Check that metadata cache directory has multiple files
        let metadata_dir = root.join(".grape_cache/metadata");
        assert!(metadata_dir.exists(), "Metadata directory should exist");

        let entries: Vec<_> = fs::read_dir(&metadata_dir)
            .expect("Failed to read metadata dir")
            .filter_map(|entry: Result<fs::DirEntry, std::io::Error>| entry.ok())
            .filter(|entry: &fs::DirEntry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext: &std::ffi::OsStr| ext.to_str())
                    == Some("json")
            })
            .collect();

        assert!(
            entries.len() >= 2,
            "Should have at least 2 cache files for different albums"
        );
    }

    #[test]
    fn test_same_album_case_insensitive() {
        use grape::library::metadata::online::{
            load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();

        let override_data = UserMetadataOverride {
            genre: Some("Test Genre".to_string()),
            year: Some(2020),
            genre_overridden: true,
            year_overridden: true,
            edited_at: 0,
        };

        // Store with lowercase
        store_user_metadata_override(root, "artist", "album", override_data)
            .expect("Failed to store");

        // Load with different case
        let loaded = load_user_metadata_override(root, "ARTIST", "ALBUM")
            .expect("Failed to load")
            .expect("Should exist even with different case");

        assert_eq!(loaded.genre, Some("Test Genre".to_string()));
    }
}

#[cfg(test)]
mod serialization_tests {
    #[test]
    fn test_online_metadata_serialization() {
        use grape::library::metadata::online::OnlineMetadata;

        let metadata = OnlineMetadata {
            genre: Some("Electronic".to_string()),
            year: Some(2023),
        };

        let serialized = serde_json::to_string(&metadata).expect("Failed to serialize");
        let deserialized: OnlineMetadata =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(metadata, deserialized);
    }

    #[test]
    fn test_user_metadata_override_serialization() {
        use grape::library::metadata::online::UserMetadataOverride;

        let override_data = UserMetadataOverride {
            genre: Some("Custom".to_string()),
            year: Some(2021),
            genre_overridden: true,
            year_overridden: false,
            edited_at: 1234567890,
        };

        let serialized = serde_json::to_string(&override_data).expect("Failed to serialize");
        let deserialized: UserMetadataOverride =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(override_data.genre, deserialized.genre);
        assert_eq!(override_data.year, deserialized.year);
        assert_eq!(
            override_data.genre_overridden,
            deserialized.genre_overridden
        );
        assert_eq!(override_data.year_overridden, deserialized.year_overridden);
        assert_eq!(override_data.edited_at, deserialized.edited_at);
    }

    #[test]
    fn test_deserialization_with_missing_fields() {
        use grape::library::metadata::online::UserMetadataOverride;

        // JSON without the newer fields (should use defaults)
        let json = r#"{
            "genre": "Rock",
            "year": 2020
        }"#;

        let deserialized: UserMetadataOverride =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(deserialized.genre, Some("Rock".to_string()));
        assert_eq!(deserialized.year, Some(2020));
        assert!(!deserialized.genre_overridden); // Default should be false
        assert!(!deserialized.year_overridden); // Default should be false
        assert_eq!(deserialized.edited_at, 0); // Default should be 0
    }
}

#[cfg(test)]
mod edge_cases {

    #[test]
    fn test_empty_artist_and_album() {
        use grape::library::metadata::online::{
            load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();

        let override_data = UserMetadataOverride {
            genre: Some("Test".to_string()),
            year: Some(2020),
            genre_overridden: true,
            year_overridden: false,
            edited_at: 0,
        };

        // Should handle empty strings
        let result = store_user_metadata_override(root, "", "", override_data);
        assert!(result.is_ok(), "Should handle empty artist/album");

        let loaded = load_user_metadata_override(root, "", "");
        assert!(loaded.is_ok(), "Should load with empty artist/album");
    }

    #[test]
    fn test_special_characters_in_names() {
        use grape::library::metadata::online::{
            load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();

        let override_data = UserMetadataOverride {
            genre: Some("Test".to_string()),
            year: Some(2020),
            genre_overridden: true,
            year_overridden: false,
            edited_at: 0,
        };

        // Test with special characters
        let artist = "Artist/With\\Special:Characters?";
        let album = "Album|With<Special>Characters*";

        let result = store_user_metadata_override(root, artist, album, override_data);
        assert!(
            result.is_ok(),
            "Should handle special characters in names"
        );

        let loaded = load_user_metadata_override(root, artist, album);
        assert!(loaded.is_ok(), "Should load with special characters");
        assert!(loaded.unwrap().is_some(), "Should have stored the data");
    }

    #[test]
    fn test_very_long_names() {
        use grape::library::metadata::online::{
            store_user_metadata_override, UserMetadataOverride,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();

        let override_data = UserMetadataOverride::default();

        // Very long names
        let long_artist = "A".repeat(500);
        let long_album = "B".repeat(500);

        let result = store_user_metadata_override(root, &long_artist, &long_album, override_data);
        assert!(result.is_ok(), "Should handle very long names");
    }

    #[test]
    fn test_unicode_in_names() {
        use grape::library::metadata::online::{
            load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();

        let override_data = UserMetadataOverride {
            genre: Some("世界音楽".to_string()),
            year: Some(2020),
            genre_overridden: true,
            year_overridden: false,
            edited_at: 0,
        };

        // Unicode characters
        let artist = "アーティスト";
        let album = "アルバム 🎵";

        let result = store_user_metadata_override(root, artist, album, override_data);
        assert!(result.is_ok(), "Should handle Unicode characters");

        let loaded = load_user_metadata_override(root, artist, album)
            .expect("Failed to load")
            .expect("Should exist");

        assert_eq!(loaded.genre, Some("世界音楽".to_string()));
    }
}

#[cfg(test)]
mod integration_tests {

    #[test]
    fn test_full_metadata_workflow() {
        use grape::library::metadata::online::{
            load_user_metadata_override, store_user_metadata_override, UserMetadataOverride,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path();

        let artist = "Test Artist";
        let album = "Test Album";

        // 1. Initially no override
        let initial = load_user_metadata_override(root, artist, album)
            .expect("Failed initial load");
        assert!(initial.is_none(), "Should have no override initially");

        // 2. Store an override
        let override1 = UserMetadataOverride {
            genre: Some("Rock".to_string()),
            year: Some(2020),
            genre_overridden: true,
            year_overridden: false,
            edited_at: 0,
        };
        store_user_metadata_override(root, artist, album, override1)
            .expect("Failed to store first override");

        // 3. Load and verify
        let loaded1 = load_user_metadata_override(root, artist, album)
            .expect("Failed to load first override")
            .expect("First override should exist");
        assert_eq!(loaded1.genre, Some("Rock".to_string()));
        assert_eq!(loaded1.year, Some(2020));

        // 4. Update the override
        let override2 = UserMetadataOverride {
            genre: Some("Jazz".to_string()),
            year: Some(2021),
            genre_overridden: true,
            year_overridden: true,
            edited_at: 0,
        };
        store_user_metadata_override(root, artist, album, override2)
            .expect("Failed to store second override");

        // 5. Load updated version
        let loaded2 = load_user_metadata_override(root, artist, album)
            .expect("Failed to load second override")
            .expect("Second override should exist");
        assert_eq!(loaded2.genre, Some("Jazz".to_string()));
        assert_eq!(loaded2.year, Some(2021));
        assert!(loaded2.year_overridden);
    }
}
