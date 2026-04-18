use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tracing::warn;

use crate::eq::EqModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    #[serde(alias = "Light")]
    Latte,
    Frappe,
    Macchiato,
    #[serde(alias = "Dark", alias = "System")]
    Mocha,
    GruvboxLight,
    GruvboxDark,
    EverblushLight,
    EverblushDark,
    KanagawaLight,
    KanagawaDark,
    KanagawaJournal,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Mocha
    }
}

impl ThemeMode {
    /// Returns the dark counterpart for the same theme family.
    pub fn dark_variant(self) -> Self {
        match self {
            Self::Latte => Self::Mocha,
            Self::GruvboxLight => Self::GruvboxDark,
            Self::EverblushLight => Self::EverblushDark,
            Self::KanagawaLight | Self::KanagawaJournal => Self::KanagawaDark,
            other => other, // already dark
        }
    }

    /// Returns the light counterpart for the same theme family.
    pub fn light_variant(self) -> Self {
        match self {
            Self::Frappe | Self::Macchiato | Self::Mocha => Self::Latte,
            Self::GruvboxDark => Self::GruvboxLight,
            Self::EverblushDark => Self::EverblushLight,
            Self::KanagawaDark => Self::KanagawaLight,
            other => other, // already light
        }
    }

    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match self {
            Self::Latte => "Latte",
            Self::Frappe => match language {
                InterfaceLanguage::English => "Frappe",
                _ => "Frappé",
            },
            Self::Macchiato => "Macchiato",
            Self::Mocha => "Mocha",
            Self::GruvboxLight => "Gruvbox Light",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::EverblushLight => "Everblush Light",
            Self::EverblushDark => "Everblush Dark",
            Self::KanagawaLight => "Kanagawa Light",
            Self::KanagawaDark => "Kanagawa Dark",
            Self::KanagawaJournal => "Kanagawa Journal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextScale {
    Normal,
    Large,
    ExtraLarge,
}

impl Default for TextScale {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessibleTextSize {
    Standard,
    Large,
    ExtraLarge,
}

impl Default for AccessibleTextSize {
    fn default() -> Self {
        Self::Standard
    }
}

impl AccessibleTextSize {
    pub fn scale(self) -> f32 {
        match self {
            Self::Standard => 1.0,
            Self::Large => 1.1,
            Self::ExtraLarge => 1.25,
        }
    }

    pub fn slider_value(self) -> f32 {
        match self {
            Self::Standard => 0.0,
            Self::Large => 1.0,
            Self::ExtraLarge => 2.0,
        }
    }

    pub fn from_slider_value(value: f32) -> Self {
        match value.round() as i32 {
            0 => Self::Standard,
            1 => Self::Large,
            _ => Self::ExtraLarge,
        }
    }

    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Standard, InterfaceLanguage::English) => "Standard",
            (Self::Large, InterfaceLanguage::English) => "Large",
            (Self::ExtraLarge, InterfaceLanguage::English) => "Extra large",
            (Self::Standard, _) => "Standard",
            (Self::Large, _) => "Grand",
            (Self::ExtraLarge, _) => "Très grand",
        }
    }
}

impl TextScale {
    pub fn scale(self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Large => 1.1,
            Self::ExtraLarge => 1.25,
        }
    }

    pub fn slider_value(self) -> f32 {
        match self {
            Self::Normal => 0.0,
            Self::Large => 1.0,
            Self::ExtraLarge => 2.0,
        }
    }

    pub fn from_slider_value(value: f32) -> Self {
        match value.round() as i32 {
            0 => Self::Normal,
            1 => Self::Large,
            _ => Self::ExtraLarge,
        }
    }

    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Normal, InterfaceLanguage::English) => "Normal",
            (Self::Large, InterfaceLanguage::English) => "Large",
            (Self::ExtraLarge, InterfaceLanguage::English) => "Extra large",
            (Self::Normal, _) => "Normal",
            (Self::Large, _) => "Large",
            (Self::ExtraLarge, _) => "Très grand",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccentColor {
    Red,
    Orange,
    Yellow,
    Blue,
    Indigo,
    Violet,
    Green,
    Amber,
}

