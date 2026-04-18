#![allow(dead_code)]
use std::path::PathBuf;
use std::time::Duration;

use crate::config::{DeclarativeAction, ThemeMode, UserSettings};
use crate::ui::message::{PlaybackMessage, SearchMessage, UiMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Artists,
    Genres,
    Albums,
    Folders,
}

impl Default for ActiveTab {
    fn default() -> Self {
        Self::Artists
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryFocus {
    Artists,
    Genres,
    Albums,
    Folders,
    Songs,
}

impl Default for LibraryFocus {
    fn default() -> Self {
        Self::Artists
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesTab {
    General,
    Appearance,
    Accessibility,
    Audio,
}

impl Default for PreferencesTab {
    fn default() -> Self {
        Self::General
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesSection {
    Startup,
    Language,
    Updates,
    Privacy,
    Storage,
    SystemIntegration,
    Performance,
    Advanced,
    AppearanceTheme,
    AppearanceAccents,
    AppearanceTypography,
    AppearanceEffects,
    AppearancePreview,
    AccessibilityVision,
    AccessibilityMovement,
    AccessibilityNavigation,
    AccessibilityPlayback,
    AudioOutput,
    AudioPlayback,
    AudioVolume,
    AudioEqualizer,
    AudioAdvanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeCategory {
    Catppuccin,
    Gruvbox,
    Everblush,
    Kanagawa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeCategoriesState {
    pub catppuccin: bool,
    pub gruvbox: bool,
    pub everblush: bool,
    pub kanagawa: bool,
}

impl ThemeCategoriesState {
    pub fn toggle(&mut self, category: ThemeCategory) {
        match category {
            ThemeCategory::Catppuccin => self.catppuccin = !self.catppuccin,
            ThemeCategory::Gruvbox => self.gruvbox = !self.gruvbox,
            ThemeCategory::Everblush => self.everblush = !self.everblush,
            ThemeCategory::Kanagawa => self.kanagawa = !self.kanagawa,
        }
    }
}

impl Default for ThemeCategoriesState {
    fn default() -> Self {
        Self {
            catppuccin: false,
            gruvbox: false,
            everblush: false,
            kanagawa: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreferencesSectionsState {
    pub startup: bool,
    pub language: bool,
    pub updates: bool,
    pub privacy: bool,
    pub storage: bool,
    pub system_integration: bool,
    pub performance: bool,
    pub advanced: bool,
    pub appearance_theme: bool,
    pub appearance_accents: bool,
    pub appearance_typography: bool,
    pub appearance_effects: bool,
    pub appearance_preview: bool,
    pub accessibility_vision: bool,
    pub accessibility_movement: bool,
    pub accessibility_navigation: bool,
    pub accessibility_playback: bool,
    pub audio_output: bool,
    pub audio_playback: bool,
    pub audio_volume: bool,
    pub audio_equalizer: bool,
    pub audio_advanced: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreferencesScrollState {
    pub general: f32,
    pub appearance: f32,
    pub accessibility: f32,
    pub audio: f32,
}

impl PreferencesScrollState {
    pub fn offset_for(&self, tab: PreferencesTab) -> f32 {
        match tab {
            PreferencesTab::General => self.general,
            PreferencesTab::Appearance => self.appearance,
            PreferencesTab::Accessibility => self.accessibility,
            PreferencesTab::Audio => self.audio,
        }
    }

    pub fn set_offset(&mut self, tab: PreferencesTab, offset_y: f32) {
        match tab {
            PreferencesTab::General => self.general = offset_y,
            PreferencesTab::Appearance => self.appearance = offset_y,
            PreferencesTab::Accessibility => self.accessibility = offset_y,
            PreferencesTab::Audio => self.audio = offset_y,
        }
    }
}

impl Default for PreferencesScrollState {
    fn default() -> Self {
        Self {
            general: 0.0,
            appearance: 0.0,
            accessibility: 0.0,
            audio: 0.0,
        }
    }
}

impl PreferencesSectionsState {
    pub fn toggle(&mut self, section: PreferencesSection) {
        match section {
            PreferencesSection::Startup => self.startup = !self.startup,
            PreferencesSection::Language => self.language = !self.language,
            PreferencesSection::Updates => self.updates = !self.updates,
            PreferencesSection::Privacy => self.privacy = !self.privacy,
            PreferencesSection::Storage => self.storage = !self.storage,
            PreferencesSection::SystemIntegration => {
                self.system_integration = !self.system_integration;
            }
            PreferencesSection::Performance => self.performance = !self.performance,
            PreferencesSection::Advanced => self.advanced = !self.advanced,
            PreferencesSection::AppearanceTheme => self.appearance_theme = !self.appearance_theme,
            PreferencesSection::AppearanceAccents => {
                self.appearance_accents = !self.appearance_accents;
            }
            PreferencesSection::AppearanceTypography => {
                self.appearance_typography = !self.appearance_typography;
            }
            PreferencesSection::AppearanceEffects => {
                self.appearance_effects = !self.appearance_effects;
            }
            PreferencesSection::AppearancePreview => {
                self.appearance_preview = !self.appearance_preview;
            }
            PreferencesSection::AccessibilityVision => {
                self.accessibility_vision = !self.accessibility_vision;
            }
            PreferencesSection::AccessibilityMovement => {
                self.accessibility_movement = !self.accessibility_movement;
            }
            PreferencesSection::AccessibilityNavigation => {
                self.accessibility_navigation = !self.accessibility_navigation;
            }
            PreferencesSection::AccessibilityPlayback => {
                self.accessibility_playback = !self.accessibility_playback;
            }
            PreferencesSection::AudioOutput => self.audio_output = !self.audio_output,
            PreferencesSection::AudioPlayback => self.audio_playback = !self.audio_playback,
            PreferencesSection::AudioVolume => self.audio_volume = !self.audio_volume,
            PreferencesSection::AudioEqualizer => self.audio_equalizer = !self.audio_equalizer,
            PreferencesSection::AudioAdvanced => self.audio_advanced = !self.audio_advanced,
        }
    }
}

impl Default for PreferencesSectionsState {
    fn default() -> Self {
        Self {
            startup: true,
            language: false,
            updates: false,
            privacy: false,
            storage: true,
            system_integration: false,
            performance: false,
            advanced: false,
            appearance_theme: true,
            appearance_accents: true,
            appearance_typography: true,
            appearance_effects: true,
            appearance_preview: true,
            accessibility_vision: true,
            accessibility_movement: true,
            accessibility_navigation: true,
            accessibility_playback: true,
            audio_output: true,
            audio_playback: true,
            audio_volume: true,
            audio_equalizer: false,
            audio_advanced: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    pub id: usize,
    pub name: String,
    pub normalized_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    pub id: usize,
    pub title: String,
    pub artist: String,
    pub year: Option<u32>,
    pub total_duration: Duration,
    pub cover_path: Option<PathBuf>,
    pub normalized_title: String,
    pub normalized_artist: String,
    pub genre: Option<String>,
    pub track_genres: Vec<String>,
    pub has_tracks_with_genre: bool,
    pub codecs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genre {
    pub id: usize,
    pub name: String,
    pub track_count: usize,
    pub normalized_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub id: usize,
    pub name: String,
    pub track_count: usize,
    pub normalized_name: String,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub total_duration: Duration,
    pub cover_path: Option<PathBuf>,
    pub codecs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub id: usize,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub track_number: Option<u32>,
    pub duration: Duration,
    pub path: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub normalized_title: String,
    pub normalized_artist: String,
    pub normalized_album: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SelectionState {
    pub selected_artist: Option<Artist>,
    pub selected_album: Option<Album>,
    pub selected_genre: Option<Genre>,
    pub selected_folder: Option<Folder>,
    pub selected_track: Option<Track>,
    pub selected_playlist: Option<usize>,
    pub playlist_name_draft: String,
    pub playlist_drag_source: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl Default for RepeatMode {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PlaybackState {
    pub position: Duration,
    pub duration: Duration,
    pub is_playing: bool,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub animated_progress: f32,
}

impl PlaybackState {
    pub fn update(&mut self, message: PlaybackMessage) {
        match message {
            PlaybackMessage::ToggleShuffle => {
                self.shuffle = !self.shuffle;
            }
            PlaybackMessage::CycleRepeat => {
                self.repeat = match self.repeat {
                    RepeatMode::Off => RepeatMode::All,
                    RepeatMode::All => RepeatMode::One,
                    RepeatMode::One => RepeatMode::Off,
                };
            }
            PlaybackMessage::TogglePlayPause
            | PlaybackMessage::NextTrack
            | PlaybackMessage::PreviousTrack
            | PlaybackMessage::SeekToRatio(_) => {}
        }
    }

    pub fn update_animated_progress(&mut self) {
        let total = self.duration.as_secs_f32();
        if total <= 0.0 {
            self.animated_progress = 0.0;
            return;
        }
        let target = progress_ratio(self.position, self.duration);
        let displayed_position = (self.animated_progress * total).clamp(0.0, total);
        let position_delta = (self.position.as_secs_f32() - displayed_position).abs();
        let ratio_delta = (target - self.animated_progress).abs();
        if position_delta > 2.5 || ratio_delta > 0.15 {
            self.animated_progress = target;
        } else if ratio_delta < 0.001 {
            self.animated_progress = target;
        } else {
            self.animated_progress =
                (self.animated_progress + (target - self.animated_progress) * 0.2).clamp(0.0, 1.0);
        }
    }
}

pub fn progress_ratio(position: Duration, duration: Duration) -> f32 {
    let total = duration.as_secs_f32();
    if total <= 0.0 {
        return 0.0;
    }
    let current = position.as_secs_f32().min(total);
    (current / total).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    Alphabetical,
    ByAlbum,
    ByYear,
    ByDuration,
}

impl Default for SortOption {
    fn default() -> Self {
        Self::ByAlbum
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchFilter {
    Genre,
    Year,
    Duration,
    Codec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchFilters {
    pub genre: bool,
    pub year: bool,
    pub duration: bool,
    pub codec: bool,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            genre: false,
            year: true,
            duration: false,
            codec: false,
        }
    }
}

impl SearchFilters {
    pub fn toggle(&mut self, filter: SearchFilter) {
        match filter {
            SearchFilter::Genre => {
                self.genre = !self.genre;
            }
            SearchFilter::Year => {
                self.year = !self.year;
            }
            SearchFilter::Duration => {
                self.duration = !self.duration;
            }
            SearchFilter::Codec => {
                self.codec = !self.codec;
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SearchState {
    pub query: String,
    pub sort: SortOption,
    pub filters: SearchFilters,
}

impl SearchState {
    pub fn update(&mut self, message: SearchMessage) {
        match message {
            SearchMessage::QueryChanged(query) => {
                self.query = query;
            }
            SearchMessage::SortChanged(sort) => {
                self.sort = sort;
            }
            SearchMessage::ToggleFilter(filter) => {
                self.filters.toggle(filter);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListLimits {
    pub artists: usize,
    pub albums: usize,
    pub tracks: usize,
    pub genres: usize,
    pub folders: usize,
}

impl Default for ListLimits {
    fn default() -> Self {
        Self {
            artists: 120,
            albums: 90,
            tracks: 120,
            genres: 120,
            folders: 120,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanStage {
    Indexing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScanStatus {
    pub stage: ScanStage,
    pub root: PathBuf,
    pub progress: f32,
}

impl ScanStatus {
    pub fn new(root: PathBuf) -> Self {
        Self {
            stage: ScanStage::Indexing,
            root,
            progress: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecentTrack {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub path: std::path::PathBuf,
    pub played_at: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiState {
    pub active_tab: ActiveTab,
    pub selection: SelectionState,
    pub playback: PlaybackState,
    pub search: SearchState,
    pub library_focus: LibraryFocus,
    pub menu_open: bool,
    pub playlist_open: bool,
    pub queue_open: bool,
    pub preferences_open: bool,
    pub preferences_tab: PreferencesTab,
    pub preferences_sections: PreferencesSectionsState,
    pub preferences_scroll: PreferencesScrollState,
    pub theme_categories: ThemeCategoriesState,
    pub pending_action: Option<DeclarativeAction>,
    pub settings: UserSettings,
    pub audio_notice: Option<String>,
    pub play_from_queue: bool,
    pub list_limits: ListLimits,
    pub scan_status: Option<ScanStatus>,
    pub needs_initial_scan: bool,
    pub inline_volume_bar_open: bool,
    pub inline_volume_visibility: f32,
    pub album_genre_draft: String,
    pub album_year_draft: String,
    pub mini_player: bool,
    pub speed_popup_open: bool,
    pub error_message: Option<String>,
    pub recently_played: Vec<RecentTrack>,
}

impl UiState {
    pub fn new(settings: UserSettings) -> Self {
        let needs_initial_scan = settings.auto_scan_on_launch;
        Self {
            active_tab: ActiveTab::default(),
            selection: SelectionState::default(),
            playback: PlaybackState::default(),
            search: SearchState::default(),
            library_focus: LibraryFocus::default(),
            menu_open: false,
            playlist_open: false,
            queue_open: false,
            preferences_open: false,
            preferences_tab: PreferencesTab::default(),
            preferences_sections: PreferencesSectionsState::default(),
            preferences_scroll: PreferencesScrollState::default(),
            theme_categories: ThemeCategoriesState::default(),
            pending_action: None,
            settings,
            audio_notice: None,
            play_from_queue: true,
            list_limits: ListLimits::default(),
            scan_status: None,
            needs_initial_scan,
            inline_volume_bar_open: false,
            inline_volume_visibility: 0.0,
            album_genre_draft: String::new(),
            album_year_draft: String::new(),
            mini_player: false,
            speed_popup_open: false,
            error_message: None,
            recently_played: Vec::new(),
        }
    }

    pub fn update_inline_volume_visibility(&mut self) {
        let target = if self.inline_volume_bar_open { 1.0 } else { 0.0 };
        if !self.settings.ui_animations
            || self.settings.reduce_animations
            || self.settings.accessibility_reduce_motion
        {
            self.inline_volume_visibility = target;
            return;
        }
        let delta = target - self.inline_volume_visibility;
        if delta.abs() < 0.02 {
            self.inline_volume_visibility = target;
        } else {
            self.inline_volume_visibility =
                (self.inline_volume_visibility + delta * 0.2).clamp(0.0, 1.0);
        }
    }

    pub fn update(&mut self, message: UiMessage) {
        match message {
            UiMessage::TabSelected(tab) => {
                self.active_tab = tab;
                self.playlist_open = false;
                self.queue_open = false;
                self.preferences_open = false;
                self.library_focus = match tab {
                    ActiveTab::Artists => LibraryFocus::Artists,
                    ActiveTab::Genres => LibraryFocus::Genres,
                    ActiveTab::Albums => LibraryFocus::Albums,
                    ActiveTab::Folders => LibraryFocus::Folders,
                };
            }
            UiMessage::SelectArtist(artist) => {
                self.selection.selected_artist = Some(artist);
                self.selection.selected_album = None;
                self.selection.selected_track = None;
                self.library_focus = LibraryFocus::Artists;
            }
            UiMessage::SelectAlbum(album) => {
                self.selection.selected_album = Some(album);
                self.selection.selected_track = None;
                self.library_focus = LibraryFocus::Albums;
            }
            UiMessage::SelectGenre(genre) => {
                self.selection.selected_genre = Some(genre);
                self.selection.selected_folder = None;
                self.selection.selected_track = None;
                self.library_focus = LibraryFocus::Genres;
            }
            UiMessage::SelectFolder(folder) => {
                self.selection.selected_folder = Some(folder);
                self.selection.selected_genre = None;
                self.library_focus = LibraryFocus::Folders;
            }
            UiMessage::SelectTrack(track) => {
                self.selection.selected_track = Some(track);
                self.library_focus = LibraryFocus::Songs;
            }
            UiMessage::AlbumGenreChanged(value) => {
                self.album_genre_draft = value;
            }
            UiMessage::AlbumYearChanged(value) => {
                self.album_year_draft = value;
            }
            UiMessage::SaveAlbumMetadata => {}
            UiMessage::EnrichAlbumMetadata => {}
            UiMessage::AlbumMetadataFetched { .. } => {}
            UiMessage::SelectPlaylist(index) => {
                self.selection.selected_playlist = Some(index);
            }
            UiMessage::Playback(message) => {
                self.playback.update(message);
            }
            UiMessage::Search(message) => {
                self.search.update(message);
            }
            UiMessage::TogglePlayFromQueue => {
                self.play_from_queue = !self.play_from_queue;
            }
            UiMessage::RemoveQueueItem(_) => {}
            UiMessage::ToggleLogoMenu => {
                self.menu_open = !self.menu_open;
            }
            UiMessage::WindowMinimize => {}
            UiMessage::WindowToggleMaximize => {}
            UiMessage::WindowClose => {}
            UiMessage::OpenPlaylist => {
                self.menu_open = false;
                self.playlist_open = true;
                self.queue_open = false;
                self.preferences_open = false;
            }
            UiMessage::ClosePlaylist => {
                self.playlist_open = false;
            }
            UiMessage::OpenQueue => {
                self.menu_open = false;
                self.playlist_open = false;
                self.queue_open = true;
                self.preferences_open = false;
            }
            UiMessage::CloseQueue => {
                self.queue_open = false;
            }
            UiMessage::ShowLibrary => {
                self.menu_open = false;
                self.playlist_open = false;
                self.queue_open = false;
                self.preferences_open = false;
            }
            UiMessage::OpenPreferences => {
                self.menu_open = false;
                self.playlist_open = false;
                self.queue_open = false;
                self.preferences_open = true;
            }
            UiMessage::ClosePreferences => {
                self.preferences_open = false;
                self.pending_action = None;
            }
            UiMessage::PreferencesTabSelected(tab) => {
                self.preferences_tab = tab;
            }
            UiMessage::PreferencesScrolled { tab, offset_y } => {
                self.preferences_scroll.set_offset(tab, offset_y);
            }
            UiMessage::ToggleThemeCategory(category) => {
                self.theme_categories.toggle(category);
            }
            UiMessage::SetThemeMode(theme_mode) => {
                self.settings.theme_mode = theme_mode;
                self.settings.follow_system_theme = false;
            }
            UiMessage::SetFollowSystemTheme(enabled) => {
                self.settings.follow_system_theme = enabled;
                if enabled {
                    self.settings.theme_mode = ThemeMode::Mocha;
                }
            }
            UiMessage::SetAccentColor(color) => {
                self.settings.accent_color = color;
                self.settings.accent_auto = false;
            }
            UiMessage::SetAccentAuto(enabled) => {
                self.settings.accent_auto = enabled;
            }
            UiMessage::SetTextScale(scale) => {
                self.settings.text_scale = scale;
                self.settings.accessibility_large_text = scale != crate::config::TextScale::Normal;
            }
            UiMessage::SetInterfaceDensity(density) => {
                self.settings.interface_density = density;
            }
            UiMessage::SetTransparencyBlur(enabled) => {
                self.settings.transparency_blur = enabled;
            }
            UiMessage::SetUiAnimations(enabled) => {
                self.settings.ui_animations = enabled;
                self.update_inline_volume_visibility();
            }
            UiMessage::SetAccessibilityLargeText(enabled) => {
                self.settings.accessibility_large_text = enabled;
                if enabled {
                    if self.settings.text_scale == crate::config::TextScale::Normal {
                        self.settings.text_scale = crate::config::TextScale::Large;
                    }
                    if self.settings.accessible_text_size
                        == crate::config::AccessibleTextSize::Standard
                    {
                        self.settings.accessible_text_size =
                            crate::config::AccessibleTextSize::Large;
                    }
                }
            }
            UiMessage::SetAccessibilityHighContrast(enabled) => {
                self.settings.accessibility_high_contrast = enabled;
                self.settings.increase_contrast = enabled;
            }
            UiMessage::SetAccessibilityReduceMotion(enabled) => {
                self.settings.accessibility_reduce_motion = enabled;
                if enabled {
                    self.settings.reduce_animations = true;
                    self.settings.reduce_transitions = true;
                }
                self.update_inline_volume_visibility();
            }
            UiMessage::SetIncreaseContrast(enabled) => {
                self.settings.increase_contrast = enabled;
                self.settings.accessibility_high_contrast = enabled;
            }
            UiMessage::SetReduceTransparency(enabled) => {
                self.settings.reduce_transparency = enabled;
            }
            UiMessage::SetAccessibleTextSize(size) => {
                self.settings.accessible_text_size = size;
            }
            UiMessage::SetReduceAnimations(enabled) => {
                self.settings.reduce_animations = enabled;
                self.settings.accessibility_reduce_motion =
                    self.settings.reduce_animations || self.settings.reduce_transitions;
                self.update_inline_volume_visibility();
            }
            UiMessage::SetReduceTransitions(enabled) => {
                self.settings.reduce_transitions = enabled;
                self.settings.accessibility_reduce_motion =
                    self.settings.reduce_animations || self.settings.reduce_transitions;
                self.update_inline_volume_visibility();
            }
            UiMessage::ToggleInlineVolumeBar => {
                self.inline_volume_bar_open = !self.inline_volume_bar_open;
                self.update_inline_volume_visibility();
            }
            UiMessage::SetHighlightKeyboardFocus(enabled) => {
                self.settings.highlight_keyboard_focus = enabled;
            }
            UiMessage::SetAdvancedShortcuts(enabled) => {
                self.settings.enable_advanced_shortcuts = enabled;
            }
            UiMessage::SetDefaultPlaybackSpeed(speed) => {
                self.settings.default_playback_speed = speed.clamp(5, 20);
            }
            UiMessage::SetPauseOnFocusLoss(enabled) => {
                self.settings.pause_on_focus_loss = enabled;
            }
            UiMessage::SetDefaultVolume(volume) => {
                self.settings.default_volume = volume.min(100);
            }
            UiMessage::SetAudioOutputDevice(device) => {
                self.settings.output_device = device;
            }
            UiMessage::SetMissingDeviceBehavior(behavior) => {
                self.settings.missing_device_behavior = behavior;
            }
            UiMessage::SetGaplessPlayback(enabled) => {
                self.settings.gapless_playback = enabled;
            }
            UiMessage::SetCrossfadeSeconds(seconds) => {
                self.settings.crossfade_seconds = seconds.min(12);
            }
            UiMessage::SetAutomixEnabled(enabled) => {
                self.settings.automix_enabled = enabled;
            }
            UiMessage::SetNormalizeVolume(enabled) => {
                self.settings.normalize_volume = enabled;
            }
            UiMessage::SetVolumeLevel(level) => {
                self.settings.volume_level = level;
            }
            UiMessage::SetEqEnabled(enabled) => {
                self.settings.eq_enabled = enabled;
            }
            UiMessage::SetEqPreset(preset) => {
                self.settings.eq_preset = preset;
                if preset != crate::config::EqPreset::Custom {
                    let mut model = self.settings.eq_model.clone();
                    preset.apply_to_model(&mut model);
                    self.settings.eq_model = model;
                }
            }
            UiMessage::SetEqBandGain(index, gain_db) => {
                if let Some(band) = self.settings.eq_model.bands.get_mut(index) {
                    band.gain_db = gain_db.clamp(-12.0, 12.0);
                    self.settings.eq_preset = crate::config::EqPreset::Custom;
                }
            }
            UiMessage::ResetEq => {
                self.settings.eq_preset = crate::config::EqPreset::Flat;
                let mut model = self.settings.eq_model.clone();
                crate::config::EqPreset::Flat.apply_to_model(&mut model);
                self.settings.eq_model = model;
            }
            UiMessage::SetAudioStabilityMode(mode) => {
                self.settings.audio_stability_mode = mode;
            }
            UiMessage::ResetAudioEngine => {}
            UiMessage::SetAudioDebugLogs(enabled) => {
                self.settings.audio_debug_logs = enabled;
            }
            UiMessage::SetLaunchAtStartup(enabled) => {
                self.settings.launch_at_startup = enabled;
            }
            UiMessage::SetRestoreLastSession(enabled) => {
                self.settings.restore_last_session = enabled;
            }
            UiMessage::SetOpenOn(open_on) => {
                self.settings.open_on = open_on;
            }
            UiMessage::SetCloseBehavior(behavior) => {
                self.settings.close_behavior = behavior;
            }
            UiMessage::SetInterfaceLanguage(language) => {
                self.settings.interface_language = language;
            }
            UiMessage::SetTimeFormat(format) => {
                self.settings.time_format = format;
            }
            UiMessage::SetAutoCheckUpdates(enabled) => {
                self.settings.auto_check_updates = enabled;
            }
            UiMessage::SetUpdateChannel(channel) => {
                self.settings.update_channel = channel;
            }
            UiMessage::SetAutoInstallUpdates(enabled) => {
                self.settings.auto_install_updates = enabled;
            }
            UiMessage::LibraryFolderChanged(path) => {
                self.settings.library_folder = path;
            }
            UiMessage::PickLibraryFolder => {}
            UiMessage::LibraryFolderPicked(path) => {
                if let Some(path) = path {
                    self.settings.library_folder = path;
                }
            }
            UiMessage::SetAutoScanOnLaunch(enabled) => {
                self.settings.auto_scan_on_launch = enabled;
            }
            UiMessage::CachePathChanged(path) => {
                self.settings.cache_path = path;
            }
            UiMessage::ClearCache => {}
            UiMessage::ClearHistory => {}
            UiMessage::SetNotificationsEnabled(enabled) => {
                self.settings.notifications_enabled = enabled;
                if !enabled {
                    self.settings.now_playing_notifications = false;
                }
            }
            UiMessage::SetNowPlayingNotifications(enabled) => {
                self.settings.now_playing_notifications = enabled;
                if enabled {
                    self.settings.notifications_enabled = true;
                }
            }
            UiMessage::SetSystemTrayEnabled(enabled) => {
                self.settings.system_tray_enabled = enabled;
            }
            UiMessage::SetHardwareAcceleration(enabled) => {
                self.settings.hardware_acceleration = enabled;
            }
            UiMessage::SetLimitCpuDuringPlayback(enabled) => {
                self.settings.limit_cpu_during_playback = enabled;
            }
            UiMessage::OpenLogsFolder => {}
            UiMessage::ReindexLibrary => {}
            UiMessage::ResetPreferences => {
                self.settings = UserSettings::default();
            }
            UiMessage::RequestDeclarativeAction(action) => {
                self.pending_action = Some(action);
            }
            UiMessage::ConfirmDeclarativeAction(_) => {
                self.pending_action = None;
            }
            UiMessage::CancelDeclarativeAction => {
                self.pending_action = None;
            }
            UiMessage::TogglePreferencesSection(section) => {
                self.preferences_sections.toggle(section);
            }
            UiMessage::CloseMenu => {
                self.menu_open = false;
            }
            UiMessage::PlaybackTick => {}
            UiMessage::AnimationTick => {}
            UiMessage::StartInitialScan => {}
            UiMessage::ScanTick => {}
            UiMessage::LibraryScanCompleted(_) => {}
            UiMessage::NavigateLibrary(_) | UiMessage::ActivateSelection => {}
            UiMessage::PlaylistNameChanged(name) => {
                self.selection.playlist_name_draft = name.clone();
            }
            UiMessage::CreatePlaylist => {}
            UiMessage::RenamePlaylist => {}
            UiMessage::DeletePlaylist => {}
            UiMessage::MovePlaylistItemUp(_)
            | UiMessage::MovePlaylistItemDown(_)
            | UiMessage::StartPlaylistItemDrag(_)
            | UiMessage::MovePlaylistItemDrag { .. }
            | UiMessage::DeletePlaylistItem(_)
            | UiMessage::SavePlaylistOrder => {}
            UiMessage::AddSelectedTrackToPlaylist => {}
            UiMessage::ClearQueue
            | UiMessage::MoveQueueItemUp(_)
            | UiMessage::MoveQueueItemDown(_) => {}
            UiMessage::DismissAudioNotice => {
                self.audio_notice = None;
            }
            UiMessage::LoadMoreArtists
            | UiMessage::LoadMoreAlbums
            | UiMessage::LoadMoreTracks
            | UiMessage::LoadMoreGenres
            | UiMessage::LoadMoreFolders => {}
            UiMessage::VolumeUp => {
                self.settings.default_volume = (self.settings.default_volume + 5).min(100);
            }
            UiMessage::VolumeDown => {
                self.settings.default_volume = self.settings.default_volume.saturating_sub(5);
            }
            UiMessage::ToggleMiniPlayer => {
                self.mini_player = !self.mini_player;
            }
            UiMessage::SetPlaybackSpeed(speed) => {
                self.settings.default_playback_speed = speed.clamp(5, 20);
                self.speed_popup_open = false;
            }
            UiMessage::ToggleSpeedPopup => {
                self.speed_popup_open = !self.speed_popup_open;
            }
            UiMessage::ExportPlaylistM3u => {}
            UiMessage::PlaylistExported(result) => match result {
                Ok(msg) => {
                    self.audio_notice = Some(msg);
                }
                Err(msg) => {
                    self.error_message = Some(msg);
                }
            },
            UiMessage::DismissError => {
                self.error_message = None;
            }
            UiMessage::WindowFocusChanged(_) => {}
        }
    }
}
