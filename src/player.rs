#![allow(dead_code)]

use std::fmt;
use std::fs::File;

use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type};
use rodio::{Decoder, OutputStream, Sink, source::Source};
use tracing::{error, info};

mod audio_options {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    use rodio::{OutputStream, OutputStreamBuilder, cpal};
    use tracing::{info, warn};

    use crate::config::{AudioOutputDevice, MissingDeviceBehavior, UserSettings};
    use crate::player::PlayerError;

    #[derive(Debug, Clone, PartialEq)]
    pub struct AudioOptions {
        pub output_device: AudioOutputDevice,
        pub sample_rate_hz: Option<u32>,
        pub missing_device_behavior: MissingDeviceBehavior,
    }

    pub struct AudioStreamOutcome {
        pub stream: OutputStream,
        pub fallback_to_default: bool,
        pub missing_device: bool,
    }

    impl AudioOptions {
        pub fn from_settings(settings: &UserSettings) -> Self {
            Self {
                output_device: settings.output_device,
                sample_rate_hz: settings.output_sample_rate_hz,
                missing_device_behavior: settings.missing_device_behavior,
            }
        }

        pub fn open_stream(&self) -> Result<AudioStreamOutcome, PlayerError> {
            let resolution = self.resolve_device()?;
            let builder = self.builder(resolution.device)?;
            match builder.open_stream() {
                Ok(stream) => Ok(AudioStreamOutcome {
                    stream,
                    fallback_to_default: self.should_fallback(resolution.missing_device),
                    missing_device: resolution.missing_device,
                }),
                Err(err) => {
                    if self.is_default() {
                        Err(err.into())
                    } else {
                        warn!(error = %err, "Failed to open stream with custom options, falling back");
                        OutputStreamBuilder::open_default_stream()
                            .map(|stream| AudioStreamOutcome {
                                stream,
                                fallback_to_default: true,
                                missing_device: resolution.missing_device,
                            })
                            .map_err(PlayerError::from)
                    }
                }
            }
        }

        fn builder(
            &self,
            device: Option<cpal::Device>,
        ) -> Result<OutputStreamBuilder, PlayerError> {
            let builder = if let Some(device) = device {
                OutputStreamBuilder::from_device(device)?
            } else {
                OutputStreamBuilder::from_default_device()?
            };
            let builder = if let Some(sample_rate) = self.sample_rate_hz {
                builder.with_sample_rate(sample_rate)
            } else {
                builder
            };
            Ok(builder)
        }

        fn resolve_device(&self) -> Result<DeviceResolution, PlayerError> {
            match self.output_device {
                AudioOutputDevice::System => {
                    Ok(DeviceResolution { device: None, missing_device: false })
                }
                AudioOutputDevice::UsbHeadset => {
                    let host = cpal::default_host();
                    let devices = host.output_devices().map_err(PlayerError::from)?;
                    for device in devices {
                        let name = device.name().unwrap_or_default();
                        let lowered = name.to_lowercase();
                        if lowered.contains("usb") || lowered.contains("headset") {
                            info!(device = %name, "Selected USB headset output device");
                            return Ok(DeviceResolution {
                                device: Some(device),
                                missing_device: false,
                            });
                        }
                    }
                    warn!("USB headset output device not found, using default");
                    Ok(DeviceResolution { device: None, missing_device: true })
                }
            }
        }

        fn is_default(&self) -> bool {
            self.output_device == AudioOutputDevice::System && self.sample_rate_hz.is_none()
        }

        fn should_fallback(&self, missing_device: bool) -> bool {
            !self.is_default() && missing_device
        }
    }

    impl Default for AudioOptions {
        fn default() -> Self {
            Self {
                output_device: AudioOutputDevice::System,
                sample_rate_hz: None,
                missing_device_behavior: MissingDeviceBehavior::SwitchToSystem,
            }
        }
    }

    struct DeviceResolution {
        device: Option<cpal::Device>,
        missing_device: bool,
    }
}

use crate::config::{EqPreset, UserSettings, VolumeLevel};
use crate::eq::EqModel;
pub use audio_options::AudioOptions;
use audio_options::AudioStreamOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlaying {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub duration_secs: u32,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub now_playing: Option<NowPlaying>,
    pub is_playing: bool,
    pub position_secs: u32,
}