impl Default for AccentColor {
    fn default() -> Self {
        Self::Blue
    }
}

impl AccentColor {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Red, InterfaceLanguage::English) => "Red",
            (Self::Orange, InterfaceLanguage::English) => "Orange",
            (Self::Yellow, InterfaceLanguage::English) => "Yellow",
            (Self::Blue, InterfaceLanguage::English) => "Blue",
            (Self::Indigo, InterfaceLanguage::English) => "Indigo",
            (Self::Violet, InterfaceLanguage::English) => "Violet",
            (Self::Green, InterfaceLanguage::English) => "Green",
            (Self::Amber, InterfaceLanguage::English) => "Amber",
            (Self::Red, _) => "Rouge",
            (Self::Orange, _) => "Orange",
            (Self::Yellow, _) => "Jaune",
            (Self::Blue, _) => "Bleu",
            (Self::Indigo, _) => "Indigo",
            (Self::Violet, _) => "Violet",
            (Self::Green, _) => "Vert",
            (Self::Amber, _) => "Ambre",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceDensity {
    Compact,
    Comfort,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarativeAction {
    ReindexLibrary,
    ClearCache,
    ResetAudioEngine,
}

impl DeclarativeAction {
    pub fn title(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::ReindexLibrary, InterfaceLanguage::English) => "Reindex library",
            (Self::ClearCache, InterfaceLanguage::English) => "Clear cache",
            (Self::ResetAudioEngine, InterfaceLanguage::English) => "Reset audio engine",
            (Self::ReindexLibrary, _) => "Réindexer la bibliothèque",
            (Self::ClearCache, _) => "Vider le cache",
            (Self::ResetAudioEngine, _) => "Réinitialiser l'audio",
        }
    }

    pub fn description(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::ReindexLibrary, InterfaceLanguage::English) => "Rebuilds the local index.",
            (Self::ClearCache, InterfaceLanguage::English) => "Removes temporary files.",
            (Self::ResetAudioEngine, InterfaceLanguage::English) => {
                "Restarts the rodio audio engine."
            }
            (Self::ReindexLibrary, _) => "Reconstruit l'index local.",
            (Self::ClearCache, _) => "Supprime les fichiers temporaires.",
            (Self::ResetAudioEngine, _) => "Redémarre le moteur audio rodio.",
        }
    }

    pub fn button_label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::ReindexLibrary, InterfaceLanguage::English) => "Reindex",
            (Self::ClearCache, InterfaceLanguage::English) => "Clear cache",
            (Self::ResetAudioEngine, InterfaceLanguage::English) => "Reset",
            (Self::ReindexLibrary, _) => "Réindexer",
            (Self::ClearCache, _) => "Vider le cache",
            (Self::ResetAudioEngine, _) => "Réinitialiser",
        }
    }

    pub fn confirm_label(self, language: InterfaceLanguage) -> &'static str {
        match language {
            InterfaceLanguage::English => "Confirm",
            _ => "Confirmer",
        }
    }
}

impl Default for InterfaceDensity {
    fn default() -> Self {
        Self::Comfort
    }
}

impl InterfaceDensity {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Compact, InterfaceLanguage::English) => "Compact",
            (Self::Comfort, InterfaceLanguage::English) => "Comfort",
            (Self::Large, InterfaceLanguage::English) => "Large",
            (Self::Compact, _) => "Compact",
            (Self::Comfort, _) => "Confort",
            (Self::Large, _) => "Large",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupScreen {
    Home,
    Library,
    Playlists,
    LastScreen,
}

impl Default for StartupScreen {
    fn default() -> Self {
        Self::Home
    }
}

impl StartupScreen {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Home, InterfaceLanguage::English) => "Home",
            (Self::Library, InterfaceLanguage::English) => "Library",
            (Self::Playlists, InterfaceLanguage::English) => "Playlists",
            (Self::LastScreen, InterfaceLanguage::English) => "Last screen",
            (Self::Home, _) => "Accueil",
            (Self::Library, _) => "Bibliothèque",
            (Self::Playlists, _) => "Playlists",
            (Self::LastScreen, _) => "Dernier écran",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseBehavior {
    Quit,
    MinimizeToTray,
}

