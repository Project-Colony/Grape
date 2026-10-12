use colony_ui::preferences::StandardPreferences;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use tracing::warn;

use crate::eq::EqModel;

mod migrate;

/// Grape's default theme, kept as it was rather than inheriting Colony's
/// `FALLBACK_PALETTE` (gruvbox/dark), so an upgrade does not restyle anyone.
pub const DEFAULT_THEME_FAMILY: &str = "catppuccin";
pub const DEFAULT_THEME_VARIANT: &str = "mocha";

/// Maps the flat `ThemeMode` enum Grape persisted before it adopted Colony's
/// shared palettes onto the (family, variant) pair that replaced it.
///
/// The last three entries are the serde aliases the old enum carried, which is
/// what a pre-0.2 preferences file actually contains on disk. "System" was an
/// alias of Mocha, not a follow-the-system flag -- that behaviour is the
/// separate `follow_system_theme` boolean.
fn theme_from_legacy_mode(mode: &str) -> (&'static str, &'static str) {
    match mode {
        "Latte" | "Light" => ("catppuccin", "latte"),
        "Frappe" => ("catppuccin", "frappe"),
        "Macchiato" => ("catppuccin", "macchiato"),
        "GruvboxLight" => ("gruvbox", "light"),
        "GruvboxDark" => ("gruvbox", "dark"),
        "EverblushLight" => ("everblush", "light"),
        "EverblushDark" => ("everblush", "dark"),
        "KanagawaLight" => ("kanagawa", "light"),
        "KanagawaDark" => ("kanagawa", "dark"),
        "KanagawaJournal" => ("kanagawa", "journal"),
        // "Mocha", "Dark", "System", anything unrecognised.
        _ => (DEFAULT_THEME_FAMILY, DEFAULT_THEME_VARIANT),
    }
}

/// The counterpart variant of the same family in the other mode, for
/// `follow_system_theme`.
///
/// Reads Colony's catalog rather than a hand-written table, so a family added
/// upstream follows the system without an edit here.
pub fn counterpart_variant(family: &str, variant: &str, want_light: bool) -> String {
    let Some(meta) = colony_ui::theme::family(family) else {
        return variant.to_string();
    };
    if meta
        .variant(variant)
        .is_some_and(|current| current.is_light() == want_light)
    {
        return variant.to_string();
    }
    meta.variants
        .iter()
        .find(|candidate| candidate.is_light() == want_light)
        .map_or_else(|| variant.to_string(), |found| found.key.to_string())
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

    /// The language the interface is drawn in. `System` follows the
    /// operating system, read once per process, and falls back to English
    /// when the system asks for a language Grape does not ship.
    pub fn resolved(self) -> Self {
        static SYSTEM: std::sync::OnceLock<InterfaceLanguage> = std::sync::OnceLock::new();
        if self != Self::System {
            return self;
        }
        *SYSTEM.get_or_init(|| first_supported(system_language_hints()))
    }

    /// The matching locale of the shared Colony labels.
    pub fn locale(self) -> colony_ui::i18n::Locale {
        match self.resolved() {
            Self::French => colony_ui::i18n::Locale::Fr,
            _ => colony_ui::i18n::Locale::En,
        }
    }

    pub fn all() -> &'static [Self; 3] {
        &[Self::System, Self::French, Self::English]
    }
}

/// The system's language preferences, most preferred first.
///
/// On Unix the POSIX variables come first, each one on its own: an
/// unparsable `LC_ALL=C` must not hide a French `LANG`. Then `sys-locale`,
/// which reads the user's UI languages on Windows and macOS.
fn system_language_hints() -> impl Iterator<Item = String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .filter(|_| cfg!(unix))
        .filter_map(|key| env::var(key).ok())
        .chain(sys_locale::get_locales())
}

/// The first hint naming a language Grape ships, or English.
fn first_supported(hints: impl IntoIterator<Item = String>) -> InterfaceLanguage {
    hints
        .into_iter()
        .find_map(|hint| parse_language_hint(&hint))
        .unwrap_or(InterfaceLanguage::English)
}