impl PlayerState {
    pub fn placeholder() -> Self {
        Self {
            now_playing: None,
            is_playing: false,
            position_secs: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Paused,
    Playing,
}

#[derive(Debug)]
pub enum PlayerError {
    Io(io::Error),
    DecoderError(rodio::decoder::DecoderError),
    StreamError(rodio::StreamError),
    PlayError(rodio::PlayError),
    DeviceError(rodio::cpal::DevicesError),
    NoTrackLoaded,
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::Io(err) => write!(f, "io error: {err}"),
            PlayerError::DecoderError(err) => write!(f, "decoder error: {err}"),
            PlayerError::StreamError(err) => write!(f, "stream error: {err}"),
            PlayerError::PlayError(err) => write!(f, "play error: {err}"),
            PlayerError::DeviceError(err) => write!(f, "device error: {err}"),
            PlayerError::NoTrackLoaded => write!(f, "no track loaded"),
        }
    }
}

impl std::error::Error for PlayerError {}

impl From<io::Error> for PlayerError {
    fn from(err: io::Error) -> Self {
        PlayerError::Io(err)
    }
}

impl From<rodio::decoder::DecoderError> for PlayerError {
    fn from(err: rodio::decoder::DecoderError) -> Self {
        PlayerError::DecoderError(err)
    }
}

impl From<rodio::StreamError> for PlayerError {
    fn from(err: rodio::StreamError) -> Self {
        PlayerError::StreamError(err)
    }
}

impl From<rodio::PlayError> for PlayerError {
    fn from(err: rodio::PlayError) -> Self {
        PlayerError::PlayError(err)
    }
}

impl From<rodio::cpal::DevicesError> for PlayerError {
    fn from(err: rodio::cpal::DevicesError) -> Self {
        PlayerError::DeviceError(err)
    }
}

pub struct Player {
    stream: OutputStream,
    sink: Sink,
    state: PlaybackState,
    pub(crate) current_track: Option<PathBuf>,
    pub(crate) position: Duration,
    pub(crate) started_at: Option<Instant>,
    options: AudioOptions,
    processing: AudioProcessingConfig,
    output_volume: f32,
    playback_speed: f32,
    last_fallback: Option<AudioFallback>,
    debug_logs: bool,
}

impl Player {
    pub fn new() -> Result<Self, PlayerError> {
        Self::new_with_settings(&UserSettings::default())
    }

    pub fn new_with_settings(settings: &UserSettings) -> Result<Self, PlayerError> {
        let options = AudioOptions::from_settings(settings);
        let processing = AudioProcessingConfig::from_settings(settings);
        let output_volume = output_volume_from_settings(settings);
        let playback_speed = playback_speed_from_settings(settings);
        let outcome = options.open_stream()?;
        let (stream, resolved_options, last_fallback) =
            Self::stream_outcome_to_player_state(options, outcome);
        let sink = Sink::connect_new(stream.mixer());
        let mut player = Self {
            stream,
            sink,
            state: PlaybackState::Stopped,
            current_track: None,
            position: Duration::ZERO,
            started_at: None,
            options: resolved_options,
            processing,
            output_volume,
            playback_speed,
            last_fallback,
            debug_logs: settings.audio_debug_logs,
        };
        player.apply_output_volume();
        player.apply_playback_speed();
        player.log_audio_config("init");
        Ok(player)
    }

    pub fn reset(&mut self, options: AudioOptions) -> Result<(), PlayerError> {
        let processing = self.processing.clone();
        let outcome = options.open_stream()?;
        let (stream, resolved_options, last_fallback) =
            Self::stream_outcome_to_player_state(options, outcome);
        let sink = Sink::connect_new(stream.mixer());
        self.stream = stream;
        self.sink = sink;
        self.state = PlaybackState::Stopped;
        self.current_track = None;
        self.position = Duration::ZERO;
        self.started_at = None;
        self.options = resolved_options;
        self.processing = processing;
        self.last_fallback = last_fallback;
        self.apply_output_volume();
        self.apply_playback_speed();
        self.log_audio_config("reset");
        Ok(())
    }