impl Default for CloseBehavior {
    fn default() -> Self {
        Self::Quit
    }
}

impl CloseBehavior {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Quit, InterfaceLanguage::English) => "Quit",
            (Self::MinimizeToTray, InterfaceLanguage::English) => "Minimize to tray",
            (Self::Quit, _) => "Quitter",
            (Self::MinimizeToTray, _) => "Réduire dans la barre",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceLanguage {
    System,
    French,
    English,
}

impl Default for InterfaceLanguage {
    fn default() -> Self {
        Self::System
    }
}

impl InterfaceLanguage {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::System, InterfaceLanguage::English) => "System (auto)",
            (Self::French, InterfaceLanguage::English) => "French",
            (Self::English, InterfaceLanguage::English) => "English",
            (Self::System, _) => "Auto (système)",
            (Self::French, _) => "Français",
            (Self::English, _) => "Anglais",
        }
    }

    pub fn resolved(self) -> Self {
        if self != Self::System {
            return self;
        }
        system_language().unwrap_or(Self::French)
    }

    pub fn all() -> &'static [Self; 3] {
        &[Self::System, Self::French, Self::English]
    }
}

fn system_language() -> Option<InterfaceLanguage> {
    let candidates = ["LC_ALL", "LC_MESSAGES", "LANG"];
    for key in candidates {
        if let Ok(value) = env::var(key) {
            if let Some(language) = parse_language_hint(&value) {
                return Some(language);
            }
        }
    }
    None
}

fn parse_language_hint(value: &str) -> Option<InterfaceLanguage> {
    let normalized = value.trim().to_lowercase();
    let language_tag = normalized
        .split('.')
        .next()
        .unwrap_or(&normalized)
        .split('@')
        .next()
        .unwrap_or(&normalized)
        .split('_')
        .next()
        .unwrap_or(&normalized);
    match language_tag {
        "en" => Some(InterfaceLanguage::English),
        "fr" => Some(InterfaceLanguage::French),
        _ => None,
    }
}

impl std::fmt::Display for InterfaceLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label(InterfaceLanguage::English))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeFormat {
    H24,
    H12,
}

impl Default for TimeFormat {
    fn default() -> Self {
        Self::H24
    }
}

impl TimeFormat {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::H24, InterfaceLanguage::English) => "24h",
            (Self::H12, InterfaceLanguage::English) => "12h",
            (Self::H24, _) => "24h",
            (Self::H12, _) => "12h",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        Self::Stable
    }
}

impl UpdateChannel {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Stable, InterfaceLanguage::English) => "Stable",
            (Self::Beta, InterfaceLanguage::English) => "Beta",
            (Self::Stable, _) => "Stable",
            (Self::Beta, _) => "Bêta",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioOutputDevice {
    System,
    UsbHeadset,
}

impl Default for AudioOutputDevice {
    fn default() -> Self {
        Self::System
    }
}

impl AudioOutputDevice {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::System, InterfaceLanguage::English) => "System (default)",
            (Self::UsbHeadset, InterfaceLanguage::English) => "USB headset",
            (Self::System, _) => "Système (par défaut)",
            (Self::UsbHeadset, _) => "Casque USB",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingDeviceBehavior {
    SwitchToSystem,
    PausePlayback,
}

impl Default for MissingDeviceBehavior {
    fn default() -> Self {
        Self::SwitchToSystem
    }
}

impl MissingDeviceBehavior {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::SwitchToSystem, InterfaceLanguage::English) => "Switch to system",
            (Self::PausePlayback, InterfaceLanguage::English) => "Pause playback",
            (Self::SwitchToSystem, _) => "Basculer vers Système",
            (Self::PausePlayback, _) => "Mettre en pause",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeLevel {
    Quiet,
    Normal,
    Loud,
}

impl Default for VolumeLevel {
    fn default() -> Self {
        Self::Normal
    }
}

impl VolumeLevel {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Quiet, InterfaceLanguage::English) => "Quiet",
            (Self::Normal, InterfaceLanguage::English) => "Normal",
            (Self::Loud, InterfaceLanguage::English) => "Loud",
            (Self::Quiet, _) => "Faible",
            (Self::Normal, _) => "Normal",
            (Self::Loud, _) => "Fort",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqPreset {
    Flat,
    Bass,
    Treble,
    Vocal,
    Custom,
}