/// Reads the language out of a POSIX locale (`fr_FR.UTF-8@euro`) or a BCP 47
/// tag (`fr-CA`).
fn parse_language_hint(value: &str) -> Option<InterfaceLanguage> {
    let language = value.trim().split(['_', '-', '.', '@']).next()?;
    if language.eq_ignore_ascii_case("en") {
        Some(InterfaceLanguage::English)
    } else if language.eq_ignore_ascii_case("fr") {
        Some(InterfaceLanguage::French)
    } else {
        None
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
    /// What every Colony program configures in Appearance and Accessibility:
    /// theme, accent, both text sizes, animations, high contrast, the dyslexia
    /// font and reduced motion. Flattened, so the keys sit at the top level of
    /// `preferences.json` under the names Colony itself writes. Grape 0.4 and
    /// earlier used other names; [`UserSettings::from_json`] renames them.
    #[serde(flatten)]
    pub standard: StandardPreferences,
    pub follow_system_theme: bool,
    pub interface_density: InterfaceDensity,
    pub transparency_blur: bool,
    pub reduce_transparency: bool,
    pub highlight_keyboard_focus: bool,
    pub enable_advanced_shortcuts: bool,
    pub default_playback_speed: u8,
    pub pause_on_focus_loss: bool,
    pub default_volume: u8,
    pub output_device: AudioOutputDevice,
    pub output_sample_rate_hz: Option<u32>,
    pub missing_device_behavior: MissingDeviceBehavior,
    pub gapless_playback: bool,
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
            standard: StandardPreferences {
                selected_theme: DEFAULT_THEME_FAMILY.to_string(),
                selected_variant: DEFAULT_THEME_VARIANT.to_string(),
                ..StandardPreferences::default()
            },
            follow_system_theme: false,
            interface_density: InterfaceDensity::default(),
            transparency_blur: true,
            reduce_transparency: false,
            highlight_keyboard_focus: true,
            enable_advanced_shortcuts: false,
            default_playback_speed: 10,
            pause_on_focus_loss: false,
            default_volume: 72,
            output_device: AudioOutputDevice::default(),
            output_sample_rate_hz: None,
            missing_device_behavior: MissingDeviceBehavior::default(),
            gapless_playback: true,
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
            library_folder: default_library_folder(),
            auto_scan_on_launch: true,
            // Empty means the Colony cache root. It used to default to
            // ".grape_cache", which put the cache inside the music folder.
            cache_path: String::new(),
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
    /// Reads a `preferences.json`, upgrading the keys older releases wrote,
    /// and returns it normalized.
    pub fn from_json(contents: &str) -> serde_json::Result<Self> {
        let mut value: Value = serde_json::from_str(contents)?;
        if let Value::Object(map) = &mut value {
            upgrade_legacy_keys(map);
        }
        Ok(serde_json::from_value::<Self>(value)?.normalized())
    }

    /// Whether anything may animate: the Appearance switch, silenced by
    /// Accessibility's reduced motion whatever its value.
    pub const fn animations_enabled(&self) -> bool {
        self.standard.animations && !self.standard.reduce_motion
    }

    /// Returns a copy with all fields clamped to valid ranges.
    pub fn normalized(mut self) -> Self {
        // An unknown family survives a downgrade or a hand-edited file; fall
        // back rather than rendering an unstyled window.
        if colony_ui::theme::family(&self.standard.selected_theme).is_none() {
            self.standard.selected_theme = DEFAULT_THEME_FAMILY.to_string();
            self.standard.selected_variant = DEFAULT_THEME_VARIANT.to_string();
        }
        self.default_volume = self.default_volume.min(100);
        self.default_playback_speed = self.default_playback_speed.clamp(5, 20);
        if let Some(sample_rate) = self.output_sample_rate_hz {
            if !(8_000..=192_000).contains(&sample_rate) {
                self.output_sample_rate_hz = None;
            }
        }
        self.eq_model = self.eq_model.normalized().clamp_gains(-12.0, 12.0);
        if self.library_folder.trim().is_empty() {
            self.library_folder = default_library_folder();
        }
        // Empty is the default and means the Colony cache root; only a value
        // the user actually typed is validated. A `..` component would let the
        // cache escape the library it is resolved against, so it falls back to
        // the default rather than to the in-library location it once did. The
        // checks that need the Colony roots run where the path is used.
        if cache_path_has_parent_dir(&self.cache_path) {
            self.cache_path = String::new();
        }
        // Everyone upgrading carries the old default explicitly, which would
        // pin them to the music folder forever. Only the exact old default is
        // cleared -- a path the user chose is theirs to keep.
        if self.cache_path.trim() == ".grape_cache" {
            self.cache_path = String::new();
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
        let wants_hardware_acceleration = self.hardware_acceleration && self.animations_enabled();
        self.notifications_enabled
            || self.now_playing_notifications
            || self.system_tray_enabled
            || self.enable_advanced_shortcuts
            || wants_hardware_acceleration
    }
}

/// Renames, in place, the keys Grape wrote before it embedded Colony's
/// [`StandardPreferences`], so nobody loses a choice on upgrade.
///
/// A key already in the new form wins, so a file this release wrote passes
/// through untouched even with an old key left next to it. The old keys
/// themselves are ignored by the parse and dropped on the next save.
///
/// The old text sizes land on the nearest Colony step: 1.1 and 1.25 on
/// Appearance's large (1.2); Accessibility keeps its names, so extra large
/// becomes xlarge (1.4). The old "auto accent", on by default, meant the
/// theme's own accent, which is what no picked accent means now, so only an
/// accent the user actually picked is carried over. The three old motion
/// switches and the two contrast switches each collapse into the one Colony
/// keeps.
fn upgrade_legacy_keys(map: &mut Map<String, Value>) {
    let text = |key: &str| map.get(key).and_then(Value::as_str);
    let flag = |key: &str| map.get(key).and_then(Value::as_bool);
    let any_on = |keys: &[&str]| {
        let found: Vec<bool> = keys.iter().filter_map(|&key| flag(key)).collect();
        (!found.is_empty()).then(|| found.contains(&true))
    };

    // The pair moves together, or a file with only one of them would mix two
    // themes. Before 0.2 Grape stored one flat `theme_mode` instead.
    let theme = match (text("theme_family"), text("theme_mode")) {
        (Some(family), _) if !family.is_empty() => {
            (family.to_owned(), text("theme_variant").unwrap_or_default().to_owned())
        }
        (_, Some(mode)) => {
            let (family, variant) = theme_from_legacy_mode(mode);
            (family.to_owned(), variant.to_owned())
        }
        _ => (DEFAULT_THEME_FAMILY.to_owned(), DEFAULT_THEME_VARIANT.to_owned()),
    };
    let theme = (!map.contains_key("selected_theme")).then_some(theme);
    let accent = if flag("accent_auto") == Some(false) {
        text("accent_color").map(str::to_lowercase)
    } else {
        None
    };
    let font_size = text("text_scale").map(|old| match old {
        "Normal" => "default",
        _ => "large",
    });
    let text_size = text("accessible_text_size").map(|old| match old {
        "Large" => "large",
        "ExtraLarge" => "xlarge",
        _ => "default",
    });
    let upgraded = [
        ("selected_accent", accent.map(Value::from)),
        ("font_size", font_size.map(Value::from)),
        ("text_size_a11y", text_size.map(Value::from)),
        ("animations", flag("ui_animations").map(Value::from)),
        (
            "high_contrast",
            any_on(&["increase_contrast", "accessibility_high_contrast"]).map(Value::from),
        ),
        ("dyslexia_font", flag("accessibility_dyslexia_font").map(Value::from)),
        (
            "reduce_motion",
            any_on(&["accessibility_reduce_motion", "reduce_animations", "reduce_transitions"])
                .map(Value::from),
        ),
    ];

    if let Some((family, variant)) = theme {
        map.insert("selected_theme".to_owned(), Value::from(family));
        map.insert("selected_variant".to_owned(), Value::from(variant));
    }
    for (key, value) in upgraded {
        if let Some(value) = value {
            map.entry(key).or_insert(value);
        }
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

/// The display name Colony nests this program's directories under.
#[cfg(not(test))]
const PROGRAM: &str = "Grape";

/// The three Colony roots, resolved once per process.
///
/// `colony_ui::paths::*` are fallible and create the directory as a side
/// effect; every accessor below predates that and is infallible, because a
/// program that cannot resolve a home directory should still run against a
/// working path rather than refuse to start. The fallback is the layout Grape
/// shipped before Colony, which is also what [`migrate`] reads from.
pub struct Roots {
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
    /// Where Grape kept everything before; see [`legacy_config_root`].
    legacy: PathBuf,
    /// Whether every file is at its current path. Until it is, each one is
    /// read and written wherever `migrate::resolve` finds it.
    migrated: bool,
}

impl Roots {
    /// Every root under one base, with nothing migrated yet.
    #[cfg(test)]
    pub(crate) fn under(base: &Path) -> Self {
        Self {
            config: base.join("config"),
            data: base.join("data"),
            cache: base.join("cache"),
            legacy: base.join("legacy"),
            migrated: false,
        }
    }
}

/// Resolves the roots and, on first call, migrates the pre-Colony layout.
pub fn roots() -> &'static Roots {
    static ROOTS: std::sync::OnceLock<Roots> = std::sync::OnceLock::new();
    ROOTS.get_or_init(resolve_roots)
}

#[cfg(not(test))]
fn resolve_roots() -> Roots {
    let mut roots = Roots {
        config: colony_ui::paths::config_dir(PROGRAM).unwrap_or_else(|err| {
            warn!(error = %err, "Falling back to the pre-Colony config directory");
            legacy_config_root()
        }),
        data: colony_ui::paths::data_dir(PROGRAM).unwrap_or_else(|err| {
            warn!(error = %err, "Falling back to the pre-Colony data directory");
            legacy_config_root()
        }),
        cache: colony_ui::paths::cache_dir(PROGRAM).unwrap_or_else(|err| {
            warn!(error = %err, "Falling back to the pre-Colony cache directory");
            legacy_config_root().join("cache")
        }),
        legacy: legacy_config_root(),
        migrated: false,
    };
    roots.migrated = migrate::run(&roots);
    roots
}

/// The unit tests share one process, so the first test to reach a Colony path
/// would otherwise resolve, and migrate, the real profile for all of them.
/// They get one throwaway base per process instead.
#[cfg(test)]
fn resolve_roots() -> Roots {
    let base = env::temp_dir().join(format!("grape-tests-{}", std::process::id()));
    let roots = Roots::under(&base);
    for dir in [&roots.config, &roots.data, &roots.cache] {
        fs::create_dir_all(dir).expect("test config root");
    }
    roots
}

/// Where Grape kept everything before it adopted the Colony layout.
///
/// Retained verbatim because the migration has to read the old location, and
/// because it is the degraded path when a Colony root cannot be resolved. On
/// macOS this is `~/.config`, which was wrong -- the layout puts macOS config
/// under `~/Library/Application Support` -- and on Linux it reads `HOME`
/// directly, ignoring `XDG_CONFIG_HOME`.
#[cfg(not(test))]
fn legacy_config_root() -> PathBuf {
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

/// Where per-album metadata the *user* typed is kept.
///
/// Deliberately in `config`, not in the cache: these are the user's own edits,
/// they cannot be re-derived by asking again, and "Clear cache" must not touch
/// them. They used to live inside the cache directory, which meant clearing the
/// cache silently destroyed them.
pub fn metadata_overrides_dir() -> PathBuf {
    migrate::resolve(roots(), migrate::METADATA_OVERRIDES)
}

fn settings_path() -> PathBuf {
    migrate::resolve(roots(), migrate::PREFERENCES)
}

pub fn playlist_path() -> PathBuf {
    migrate::resolve(roots(), migrate::PLAYLISTS)
}

fn logs_dir() -> PathBuf {
    migrate::resolve(roots(), migrate::LOGS)
}

/// M3U playlists exported for the user to take away: program output, so
/// `data`, not beside their preferences.
pub fn exports_dir() -> PathBuf {
    migrate::resolve(roots(), migrate::EXPORTS)
}

/// Where the scanned-library cache for `root` is kept.
///
/// An empty `cache_path` -- the default -- means the Colony cache root,
/// namespaced per library so two libraries never share an entry: cache keys are
/// paths *relative* to the library root, so `Album/01.mp3` from two folders
/// would otherwise collide.
///
/// A non-empty `cache_path` is the user overriding that: absolute is taken as
/// given, relative is resolved against the library, which is where Grape used
/// to put the cache unconditionally. Grape keeps its files in a
/// [`CUSTOM_CACHE_DIRNAME`] folder inside that location, never directly in it.
/// An override [`custom_cache_dir`] refuses is ignored, with a warning, in
/// favour of the default.
///
/// The check runs here, at the point of use, so a value typed during the
/// session is covered as much as one read from disk.
pub fn library_cache_dir(settings: &UserSettings, root: &Path) -> PathBuf {
    resolve_library_cache_dir(&settings.cache_path, root, roots(), home_dir().as_deref())
}

fn resolve_library_cache_dir(
    cache_path: &str,
    root: &Path,
    roots: &Roots,
    home: Option<&Path>,
) -> PathBuf {
    let configured = cache_path.trim();
    if !configured.is_empty() {
        match custom_cache_dir(configured, root, roots, home) {
            Ok(dir) => return dir,
            Err(reason) => warn!(
                cache_path = configured,
                reason, "Ignoring the custom cache location; using the default one"
            ),
        }
    }
    roots.cache.join("libraries").join(crate::library::cache::library_key(root))
}

/// The folder Grape creates inside a custom cache location and keeps its files
/// in.
///
/// The entry names are generic (`covers/`, `metadata/`, `index.json`), and both
/// *Clear cache* and the end-of-scan prune delete inside them. Written straight
/// into a folder the user chose, they could land on folders of the user's own
/// with the same names.
const CUSTOM_CACHE_DIRNAME: &str = "grape-cache";

/// Resolves a cache location the user typed, or says why it cannot be used.
///
/// *Clear cache* deletes inside this directory, and nothing stops the user
/// from typing a folder full of their own files. So a location that is, or
/// holds, something Grape must never delete from is refused: the filesystem
/// root, the home folder, the library, and Grape's own config and data roots.
/// A `..` component is refused before anything is resolved. Paths are compared
/// as written, then again with symlinks resolved when both exist. An accepted
/// location gets Grape's own [`CUSTOM_CACHE_DIRNAME`] sub-folder.
fn custom_cache_dir(
    configured: &str,
    root: &Path,
    roots: &Roots,
    home: Option<&Path>,
) -> Result<PathBuf, &'static str> {
    if cache_path_has_parent_dir(configured) {
        return Err("it contains '..'");
    }
    // An absolute path replaces `root` in the join. Collecting the components
    // drops every `.`, so `.` resolves to the library itself.
    let dir: PathBuf = root.join(configured).components().collect();
    if !dir.components().any(|c| matches!(c, Component::Normal(_))) {
        return Err("it is empty or the filesystem root");
    }
    for (protected, reason) in [
        (Some(root), "it is or holds the library folder"),
        (home, "it is or holds the home folder"),
        (Some(roots.config.as_path()), "it is or holds Grape's config folder"),
        (Some(roots.data.as_path()), "it is or holds Grape's data folder"),
    ] {
        if protected.is_some_and(|protected| is_same_or_ancestor(&dir, protected)) {
            return Err(reason);
        }
    }
    Ok(dir.join(CUSTOM_CACHE_DIRNAME))
}

/// Whether `dir` is `path` itself or one of its ancestors.
fn is_same_or_ancestor(dir: &Path, path: &Path) -> bool {
    let lexical: PathBuf = path.components().collect();
    lexical.starts_with(dir)
        || matches!(
            (dir.canonicalize(), path.canonicalize()),
            (Ok(dir), Ok(path)) if path.starts_with(&dir)
        )
}

/// Whether a typed cache location contains a `..` component.
///
/// Refused wherever it appears, absolute paths included: it is how a location
/// that reads like a sub-folder lands on the library's parent, or on home.
pub fn cache_path_has_parent_dir(cache_path: &str) -> bool {
    Path::new(cache_path.trim())
        .components()
        .any(|c| matches!(c, Component::ParentDir))
}

/// The user's home folder, which no cache location may be or hold.
fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
}

/// The cache directory in force, published so `library::cache` can reach it
/// without every function there taking the settings.
///
/// Set at startup and whenever the library folder or `cache_path` changes.
/// Before it is set, `library::cache` degrades to the pre-Colony location,
/// which is also what it did unconditionally before this existed.
static ACTIVE_CACHE_DIR: std::sync::RwLock<Option<PathBuf>> = std::sync::RwLock::new(None);

/// Every library switch goes through here, at startup and from the
/// preferences, so this is also where each library's hand-typed album metadata
/// is rescued from its old in-library cache, before anything reads it.
///
/// The library and cache path fields call this on every keystroke, so each
/// library root is checked once per session. A lift that failed is retried at
/// the next launch.
pub fn set_active_cache_dir(settings: &UserSettings, root: &Path) {
    static CHECKED: std::sync::Mutex<std::collections::BTreeSet<PathBuf>> =
        std::sync::Mutex::new(std::collections::BTreeSet::new());
    if CHECKED.lock().unwrap().insert(root.to_path_buf()) {
        migrate::lift_metadata_overrides(roots(), root);
    }
    *ACTIVE_CACHE_DIR.write().unwrap() = Some(library_cache_dir(settings, root));
}

pub fn active_cache_dir(root: &Path) -> PathBuf {
    ACTIVE_CACHE_DIR
        .read()
        .unwrap()
        .clone()
        .unwrap_or_else(|| root.join(".grape_cache"))
}

pub fn ensure_logs_dir() -> io::Result<PathBuf> {
    let path = logs_dir();
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

pub fn clear_history() -> io::Result<()> {
    // The migration copies rather than moves, and deliberately leaves the
    // originals in place for one release. "Clear local history" has to remove
    // every copy, or it quietly leaves the file it promised to delete.
    for path in migrate::HISTORY.locations(roots()) {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Deletes what Grape keeps in the cache directory `dir`, and only that.
///
/// `dir` can be a folder the user chose, holding files of their own next to
/// the cache, so it is never removed wholesale: Grape's entries go, and the
/// directory itself goes only when that leaves it empty.
pub fn clear_library_cache(dir: &Path) -> io::Result<()> {
    use crate::library::cache::{
        COVER_DIRNAME, FOLDERS_DIRNAME, INDEX_FILENAME, METADATA_DIRNAME, TRACKS_DIRNAME,
    };
    let ignore_missing = |result: io::Result<()>| match result {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
        _ => Ok(()),
    };
    ignore_missing(fs::remove_file(dir.join(INDEX_FILENAME)))?;
    for name in [FOLDERS_DIRNAME, TRACKS_DIRNAME, COVER_DIRNAME, METADATA_DIRNAME] {
        // Does not follow a symlink: a link with an entry's name is removed,
        // not what it points at.
        ignore_missing(fs::remove_dir_all(dir.join(name)))?;
    }
    // Fails, as intended, while anything else is still in there.
    let _ = fs::remove_dir(dir);
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
    // The file holds the Last.fm API key. One written before atomic_write made
    // it owner-only, or hand-edited to add the key, keeps a wider mode until
    // the next save, so it is narrowed here. So are the older copies the
    // migration leaves behind.
    for copy in migrate::PREFERENCES.locations(roots()) {
        restrict_to_owner(&copy);
    }

    match UserSettings::from_json(&contents) {
        Ok(settings) => settings,
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
    let settings = storable_settings(settings, roots(), home_dir().as_deref());
    let payload = serde_json::to_string_pretty(&settings)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    atomic_write(&path, payload.as_bytes())
}

/// `settings` as they are written to disk.
///
/// A cache location refused at the point of use is not written either, so it
/// does not come back on the next launch. The in-memory value is left alone:
/// it is what the text field shows while the user is still typing.
fn storable_settings(
    settings: &UserSettings,
    roots: &Roots,
    home: Option<&Path>,
) -> UserSettings {
    let mut settings = settings.clone();
    let configured = settings.cache_path.trim();
    if !configured.is_empty()
        && custom_cache_dir(configured, Path::new(settings.library_folder.trim()), roots, home)
            .is_err()
    {
        settings.cache_path.clear();
    }
    settings
}

/// Makes `path` readable and writable by its owner only, on Unix. Best effort:
/// a missing file is not an error.
fn restrict_to_owner(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Writes `data` to a temporary file in the same directory as `path`, then
/// atomically renames it into place. This prevents corruption if the process
/// is interrupted mid-write.
///
/// On Unix the file is readable by its owner only: `preferences.json` holds
/// the Last.fm API key.
fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    use std::io::Write;
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp_path =
        parent.join(format!(".{}.tmp", path.file_name().unwrap_or_default().to_string_lossy()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
        // The mode applies only to a file this call creates, so a temp file
        // left behind by an interrupted write must not be reused.
        match fs::remove_file(&tmp_path) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => return Err(err),
            _ => {}
        }
    }
    options.open(&tmp_path)?.write_all(data)?;
    fs::rename(&tmp_path, path)
}

// --- Session state persistence ---

/// The resume point is program-produced state, not a user preference.
fn session_path() -> PathBuf {
    migrate::resolve(roots(), migrate::SESSION)
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
    use colony_ui::{FontSize, TextSize};

    #[test]
    fn the_system_language_falls_back_to_english() {
        let first = |hints: &[&str]| first_supported(hints.iter().map(|hint| (*hint).to_string()));
        assert_eq!(first(&[]), InterfaceLanguage::English);
        assert_eq!(first(&["de_DE.UTF-8", "ja-JP"]), InterfaceLanguage::English);
        // An unparsable LC_ALL must not hide a French LANG.
        assert_eq!(first(&["C", "fr_FR.UTF-8@euro"]), InterfaceLanguage::French);
        // BCP 47, as sys-locale reports it on Windows and macOS.
        assert_eq!(first(&["fr-CA", "en-US"]), InterfaceLanguage::French);
        assert_eq!(first(&["EN-gb"]), InterfaceLanguage::English);
    }

    #[test]
    fn legacy_theme_modes_migrate_to_the_colony_catalog() {
        for (legacy, family, variant) in [
            ("Latte", "catppuccin", "latte"),
            ("Frappe", "catppuccin", "frappe"),
            ("Macchiato", "catppuccin", "macchiato"),
            ("Mocha", "catppuccin", "mocha"),
            ("GruvboxLight", "gruvbox", "light"),
            ("GruvboxDark", "gruvbox", "dark"),
            ("EverblushLight", "everblush", "light"),
            ("EverblushDark", "everblush", "dark"),
            ("KanagawaLight", "kanagawa", "light"),
            ("KanagawaDark", "kanagawa", "dark"),
            ("KanagawaJournal", "kanagawa", "journal"),
            // The serde aliases a pre-0.2 file actually carries.
            ("Light", "catppuccin", "latte"),
            ("Dark", "catppuccin", "mocha"),
            ("System", "catppuccin", "mocha"),
            ("something nobody shipped", "catppuccin", "mocha"),
        ] {
            let raw = format!("{{\"theme_mode\":\"{legacy}\"}}");
            let settings = UserSettings::from_json(&raw).unwrap().standard;
            assert_eq!(settings.selected_theme, family, "family for {legacy}");
            assert_eq!(settings.selected_variant, variant, "variant for {legacy}");
            assert!(
                colony_ui::theme::family(&settings.selected_theme)
                    .and_then(|f| f.variant(&settings.selected_variant))
                    .is_some(),
                "{legacy} must land on a variant that exists in the catalog"
            );
        }
    }

    #[test]
    fn explicit_colony_keys_win_over_a_stale_legacy_mode() {
        let raw = r#"{"theme_mode":"Mocha","theme_family":"nord","theme_variant":"dark"}"#;
        let settings = UserSettings::from_json(raw).unwrap().standard;
        assert_eq!(settings.selected_theme, "nord");
        assert_eq!(settings.selected_variant, "dark");

        // And the current keys win over every older one.
        let raw = r#"{"selected_theme":"gruvbox","selected_variant":"light",
            "theme_family":"nord","theme_variant":"dark","font_size":"small",
            "text_scale":"ExtraLarge","high_contrast":false,"increase_contrast":true}"#;
        let settings = UserSettings::from_json(raw).unwrap().standard;
        assert_eq!(
            (settings.selected_theme.as_str(), settings.selected_variant.as_str()),
            ("gruvbox", "light")
        );
        assert_eq!(settings.font_size, FontSize::Small);
        assert!(!settings.high_contrast);
    }

    /// Everything a 0.4 release wrote for the standard preferences, with the
    /// user away from every default.
    #[test]
    fn a_0_4_file_keeps_every_standard_choice() {
        let raw = r#"{"theme_family":"nord","theme_variant":"dark","follow_system_theme":false,
            "accent_color":"Violet","accent_auto":false,"text_scale":"ExtraLarge",
            "accessible_text_size":"ExtraLarge","ui_animations":false,
            "accessibility_large_text":true,"accessibility_high_contrast":false,
            "increase_contrast":true,"accessibility_dyslexia_font":true,
            "accessibility_reduce_motion":false,"reduce_animations":false,
            "reduce_transitions":true,"default_volume":40}"#;
        let settings = UserSettings::from_json(raw).unwrap();
        assert_eq!(settings.default_volume, 40, "Grape's own keys are untouched");
        assert_eq!(
            settings.standard,
            StandardPreferences {
                selected_theme: "nord".to_string(),
                selected_variant: "dark".to_string(),
                selected_accent: Some("violet".to_string()),
                auto_accent: false,
                font_size: FontSize::Large,
                animations: false,
                high_contrast: true,
                dyslexia_font: true,
                reduce_motion: true,
                text_size_a11y: TextSize::XLarge,
            }
        );
        // The migrated key is a real Colony accent, not a dead override.
        assert!(settings.standard.accent_override().is_some());
    }

    /// The defaults a 0.4 release wrote come back as Colony's defaults, so an
    /// upgrade changes nothing on screen.
    #[test]
    fn a_0_4_file_on_the_defaults_upgrades_to_the_defaults() {
        let raw = r#"{"theme_family":"catppuccin","theme_variant":"mocha",
            "accent_color":"Blue","accent_auto":true,"text_scale":"Normal",
            "accessible_text_size":"Standard","ui_animations":true,
            "accessibility_high_contrast":false,"increase_contrast":false,
            "accessibility_dyslexia_font":false,"accessibility_reduce_motion":false,
            "reduce_animations":false,"reduce_transitions":false}"#;
        let settings = UserSettings::from_json(raw).unwrap();
        assert_eq!(settings.standard, UserSettings::default().standard);
        assert_eq!(settings.standard.accent_override(), None, "the theme's own accent");
    }

    #[test]
    fn old_text_sizes_land_on_the_nearest_colony_step() {
        for (text_scale, accessible, font_size, text_size) in [
            ("Normal", "Standard", FontSize::Default, TextSize::Default),
            ("Large", "Large", FontSize::Large, TextSize::Large),
            ("ExtraLarge", "ExtraLarge", FontSize::Large, TextSize::XLarge),
        ] {
            let raw = format!(
                r#"{{"text_scale":"{text_scale}","accessible_text_size":"{accessible}"}}"#
            );
            let settings = UserSettings::from_json(&raw).unwrap().standard;
            assert_eq!(settings.font_size, font_size, "{text_scale}");
            assert_eq!(settings.text_size_a11y, text_size, "{accessible}");
        }
    }

    /// design/typography.md: the two sizes multiply, and a layout has to
    /// survive both ends of the product.
    #[test]
    fn both_text_sizes_multiply_into_sz() {
        let mut settings = UserSettings::default();
        for (font_size, text_size, scale, title) in [
            (FontSize::Small, TextSize::Small, 0.7225, 22.0),
            (FontSize::Default, TextSize::Default, 1.0, 30.0),
            (FontSize::Large, TextSize::XLarge, 1.68, 50.0),
        ] {
            settings.standard.font_size = font_size;
            settings.standard.text_size_a11y = text_size;
            settings.standard.apply();
            let typo = colony_ui::Typography::current();
            assert!((typo.scale - scale).abs() < 1e-6, "{font_size:?} x {text_size:?}");
            assert!((typo.sz(30) - title).abs() < f32::EPSILON);
        }
        UserSettings::default().standard.apply();
    }

    #[test]
    fn animations_follow_reduced_motion() {
        let mut settings = UserSettings::default();
        assert!(settings.animations_enabled());
        settings.standard.reduce_motion = true;
        assert!(!settings.animations_enabled(), "reduced motion silences them");
        settings.standard.reduce_motion = false;
        settings.standard.animations = false;
        assert!(!settings.animations_enabled());
    }

    #[test]
    fn normalized_clamps_volume() {
        let mut settings = UserSettings::default();
        settings.default_volume = 200;
        let normalized = settings.normalized();
        assert_eq!(normalized.default_volume, 100);
    }

    #[test]
    fn a_file_with_the_removed_settings_still_loads() {
        // Written by releases that still showed these controls. Unknown keys
        // must be ignored, not turn the whole file into defaults.
        let json = r#"{"default_volume":40,"crossfade_seconds":4,"automix_enabled":true,
            "normalize_volume":true,"auto_check_updates":false,"update_channel":"Beta",
            "auto_install_updates":true}"#;
        let settings: UserSettings = serde_json::from_str(json).expect("old keys are ignored");
        assert_eq!(settings.default_volume, 40);
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
    fn normalized_resets_traversal_cache_path() {
        let mut settings = UserSettings::default();
        settings.cache_path = "../escape".to_string();
        let normalized = settings.normalized();
        assert_eq!(normalized.cache_path, "", "a traversal falls back to the default");

        let mut settings = UserSettings::default();
        settings.cache_path = "/srv/cache/../..".to_string();
        assert_eq!(settings.normalized().cache_path, "", "absolute paths are no exception");
    }

    #[test]
    fn the_old_in_library_default_is_cleared_on_upgrade() {
        let mut settings = UserSettings::default();
        settings.cache_path = ".grape_cache".to_string();
        assert_eq!(
            settings.normalized().cache_path,
            "",
            "the old default must not pin an upgrading user to their music folder"
        );
    }

    #[test]
    fn default_cache_path_resolves_to_the_colony_cache_root() {
        let settings = UserSettings::default();
        let root = std::path::Path::new("/music");
        let dir = library_cache_dir(&settings, root);
        assert!(
            !dir.starts_with(root),
            "the default cache must not live inside the library: {}",
            dir.display()
        );
        assert!(
            dir.ends_with(crate::library::cache::library_key(root)),
            "and it must be namespaced per library: {}",
            dir.display()
        );
    }

    #[test]
    fn an_explicit_relative_cache_path_still_resolves_against_the_library() {
        let mut settings = UserSettings::default();
        settings.cache_path = "my_cache".to_string();
        let root = std::path::Path::new("/music");
        assert_eq!(
            library_cache_dir(&settings, root),
            root.join("my_cache").join(CUSTOM_CACHE_DIRNAME)
        );
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

    /// Grape's roots, a home folder and a library in a temporary directory,
    /// with files *Clear cache* must never touch: a track, a folder of the
    /// user's own named like a cache entry, and unrelated files in home.
    fn sandbox() -> (tempfile::TempDir, Roots, PathBuf, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let roots = Roots {
            config: tmp.path().join("home/.config/Colony/Grape"),
            data: tmp.path().join("home/.local/share/Colony/Grape"),
            cache: tmp.path().join("home/.cache/Colony/Grape"),
            legacy: tmp.path().join("home/.config/Colony/Grape"),
            migrated: true,
        };
        let home = tmp.path().join("home");
        let library = home.join("Music");
        for file in [
            library.join("Artist/01 - Song.flac"),
            library.join("covers/front.jpg"),
            home.join("notes.txt"),
            home.join("index.json"),
            roots.config.join("preferences.json"),
        ] {
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, "keep").unwrap();
        }
        (tmp, roots, home, library)
    }

    fn assert_untouched(home: &Path, library: &Path, roots: &Roots, cache_path: &str) {
        for file in [
            library.join("Artist/01 - Song.flac"),
            library.join("covers/front.jpg"),
            home.join("notes.txt"),
            home.join("index.json"),
            roots.config.join("preferences.json"),
        ] {
            assert!(file.exists(), "cache_path {cache_path:?} deleted {}", file.display());
        }
    }

    #[test]
    fn clear_cache_never_reaches_a_refused_location() {
        let (tmp, roots, home, library) = sandbox();
        let default = resolve_library_cache_dir("", &library, &roots, Some(&home));
        let filesystem_root = tmp.path().ancestors().last().unwrap();
        let refused = [
            ".".to_string(),
            "./".to_string(),
            "..".to_string(),
            "Artist/../..".to_string(),
            library.display().to_string(),
            home.display().to_string(),
            // An ancestor of the library, of home and of every root.
            tmp.path().display().to_string(),
            roots.config.display().to_string(),
            roots.data.display().to_string(),
            filesystem_root.display().to_string(),
        ];
        for cache_path in &refused {
            let dir = resolve_library_cache_dir(cache_path, &library, &roots, Some(&home));
            assert_eq!(dir, default, "{cache_path:?} must fall back to the default");
            clear_library_cache(&dir).unwrap();
            assert_untouched(&home, &library, &roots, cache_path);
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_home_is_refused_too() {
        let (tmp, roots, home, library) = sandbox();
        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&home, &alias).unwrap();
        let cache_path = alias.display().to_string();
        let dir = resolve_library_cache_dir(&cache_path, &library, &roots, Some(&home));
        assert_eq!(dir, resolve_library_cache_dir("", &library, &roots, Some(&home)));
    }

    #[test]
    fn a_custom_location_outside_everything_is_used_as_given() {
        let (tmp, roots, home, library) = sandbox();
        let elsewhere = tmp.path().join("elsewhere/cache");
        let cache_path = elsewhere.display().to_string();
        assert_eq!(
            resolve_library_cache_dir(&cache_path, &library, &roots, Some(&home)),
            elsewhere.join(CUSTOM_CACHE_DIRNAME)
        );
        assert_eq!(
            resolve_library_cache_dir("Cache", &library, &roots, Some(&home)),
            library.join("Cache").join(CUSTOM_CACHE_DIRNAME),
            "a sub-folder of the library is not the library"
        );
    }

    #[test]
    fn a_custom_location_keeps_the_users_folders_named_like_cache_entries() {
        let (tmp, roots, home, library) = sandbox();
        let documents = tmp.path().join("Documents");
        let own = [
            "index.json",
            "folders/a.json",
            "tracks/b.json",
            "covers/c.jpg",
            "metadata/d.json",
        ];
        for entry in own {
            let file = documents.join(entry);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, "keep").unwrap();
        }

        let cache_path = documents.display().to_string();
        let dir = resolve_library_cache_dir(&cache_path, &library, &roots, Some(&home));
        assert_eq!(dir, documents.join(CUSTOM_CACHE_DIRNAME));
        for entry in own {
            let file = dir.join(entry);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, "x").unwrap();
        }

        clear_library_cache(&dir).unwrap();
        assert!(!dir.exists(), "Grape's own folder is left empty, so it goes");
        for entry in own {
            assert!(documents.join(entry).exists(), "the user's {entry} was deleted");
        }
    }

    #[test]
    fn a_refused_cache_location_is_not_saved() {
        let (tmp, roots, home, library) = sandbox();
        let mut settings = UserSettings::default();
        settings.library_folder = library.display().to_string();
        for refused in [
            ".".to_string(),
            library.display().to_string(),
            home.display().to_string(),
            tmp.path().display().to_string(),
        ] {
            settings.cache_path = refused.clone();
            let stored = storable_settings(&settings, &roots, Some(&home));
            assert_eq!(stored.cache_path, "", "{refused:?} was saved");
            assert_eq!(settings.cache_path, refused, "the field being typed in is left alone");
        }

        let kept = tmp.path().join("elsewhere").display().to_string();
        settings.cache_path = kept.clone();
        assert_eq!(storable_settings(&settings, &roots, Some(&home)).cache_path, kept);
    }

    #[test]
    fn clear_cache_removes_grape_entries_and_keeps_foreign_files() {
        let (_tmp, roots, home, library) = sandbox();
        let dir = resolve_library_cache_dir("", &library, &roots, Some(&home));
        let grape_entries = [
            "index.json",
            "folders/a.json",
            "tracks/b.json",
            "covers/c.jpg",
            "metadata/d.json",
        ];
        for entry in grape_entries.iter().chain(["notes.txt"].iter()) {
            let file = dir.join(entry);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, "x").unwrap();
        }

        clear_library_cache(&dir).unwrap();
        for entry in ["index.json", "folders", "tracks", "covers", "metadata"] {
            assert!(!dir.join(entry).exists(), "{entry} survived the clear");
        }
        assert!(dir.join("notes.txt").exists(), "a foreign file was deleted");
        assert_untouched(&home, &library, &roots, "");

        // With nothing foreign left, the directory itself goes, and clearing
        // a cache that is not there is not an error.
        fs::remove_file(dir.join("notes.txt")).unwrap();
        fs::write(dir.join("index.json"), "x").unwrap();
        clear_library_cache(&dir).unwrap();
        assert!(!dir.exists());
        clear_library_cache(&dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_leaves_an_owner_only_file() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("preferences.json");
        // A world-readable temp file left by an interrupted write must not
        // pass its mode on.
        let stale = tmp.path().join(".preferences.json.tmp");
        fs::write(&stale, "old").unwrap();
        fs::set_permissions(&stale, fs::Permissions::from_mode(0o644)).unwrap();

        atomic_write(&path, b"{}").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"{}");
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn default_settings_roundtrip_through_json() {
        let settings = UserSettings::default();
        let json = serde_json::to_string(&settings).expect("serialize");
        let deserialized: UserSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(settings, deserialized);
    }
}
