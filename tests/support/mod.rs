use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a test audio file (silence WAV)
pub fn create_test_wav(dir: &TempDir, name: &str, duration_secs: u8) -> PathBuf {
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
