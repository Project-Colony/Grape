// NOTE: Most tests in this file require audio hardware and are marked with #[ignore].
// To run them, use: cargo test --test player_tests -- --ignored
// These tests will fail in CI/CD environments without audio devices.

use grape::player::{AudioOptions, NowPlaying, Player, PlaybackState, PlayerState};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Helper function to create a test audio file (silence WAV)
fn create_test_wav(dir: &TempDir, name: &str, duration_secs: u8) -> PathBuf {
    let path = dir.path().join(name);
    // Create a minimal valid WAV file with silence
    // WAV header format: RIFF chunk + fmt chunk + data chunk
    let sample_rate: u32 = 44100;
    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample) / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let num_samples = sample_rate * u32::from(duration_secs);
    let data_size = num_samples * u32::from(num_channels) * u32::from(bits_per_sample) / 8;

    let mut wav_data = Vec::new();

    // RIFF header
    wav_data.extend_from_slice(b"RIFF");
    wav_data.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav_data.extend_from_slice(b"WAVE");

    // fmt chunk
    wav_data.extend_from_slice(b"fmt ");
    wav_data.extend_from_slice(&16_u32.to_le_bytes()); // chunk size
    wav_data.extend_from_slice(&1_u16.to_le_bytes()); // audio format (PCM)
    wav_data.extend_from_slice(&num_channels.to_le_bytes());
    wav_data.extend_from_slice(&sample_rate.to_le_bytes());
    wav_data.extend_from_slice(&byte_rate.to_le_bytes());
    wav_data.extend_from_slice(&block_align.to_le_bytes());
    wav_data.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav_data.extend_from_slice(b"data");
    wav_data.extend_from_slice(&data_size.to_le_bytes());

    // Write silence (zeros) for the specified duration
    wav_data.extend(vec![0_u8; data_size as usize]);

    std::fs::write(&path, wav_data).expect("Failed to write test WAV file");
    path
}

    #[ignore]
#[test]
fn test_player_creation() {
    // Test that we can create a player
    let player = Player::new();
    assert!(
        player.is_ok(),
        "Player creation should succeed on systems with audio output"
    );
}

    #[ignore]
#[test]
fn test_player_initial_state() {
    let player = Player::new().expect("Failed to create player");

    // Initial state should be Stopped
    assert_eq!(player.state(), PlaybackState::Stopped);

    // Initial position should be zero
    assert_eq!(player.position(), Duration::ZERO);
}

#[test]
fn test_player_state_placeholder() {
    let state = PlayerState::placeholder();

    assert!(state.now_playing.is_none());
    assert!(!state.is_playing);
    assert_eq!(state.position_secs, 0);
}

#[test]
fn test_now_playing_fields() {
    let now_playing = NowPlaying {
        artist: "Test Artist".to_string(),
        album: "Test Album".to_string(),
        title: "Test Track".to_string(),
        duration_secs: 180,
        path: PathBuf::from("/test/path.mp3"),
    };

    assert_eq!(now_playing.artist, "Test Artist");
    assert_eq!(now_playing.album, "Test Album");
    assert_eq!(now_playing.title, "Test Track");
    assert_eq!(now_playing.duration_secs, 180);
    assert_eq!(now_playing.path, PathBuf::from("/test/path.mp3"));
}

    #[ignore]
#[test]
fn test_load_nonexistent_file() {
    let mut player = Player::new().expect("Failed to create player");
    let nonexistent_path = PathBuf::from("/tmp/nonexistent_file_grape_test.wav");

    let result = player.load(&nonexistent_path);
    assert!(result.is_err(), "Loading nonexistent file should fail");
}

    #[ignore]
#[test]
fn test_load_and_state_transition() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let wav_path = create_test_wav(&temp_dir, "test.wav", 2);

    let mut player = Player::new().expect("Failed to create player");

    // Initially stopped
    assert_eq!(player.state(), PlaybackState::Stopped);

    // Load track
    let result = player.load(&wav_path);
    assert!(result.is_ok(), "Loading valid WAV should succeed");

    // After loading, should be paused
    assert_eq!(player.state(), PlaybackState::Paused);
}

    #[ignore]