impl Default for EqPreset {
    fn default() -> Self {
        Self::Flat
    }
}

impl EqPreset {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Flat, InterfaceLanguage::English) => "Flat",
            (Self::Bass, InterfaceLanguage::English) => "Bass",
            (Self::Treble, InterfaceLanguage::English) => "Treble",
            (Self::Vocal, InterfaceLanguage::English) => "Vocal",
            (Self::Custom, InterfaceLanguage::English) => "Custom…",
            (Self::Flat, _) => "Plat",
            (Self::Bass, _) => "Bass",
            (Self::Treble, _) => "Aigus",
            (Self::Vocal, _) => "Voix",
            (Self::Custom, _) => "Personnalisé…",
        }
    }

    pub fn apply_to_model(self, model: &mut EqModel) {
        let normalized = model.clone().normalized();
        let gains = preset_gains(normalized.band_count, self);
        let mut next = normalized;
        for (band, gain) in next.bands.iter_mut().zip(gains) {
            band.gain_db = gain;
        }
        *model = next;
    }
}

fn preset_gains(band_count: crate::eq::EqBandCount, preset: EqPreset) -> Vec<f32> {
    match (band_count, preset) {
        (_, EqPreset::Custom) => vec![],
        (crate::eq::EqBandCount::Three, EqPreset::Flat) => vec![0.0, 0.0, 0.0],
        (crate::eq::EqBandCount::Three, EqPreset::Bass) => vec![4.5, 1.5, -1.0],
        (crate::eq::EqBandCount::Three, EqPreset::Treble) => vec![-1.0, 1.0, 4.0],
        (crate::eq::EqBandCount::Three, EqPreset::Vocal) => vec![-1.5, 3.0, 1.0],
        (crate::eq::EqBandCount::Five, EqPreset::Flat) => vec![0.0, 0.0, 0.0, 0.0, 0.0],
        (crate::eq::EqBandCount::Five, EqPreset::Bass) => vec![5.0, 3.0, 0.5, -1.5, -2.5],
        (crate::eq::EqBandCount::Five, EqPreset::Treble) => vec![-2.0, -0.5, 1.5, 3.5, 4.5],
        (crate::eq::EqBandCount::Five, EqPreset::Vocal) => vec![-1.5, 1.0, 4.0, 2.0, -1.0],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioStabilityMode {
    Auto,
    Stable,
    LowLatency,
}

impl Default for AudioStabilityMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl AudioStabilityMode {
    pub fn label(self, language: InterfaceLanguage) -> &'static str {
        match (self, language) {
            (Self::Auto, InterfaceLanguage::English) => "Auto",
            (Self::Stable, InterfaceLanguage::English) => "Stable",
            (Self::LowLatency, InterfaceLanguage::English) => "Low-latency",
            (Self::Auto, _) => "Auto",
            (Self::Stable, _) => "Stable",
            (Self::LowLatency, _) => "Faible latence",
        }
    }
}

/// All user-configurable settings, persisted as JSON on disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UserSettings {
    pub theme_mode: ThemeMode,
    pub follow_system_theme: bool,
    pub accent_color: AccentColor,
    pub accent_auto: bool,
    pub text_scale: TextScale,
    pub interface_density: InterfaceDensity,
    pub transparency_blur: bool,
    pub ui_animations: bool,
    pub accessibility_large_text: bool,
    pub accessibility_high_contrast: bool,
    pub accessibility_reduce_motion: bool,
    pub increase_contrast: bool,
    pub reduce_transparency: bool,
    pub accessible_text_size: AccessibleTextSize,
    pub reduce_animations: bool,
    pub reduce_transitions: bool,
    pub highlight_keyboard_focus: bool,
    pub enable_advanced_shortcuts: bool,
    pub default_playback_speed: u8,
    pub pause_on_focus_loss: bool,
    pub default_volume: u8,
    pub output_device: AudioOutputDevice,
    pub output_sample_rate_hz: Option<u32>,
    pub missing_device_behavior: MissingDeviceBehavior,
    pub gapless_playback: bool,
    pub crossfade_seconds: u8,
    pub automix_enabled: bool,
    pub normalize_volume: bool,
    pub volume_level: VolumeLevel,
    pub eq_enabled: bool,
    pub eq_preset: EqPreset,
    pub eq_model: EqModel,
    pub audio_stability_mode: AudioStabilityMode,
    pub audio_debug_logs: bool,
    pub launch_at_startup: bool,
    pub restore_last_session: bool,
    pub open_on: StartupScreen,
    pub close_behavior: CloseBehavior,
    pub interface_language: InterfaceLanguage,
    pub time_format: TimeFormat,
    pub auto_check_updates: bool,
    pub update_channel: UpdateChannel,
    pub auto_install_updates: bool,
    pub library_folder: String,
    pub auto_scan_on_launch: bool,
    pub cache_path: String,
    pub notifications_enabled: bool,
    pub now_playing_notifications: bool,
    pub system_tray_enabled: bool,
    pub hardware_acceleration: bool,
    pub limit_cpu_during_playback: bool,
    pub metadata_api_key: String,
    pub metadata_cache_ttl_hours: u32,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Mocha,
            follow_system_theme: false,
            accent_color: AccentColor::default(),
            accent_auto: true,
            text_scale: TextScale::Normal,
            interface_density: InterfaceDensity::default(),
            transparency_blur: true,
            ui_animations: true,
            accessibility_large_text: false,
            accessibility_high_contrast: false,
            accessibility_reduce_motion: false,
            increase_contrast: false,
            reduce_transparency: false,
            accessible_text_size: AccessibleTextSize::default(),
            reduce_animations: false,
            reduce_transitions: false,
            highlight_keyboard_focus: true,
            enable_advanced_shortcuts: false,
            default_playback_speed: 10,
            pause_on_focus_loss: false,
            default_volume: 72,
            output_device: AudioOutputDevice::default(),
            output_sample_rate_hz: None,
            missing_device_behavior: MissingDeviceBehavior::default(),
            gapless_playback: true,
            crossfade_seconds: 4,
            automix_enabled: false,
            normalize_volume: true,
            volume_level: VolumeLevel::default(),
            eq_enabled: false,
            eq_preset: EqPreset::default(),
            eq_model: EqModel::default(),
            audio_stability_mode: AudioStabilityMode::default(),
            audio_debug_logs: false,
            launch_at_startup: false,
            restore_last_session: true,
            open_on: StartupScreen::Home,
            close_behavior: CloseBehavior::Quit,
            interface_language: InterfaceLanguage::System,
            time_format: TimeFormat::H24,
            auto_check_updates: true,
            update_channel: UpdateChannel::Stable,
            auto_install_updates: true,
            library_folder: default_library_folder(),
            auto_scan_on_launch: true,
            cache_path: ".grape_cache".to_string(),
            notifications_enabled: false,
            now_playing_notifications: false,
            system_tray_enabled: false,
            hardware_acceleration: false,
            limit_cpu_during_playback: false,
            metadata_api_key: String::new(),
            metadata_cache_ttl_hours: 24,
        }
    }
}