    pub fn apply_settings(&mut self, settings: &UserSettings) -> Result<(), PlayerError> {
        let mut updated = false;
        let was_debug_logs = self.debug_logs;
        let options = AudioOptions::from_settings(settings);
        let processing = AudioProcessingConfig::from_settings(settings);
        let output_volume = output_volume_from_settings(settings);
        let playback_speed = playback_speed_from_settings(settings);
        if self.debug_logs != settings.audio_debug_logs {
            self.debug_logs = settings.audio_debug_logs;
            info!(enabled = self.debug_logs, "Audio debug logging preference updated");
            if !was_debug_logs && self.debug_logs {
                self.log_audio_config("debug_enabled");
            }
        }
        if options != self.options {
            let outcome = options.open_stream()?;
            let (stream, resolved_options, last_fallback) =
                Self::stream_outcome_to_player_state(options, outcome);
            self.stream = stream;
            self.options = resolved_options;
            self.last_fallback = last_fallback;
            updated = true;
        }
        if processing != self.processing {
            self.processing = processing;
            updated = true;
        }
        if (self.output_volume - output_volume).abs() > f32::EPSILON {
            self.output_volume = output_volume;
            self.apply_output_volume();
        }
        if (self.playback_speed - playback_speed).abs() > f32::EPSILON {
            self.playback_speed = playback_speed;
            self.apply_playback_speed();
        }
        if updated {
            self.reload_current_track()?;
            self.log_audio_config("apply_settings");
        }
        Ok(())
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), PlayerError> {
        let path = path.as_ref().to_path_buf();
        info!(path = %path.display(), "Loading track");
        self.current_track = Some(path.clone());
        self.position = Duration::ZERO;
        self.started_at = None;
        self.sink.stop();
        self.sink = Sink::connect_new(self.stream.mixer());
        self.apply_output_volume();
        self.apply_playback_speed();
        let source = self.processed_source(&path, None).map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to load track");
            err
        })?;
        self.sink.append(source);
        self.sink.pause();
        self.state = PlaybackState::Paused;
        Ok(())
    }

    /// Appends a track to the current sink without stopping it.
    /// Used for gapless playback: the next track plays seamlessly when the
    /// current one ends, without a stop/recreate cycle.
    pub fn append_gapless(&mut self, path: impl AsRef<Path>) -> Result<(), PlayerError> {
        let path = path.as_ref().to_path_buf();
        info!(path = %path.display(), "Queuing gapless next track");
        let source = self.processed_source(&path, None)?;
        self.sink.append(source);
        Ok(())
    }

    pub fn play(&mut self) {
        info!("Playback start");
        if self.state != PlaybackState::Playing {
            self.started_at = Some(Instant::now());
        }
        self.sink.play();
        self.state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        info!("Playback pause");
        if self.state == PlaybackState::Playing {
            if let Some(started_at) = self.started_at.take() {
                self.position = self.position.saturating_add(started_at.elapsed());
            }
        }
        self.sink.pause();
        self.state = PlaybackState::Paused;
    }

    pub fn seek(&mut self, position: Duration) -> Result<(), PlayerError> {
        info!(position_secs = position.as_secs(), "Seeking");
        let path = self.current_track.clone().ok_or(PlayerError::NoTrackLoaded)?;
        let prev_state = self.state;
        self.sink.stop();
        self.sink = Sink::connect_new(self.stream.mixer());
        self.apply_output_volume();
        self.apply_playback_speed();
        let source = self.processed_source(&path, Some(position)).map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to seek");
            err
        })?;
        // Only update position after the source was successfully created
        self.position = position;
        self.sink.append(source);
        match prev_state {
            PlaybackState::Playing => {
                self.started_at = Some(Instant::now());
                self.sink.play();
            }
            _ => {
                self.started_at = None;
                self.sink.pause();
            }
        }
        Ok(())
    }

    pub fn state(&self) -> PlaybackState {
        self.state
    }

    pub fn position(&self) -> Duration {
        if self.state == PlaybackState::Playing {
            if let Some(started_at) = self.started_at {
                return self.position.saturating_add(started_at.elapsed());
            }
        }
        self.position
    }

    pub fn take_last_fallback_notice(&mut self) -> Option<AudioFallback> {
        self.last_fallback.take()
    }

    fn log_audio_config(&self, context: &str) {
        if self.debug_logs {
            info!(
                context,
                options = ?self.options,
                processing = ?self.processing,
                "Audio debug configuration"
            );
        }
    }

    fn reload_current_track(&mut self) -> Result<(), PlayerError> {
        self.sink.stop();
        self.sink = Sink::connect_new(self.stream.mixer());
        self.apply_output_volume();
        self.apply_playback_speed();
        let Some(path) = self.current_track.clone() else {
            self.state = PlaybackState::Stopped;
            self.position = Duration::ZERO;
            self.started_at = None;
            return Ok(());
        };
        let position = self.position;
        let state = self.state;
        let source = self.processed_source(&path, Some(position))?;
        self.sink.append(source);
        match state {
            PlaybackState::Playing => {
                self.started_at = Some(Instant::now());
                self.sink.play();
            }
            PlaybackState::Paused | PlaybackState::Stopped => {
                self.started_at = None;
                self.sink.pause();
            }
        }
        Ok(())
    }

    fn processed_source(
        &self,
        path: &Path,
        position: Option<Duration>,
    ) -> Result<AudioProcessingSource<Box<dyn Source<Item = f32> + Send>>, PlayerError> {
        let position = position.unwrap_or(Duration::ZERO);
        let seekable = self.decode_seekable_source(path);
        if let Ok(mut decoder) = seekable {
            if position == Duration::ZERO || decoder.try_seek(position).is_ok() {
                return AudioProcessingSource::new(Box::new(decoder), &self.processing);
            }
            // Explicitly drop the seekable decoder to release the file handle
            // before opening a new one for the skip_duration fallback.
            drop(decoder);
        }
        let source = self.decode_source(path)?;
        let source = source.skip_duration(position);
        AudioProcessingSource::new(Box::new(source), &self.processing)
    }

    fn decode_source(&self, path: &Path) -> Result<Decoder<io::BufReader<File>>, PlayerError> {
        let file = File::open(path).map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to open track file");
            err
        })?;
        Decoder::new(io::BufReader::new(file)).map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to decode track");
            err.into()
        })
    }

    fn decode_seekable_source(
        &self,
        path: &Path,
    ) -> Result<Decoder<io::BufReader<File>>, PlayerError> {
        let file = File::open(path).map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to open track file");
            err
        })?;
        let byte_len = file.metadata().map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to read track metadata");
            err
        })?;
        Decoder::builder()
            .with_data(io::BufReader::new(file))
            .with_byte_len(byte_len.len())
            .build()
            .map_err(|err| {
                error!(error = %err, path = %path.display(), "Failed to decode track");
                err.into()
            })
    }

    fn stream_outcome_to_player_state(
        options: AudioOptions,
        outcome: AudioStreamOutcome,
    ) -> (OutputStream, AudioOptions, Option<AudioFallback>) {
        if outcome.fallback_to_default {
            let fallback = AudioFallback {
                missing_device: outcome.missing_device,
                behavior: options.missing_device_behavior,
            };
            (outcome.stream, AudioOptions::default(), Some(fallback))
        } else {
            (outcome.stream, options, None)
        }
    }

    fn apply_output_volume(&mut self) {
        self.sink.set_volume(self.output_volume);
    }

    fn apply_playback_speed(&mut self) {
        self.sink.set_speed(self.playback_speed);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AudioProcessingConfig {
    eq_enabled: bool,
    eq_preset: EqPreset,
    eq_model: EqModel,
    normalize_volume: bool,
    volume_level: VolumeLevel,
}

impl AudioProcessingConfig {
    fn from_settings(settings: &UserSettings) -> Self {
        Self {
            eq_enabled: settings.eq_enabled,
            eq_preset: settings.eq_preset,
            eq_model: settings.eq_model.clone().normalized().clamp_gains(-12.0, 12.0),
            normalize_volume: settings.normalize_volume,
            volume_level: settings.volume_level,
        }
    }

    fn target_gain(&self) -> f32 {
        match self.volume_level {
            VolumeLevel::Quiet => 0.75,
            VolumeLevel::Normal => 1.0,
            VolumeLevel::Loud => 1.25,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AudioFallback {
    pub missing_device: bool,
    pub behavior: crate::config::MissingDeviceBehavior,
}

impl AudioFallback {
    pub fn notice(&self, language: crate::config::InterfaceLanguage) -> String {
        match (self.missing_device, self.behavior) {
            (true, crate::config::MissingDeviceBehavior::PausePlayback) => match language {
                crate::config::InterfaceLanguage::English => {
                    "Audio device not found. Playback paused, switching to system output."
                        .to_string()
                }
                _ => "Périphérique audio introuvable. Lecture mise en pause, retour au système."
                    .to_string(),
            },
            (true, crate::config::MissingDeviceBehavior::SwitchToSystem) => match language {
                crate::config::InterfaceLanguage::English => {
                    "Audio device not found. Switching back to system output.".to_string()
                }
                _ => "Périphérique audio introuvable. Retour à la sortie système.".to_string(),
            },
            (false, _) => match language {
                crate::config::InterfaceLanguage::English => {
                    "Audio configuration unavailable. Switching to system output.".to_string()
                }
                _ => "Configuration audio non disponible. Retour au système.".to_string(),
            },
        }
    }
}

struct AudioProcessingSource<S> {
    source: S,
    channels: u16,
    channel_index: u16,
    gain: f32,
    eq: Option<EqFilters>,
}

impl<S> AudioProcessingSource<S>
where
    S: Source<Item = f32>,
{
    fn new(source: S, config: &AudioProcessingConfig) -> Result<Self, PlayerError> {
        let channels = source.channels();
        let sample_rate = source.sample_rate();
        let eq = if config.eq_enabled {
            Some(EqFilters::new(&config.eq_model, sample_rate, channels)?)
        } else {
            None
        };
        Ok(Self {
            source,
            channels,
            channel_index: 0,
            gain: config.target_gain(),
            eq,
        })
    }
}

impl<S> Iterator for AudioProcessingSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.source.next()?;
        let channel = self.channel_index as usize;
        self.channel_index = (self.channel_index + 1) % self.channels.max(1);
        let mut processed = sample;
        if let Some(eq) = self.eq.as_mut() {
            processed = eq.apply(channel, processed);
        }
        Some(processed * self.gain)
    }
}

fn output_volume_from_settings(settings: &UserSettings) -> f32 {
    (settings.default_volume as f32 / 100.0).clamp(0.0, 1.0)
}

fn playback_speed_from_settings(settings: &UserSettings) -> f32 {
    (settings.default_playback_speed as f32 / 10.0).clamp(0.5, 2.0)
}

impl<S> Source for AudioProcessingSource<S>
where
    S: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}

struct EqFilters {
    filters: Vec<Vec<DirectForm1<f32>>>,
}

impl EqFilters {
    fn new(model: &EqModel, sample_rate: u32, channels: u16) -> Result<Self, PlayerError> {
        let mut filters = Vec::new();
        let sample_rate_hz = sample_rate.max(1) as f32;
        let nyquist = (sample_rate_hz / 2.0).max(20.0);
        for band in &model.bands {
            let target = (band.frequency_hz as f32).min(nyquist - 1.0);
            let coeffs = Coefficients::<f32>::from_params(
                Type::PeakingEQ(band.gain_db),
                sample_rate_hz.hz(),
                target.hz(),
                0.707,
            )
            .map_err(|err| {
                PlayerError::Io(io::Error::new(io::ErrorKind::Other, format!("{err:?}")))
            })?;
            let band_filters = (0..channels).map(|_| DirectForm1::<f32>::new(coeffs)).collect();
            filters.push(band_filters);
        }
        Ok(Self { filters })
    }

    fn apply(&mut self, channel: usize, sample: f32) -> f32 {
        let mut value = sample;
        for band in &mut self.filters {
            if let Some(filter) = band.get_mut(channel) {
                value = filter.run(value);
            }
        }
        value
    }
}