#[test]
fn test_play_pause_cycle() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let wav_path = create_test_wav(&temp_dir, "test.wav", 2);

    let mut player = Player::new().expect("Failed to create player");
    player.load(&wav_path).expect("Failed to load track");

    // Should start paused
    assert_eq!(player.state(), PlaybackState::Paused);

    // Play
    player.play();
    assert_eq!(player.state(), PlaybackState::Playing);

    // Pause
    player.pause();
    assert_eq!(player.state(), PlaybackState::Paused);

    // Play again
    player.play();
    assert_eq!(player.state(), PlaybackState::Playing);
}

    #[ignore]
#[test]
fn test_position_tracking() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let wav_path = create_test_wav(&temp_dir, "test.wav", 3);

    let mut player = Player::new().expect("Failed to create player");
    player.load(&wav_path).expect("Failed to load track");

    // Position should be zero after loading
    assert_eq!(player.position(), Duration::ZERO);

    // Start playing
    player.play();

    // Wait a bit
    std::thread::sleep(Duration::from_millis(100));

    // Position should have advanced
    let position = player.position();
    assert!(
        position > Duration::ZERO,
        "Position should advance during playback"
    );

    // Pause
    player.pause();

    // Position should remain stable while paused
    let paused_position = player.position();
    std::thread::sleep(Duration::from_millis(50));
    let still_paused_position = player.position();

    assert_eq!(
        paused_position, still_paused_position,
        "Position should not advance while paused"
    );
}

    #[ignore]
#[test]
fn test_seek() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let wav_path = create_test_wav(&temp_dir, "test.wav", 5);

    let mut player = Player::new().expect("Failed to create player");
    player.load(&wav_path).expect("Failed to load track");

    // Seek to 2 seconds
    let seek_position = Duration::from_secs(2);
    let result = player.seek(seek_position);
    assert!(result.is_ok(), "Seeking should succeed");

    // Position should be at seek position
    let position = player.position();
    assert!(
        (position.as_millis() as i64 - seek_position.as_millis() as i64).abs() < 100,
        "Position should be close to seek position"
    );
}

    #[ignore]
#[test]
fn test_seek_without_loaded_track() {
    let mut player = Player::new().expect("Failed to create player");

    // Try to seek without loading a track
    let result = player.seek(Duration::from_secs(1));
    assert!(
        result.is_err(),
        "Seeking without a loaded track should fail"
    );
}

    #[ignore]
#[test]
fn test_seek_while_playing() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let wav_path = create_test_wav(&temp_dir, "test.wav", 5);

    let mut player = Player::new().expect("Failed to create player");
    player.load(&wav_path).expect("Failed to load track");
    player.play();

    // Seek while playing
    let seek_position = Duration::from_secs(2);
    let result = player.seek(seek_position);
    assert!(result.is_ok(), "Seeking while playing should succeed");

    // Should still be playing after seek
    assert_eq!(player.state(), PlaybackState::Playing);
}

    #[ignore]
#[test]
fn test_seek_while_paused() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let wav_path = create_test_wav(&temp_dir, "test.wav", 5);

    let mut player = Player::new().expect("Failed to create player");
    player.load(&wav_path).expect("Failed to load track");
    // Don't play, leave it paused

    // Seek while paused
    let seek_position = Duration::from_secs(2);
    let result = player.seek(seek_position);
    assert!(result.is_ok(), "Seeking while paused should succeed");

    // Should still be paused after seek
    assert_eq!(player.state(), PlaybackState::Paused);
}

    #[ignore]
#[test]
fn test_multiple_loads() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let wav_path1 = create_test_wav(&temp_dir, "test1.wav", 2);
    let wav_path2 = create_test_wav(&temp_dir, "test2.wav", 3);

    let mut player = Player::new().expect("Failed to create player");

    // Load first track
    player.load(&wav_path1).expect("Failed to load track 1");
    player.play();
    std::thread::sleep(Duration::from_millis(100));

    // Load second track (should replace first)
    let result = player.load(&wav_path2);
    assert!(result.is_ok(), "Loading second track should succeed");

    // Should be paused after loading new track
    assert_eq!(player.state(), PlaybackState::Paused);

    // Position should be reset
    assert_eq!(player.position(), Duration::ZERO);
}