impl UserSettings {
    /// Returns a copy with all fields clamped to valid ranges and accessibility
    /// flags cascaded to their dependent settings.
    pub fn normalized(mut self) -> Self {
        self.default_volume = self.default_volume.min(100);
        self.crossfade_seconds = self.crossfade_seconds.min(12);
        self.default_playback_speed = self.default_playback_speed.clamp(5, 20);
        if self.accessibility_large_text {
            if self.text_scale == TextScale::Normal {
                self.text_scale = TextScale::Large;
            }
            if self.accessible_text_size == AccessibleTextSize::Standard {
                self.accessible_text_size = AccessibleTextSize::Large;
            }
        }
        if self.accessibility_high_contrast {
            self.increase_contrast = true;
        }
        if self.accessibility_reduce_motion {
            self.reduce_animations = true;
            self.reduce_transitions = true;
        }
        self.accessibility_large_text |= self.text_scale != TextScale::Normal;
        self.accessibility_high_contrast |= self.increase_contrast;
        self.accessibility_reduce_motion |= self.reduce_animations || self.reduce_transitions;
        if let Some(sample_rate) = self.output_sample_rate_hz {
            if !(8_000..=192_000).contains(&sample_rate) {
                self.output_sample_rate_hz = None;
            }
        }
        self.eq_model = self.eq_model.normalized().clamp_gains(-12.0, 12.0);
        if self.library_folder.trim().is_empty() {
            self.library_folder = default_library_folder();
        }
        if self.cache_path.trim().is_empty() {
            self.cache_path = ".grape_cache".to_string();
        }
        let cache = std::path::PathBuf::from(&self.cache_path);
        if !cache.is_absolute() {
            for component in cache.components() {
                if matches!(component, std::path::Component::ParentDir) {
                    self.cache_path = ".grape_cache".to_string();
                    break;
                }
            }
        }
        if !self.notifications_enabled {
            self.now_playing_notifications = false;
        }
        if self.metadata_cache_ttl_hours > 24 * 365 {
            self.metadata_cache_ttl_hours = 24 * 365;
        }
        self
    }

    pub fn wants_system_integration(&self) -> bool {
        let wants_hardware_acceleration = self.hardware_acceleration
            && self.ui_animations
            && !self.reduce_animations
            && !self.reduce_transitions;
        self.notifications_enabled
            || self.now_playing_notifications
            || self.system_tray_enabled
            || self.enable_advanced_shortcuts
            || wants_hardware_acceleration
    }
}

fn default_library_folder() -> String {
    if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join("Music").to_string_lossy().to_string()
    } else if let Ok(profile) = env::var("USERPROFILE") {
        PathBuf::from(profile).join("Music").to_string_lossy().to_string()
    } else {
        ".".to_string()
    }
}

pub fn config_root() -> PathBuf {
    if cfg!(windows) {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data).join("Colony").join("Grape")
        } else if let Ok(profile) = env::var("USERPROFILE") {
            PathBuf::from(profile)
                .join("AppData")
                .join("Local")
                .join("Colony")
                .join("Grape")
        } else {
            PathBuf::from(".")
        }
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".config").join("Colony").join("Grape")
    } else if let Ok(profile) = env::var("USERPROFILE") {
        PathBuf::from(profile).join(".config").join("Colony").join("Grape")
    } else {
        PathBuf::from(".")
    }
}

fn settings_path() -> PathBuf {
    config_root().join("preferences.json")
}

fn history_path() -> PathBuf {
    config_root().join("history.json")
}

fn logs_dir() -> PathBuf {
    config_root().join("logs")
}

pub fn library_cache_dir(settings: &UserSettings, root: &Path) -> PathBuf {
    let path = PathBuf::from(&settings.cache_path);
    if path.is_absolute() { path } else { root.join(path) }
}

pub fn ensure_logs_dir() -> io::Result<PathBuf> {
    let path = logs_dir();
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

pub fn clear_history() -> io::Result<()> {
    let path = history_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn clear_library_cache(settings: &UserSettings, root: &Path) -> io::Result<()> {
    let path = library_cache_dir(settings, root);
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub fn load_settings() -> UserSettings {
    let path = settings_path();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return UserSettings::default();
        }
        Err(err) => {
            warn!(error = %err, path = %path.display(), "Failed to read preferences");
            return UserSettings::default();
        }
    };

    match serde_json::from_str::<UserSettings>(&contents) {
        Ok(settings) => settings.normalized(),
        Err(err) => {
            warn!(error = %err, path = %path.display(), "Failed to parse preferences");
            UserSettings::default()
        }
    }
}

pub fn save_settings(settings: &UserSettings) -> io::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(settings)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    atomic_write(&path, payload.as_bytes())
}

/// Writes `data` to a temporary file in the same directory as `path`, then
/// atomically renames it into place. This prevents corruption if the process
/// is interrupted mid-write.
fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp_path =
        parent.join(format!(".{}.tmp", path.file_name().unwrap_or_default().to_string_lossy()));
    fs::write(&tmp_path, data)?;
    fs::rename(&tmp_path, path)
}

// --- Session state persistence ---