#[test]
fn test_audio_options_default() {
    let options = AudioOptions::default();
    // Just ensure defaults can be created
    assert_eq!(options, AudioOptions::default());
}

#[test]
fn test_playback_states() {
    // Test all playback states exist and are distinct
    assert_ne!(PlaybackState::Stopped, PlaybackState::Paused);
    assert_ne!(PlaybackState::Stopped, PlaybackState::Playing);
    assert_ne!(PlaybackState::Paused, PlaybackState::Playing);
}

#[test]
fn test_now_playing_clone() {
    let now_playing = NowPlaying {
        artist: "Artist".to_string(),
        album: "Album".to_string(),
        title: "Title".to_string(),
        duration_secs: 100,
        path: PathBuf::from("/test.mp3"),
    };

    let cloned = now_playing.clone();
    assert_eq!(now_playing, cloned);
}

#[cfg(test)]
mod audio_processing_tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_load_different_sample_rates() {
        // This test verifies that the player can handle different audio formats
        // In a real scenario, you'd create files with different sample rates
        // For now, we just verify the basic WAV loading works
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let wav_path = create_test_wav(&temp_dir, "test_44khz.wav", 1);

        let mut player = Player::new().expect("Failed to create player");
        let result = player.load(&wav_path);
        assert!(result.is_ok(), "Should handle 44.1kHz audio");
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[ignore]
    #[test]
    fn test_pause_without_playing() {
        let mut player = Player::new().expect("Failed to create player");

        // Pause without playing should not crash
        player.pause();
        assert_eq!(player.state(), PlaybackState::Stopped);
    }

    #[ignore]
    #[test]
    fn test_play_without_loading() {
        let mut player = Player::new().expect("Failed to create player");

        // Play without loading should not crash
        player.play();
        // State might be Playing but no audio will play
        // This is acceptable behavior
    }

    #[ignore]
    #[test]
    fn test_position_without_track() {
        let player = Player::new().expect("Failed to create player");

        // Getting position without a track should return zero
        assert_eq!(player.position(), Duration::ZERO);
    }

    #[ignore]
    #[test]
    fn test_multiple_pauses() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let wav_path = create_test_wav(&temp_dir, "test.wav", 2);

        let mut player = Player::new().expect("Failed to create player");
        player.load(&wav_path).expect("Failed to load track");
        player.play();

        // Multiple pauses should be idempotent
        player.pause();
        let pos1 = player.position();
        player.pause();
        let pos2 = player.position();
        player.pause();
        let pos3 = player.position();

        assert_eq!(pos1, pos2);
        assert_eq!(pos2, pos3);
    }

    #[ignore]
    #[test]
    fn test_multiple_plays() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let wav_path = create_test_wav(&temp_dir, "test.wav", 2);

        let mut player = Player::new().expect("Failed to create player");
        player.load(&wav_path).expect("Failed to load track");

        // Multiple plays should be idempotent
        player.play();
        assert_eq!(player.state(), PlaybackState::Playing);
        player.play();
        assert_eq!(player.state(), PlaybackState::Playing);
        player.play();
        assert_eq!(player.state(), PlaybackState::Playing);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_full_playback_scenario() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let wav_path = create_test_wav(&temp_dir, "test.wav", 3);

        let mut player = Player::new().expect("Failed to create player");

        // 1. Load track
        player.load(&wav_path).expect("Failed to load");
        assert_eq!(player.state(), PlaybackState::Paused);
        assert_eq!(player.position(), Duration::ZERO);

        // 2. Play
        player.play();
        assert_eq!(player.state(), PlaybackState::Playing);
        std::thread::sleep(Duration::from_millis(100));
        assert!(player.position() > Duration::ZERO);

        // 3. Pause
        player.pause();
        assert_eq!(player.state(), PlaybackState::Paused);
        let paused_pos = player.position();

        // 4. Seek forward
        let seek_target = paused_pos + Duration::from_secs(1);
        player.seek(seek_target).expect("Failed to seek");
        assert!(player.position() >= seek_target);

        // 5. Play again
        player.play();
        assert_eq!(player.state(), PlaybackState::Playing);

        // 6. Final pause
        player.pause();
        assert_eq!(player.state(), PlaybackState::Paused);
    }
}