fn session_path() -> PathBuf {
    config_root().join("session.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub track_path: Option<PathBuf>,
    pub position_secs: f64,
    pub active_tab: String,
    pub queue_index: usize,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            track_path: None,
            position_secs: 0.0,
            active_tab: "artists".to_string(),
            queue_index: 0,
        }
    }
}

pub fn save_session(session: &SessionState) -> io::Result<()> {
    let path = session_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload =
        serde_json::to_string(session).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    atomic_write(&path, payload.as_bytes())
}

pub fn load_session() -> Option<SessionState> {
    let path = session_path();
    let contents = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&contents).ok()
}

/// Detect whether the system prefers dark mode.
/// Tries `gsettings` on Linux, falls back to assuming dark.
pub fn system_prefers_dark() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("prefer-light") {
                return false;
            }
        }
    }
    // Default to dark on non-Linux or when detection fails.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_clamps_volume() {
        let mut settings = UserSettings::default();
        settings.default_volume = 200;
        let normalized = settings.normalized();
        assert_eq!(normalized.default_volume, 100);
    }

    #[test]
    fn normalized_clamps_crossfade() {
        let mut settings = UserSettings::default();
        settings.crossfade_seconds = 50;
        let normalized = settings.normalized();
        assert_eq!(normalized.crossfade_seconds, 12);
    }

    #[test]
    fn normalized_clamps_playback_speed() {
        let mut settings = UserSettings::default();
        settings.default_playback_speed = 1;
        let normalized = settings.normalized();
        assert_eq!(normalized.default_playback_speed, 5);

        let mut settings = UserSettings::default();
        settings.default_playback_speed = 100;
        let normalized = settings.normalized();
        assert_eq!(normalized.default_playback_speed, 20);
    }

    #[test]
    fn normalized_cascades_accessibility_large_text() {
        let mut settings = UserSettings::default();
        settings.accessibility_large_text = true;
        let normalized = settings.normalized();
        assert_eq!(normalized.text_scale, TextScale::Large);
        assert_eq!(normalized.accessible_text_size, AccessibleTextSize::Large);
    }

    #[test]
    fn normalized_cascades_accessibility_high_contrast() {
        let mut settings = UserSettings::default();
        settings.accessibility_high_contrast = true;
        let normalized = settings.normalized();
        assert!(normalized.increase_contrast);
    }

    #[test]
    fn normalized_cascades_accessibility_reduce_motion() {
        let mut settings = UserSettings::default();
        settings.accessibility_reduce_motion = true;
        let normalized = settings.normalized();
        assert!(normalized.reduce_animations);
        assert!(normalized.reduce_transitions);
    }

    #[test]
    fn normalized_resets_traversal_cache_path() {
        let mut settings = UserSettings::default();
        settings.cache_path = "../escape".to_string();
        let normalized = settings.normalized();
        assert_eq!(normalized.cache_path, ".grape_cache");
    }

    #[test]
    fn normalized_keeps_valid_cache_path() {
        let mut settings = UserSettings::default();
        settings.cache_path = "my_cache".to_string();
        let normalized = settings.normalized();
        assert_eq!(normalized.cache_path, "my_cache");
    }

    #[test]
    fn normalized_rejects_out_of_range_sample_rate() {
        let mut settings = UserSettings::default();
        settings.output_sample_rate_hz = Some(500_000);
        let normalized = settings.normalized();
        assert_eq!(normalized.output_sample_rate_hz, None);
    }

    #[test]
    fn normalized_disables_now_playing_when_notifications_off() {
        let mut settings = UserSettings::default();
        settings.notifications_enabled = false;
        settings.now_playing_notifications = true;
        let normalized = settings.normalized();
        assert!(!normalized.now_playing_notifications);
    }

    #[test]
    fn normalized_caps_metadata_ttl() {
        let mut settings = UserSettings::default();
        settings.metadata_cache_ttl_hours = u32::MAX;
        let normalized = settings.normalized();
        assert_eq!(normalized.metadata_cache_ttl_hours, 24 * 365);
    }

    #[test]
    fn default_settings_roundtrip_through_json() {
        let settings = UserSettings::default();
        let json = serde_json::to_string(&settings).expect("serialize");
        let deserialized: UserSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(settings, deserialized);
    }
}
