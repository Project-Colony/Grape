#![allow(dead_code)]
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use colony_ui::preferences::Change;

use crate::config::{DeclarativeAction, UserSettings};
use crate::ui::message::{PlaybackMessage, SearchMessage, UiMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTab {
    #[default]
    Artists,
    Genres,
    Albums,
    Folders,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryFocus {
    #[default]
    Artists,
    Genres,
    Albums,
    Folders,
    Songs,
}

/// The Preferences categories, in the order the category list shows them:
/// Colony's three, then Grape's own (design/settings-page.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreferencesTab {
    #[default]
    General,
    Appearance,
    Accessibility,
    Audio,
}

impl PreferencesTab {
    pub const ALL: [Self; 4] = [Self::General, Self::Appearance, Self::Accessibility, Self::Audio];

    /// The position in the category list, which is the order of [`Self::ALL`].
    pub const fn index(self) -> usize {
        self as usize
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    One,
    All,
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
    pub const fn update(&mut self, message: &PlaybackMessage) {
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
        if position_delta > 2.5 || ratio_delta > 0.15 || ratio_delta < 0.001 {
            self.animated_progress = target;
        } else {
            self.animated_progress = (target - self.animated_progress)
                .mul_add(0.2, self.animated_progress)
                .clamp(0.0, 1.0);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOption {
    Alphabetical,
    #[default]
    ByAlbum,
    ByYear,
    ByDuration,
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
    pub const fn toggle(&mut self, filter: SearchFilter) {
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
    pub const fn new(root: PathBuf) -> Self {
        Self {
            stage: ScanStage::Indexing,
            root,
            progress: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// The keys of the Preferences sections the user opened. Every section
    /// arrives closed, and the set is emptied each time the page opens.
    pub preferences_sections: HashSet<String>,
    pub pending_action: Option<DeclarativeAction>,
    pub settings: UserSettings,
    pub audio_notice: Option<String>,
    /// Set when a theme is applied; Preferences show the toast until dismissed.
    pub theme_notice: bool,
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
    /// Re-resolves where the library cache lives.
    ///
    /// Both inputs are user-editable at runtime, and a stale value would send
    /// reads and writes to the previous library's cache.
    fn refresh_cache_dir(&self) {
        crate::config::set_active_cache_dir(
            &self.settings,
            std::path::Path::new(self.settings.library_folder.trim()),
        );
    }

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
            preferences_sections: HashSet::new(),
            pending_action: None,
            settings,
            audio_notice: None,
            theme_notice: false,
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
        if !self.settings.animations_enabled() {
            self.inline_volume_visibility = target;
            return;
        }
        let delta = target - self.inline_volume_visibility;
        if delta.abs() < 0.02 {
            self.inline_volume_visibility = target;
        } else {
            self.inline_volume_visibility =
                delta.mul_add(0.2, self.inline_volume_visibility).clamp(0.0, 1.0);
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
                self.playback.update(&message);
            }
            UiMessage::Search(message) => {
                self.search.update(message);
            }
            UiMessage::TogglePlayFromQueue => {
                self.play_from_queue = !self.play_from_queue;
            }
            UiMessage::RemoveQueueItem(_) => {}
            UiMessage::ToggleLogoMenu => {
                if self.preferences_open {
                    self.preferences_open = false;
                    self.pending_action = None;
                } else {
                    self.menu_open = !self.menu_open;
                }
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
                // Always arrives on General with every section closed.
                self.preferences_tab = PreferencesTab::default();
                self.preferences_sections.clear();
                self.theme_notice = false;
            }
            UiMessage::PreferencesTabSelected(tab) => {
                self.preferences_tab = tab;
            }
            UiMessage::Preference(change) => {
                // Records it and puts it into effect at once: there is no
                // Save button.
                self.settings.standard.update(change);
                match change {
                    Change::Theme { .. } => {
                        self.settings.follow_system_theme = false;
                        self.theme_notice = true;
                    }
                    Change::Animations(_) | Change::ReduceMotion(_) => {
                        self.update_inline_volume_visibility();
                    }
                    _ => {}
                }
            }
            UiMessage::SetFollowSystemTheme(enabled) => {
                self.settings.follow_system_theme = enabled;
            }
            UiMessage::SetReduceTransparency(enabled) => {
                self.settings.reduce_transparency = enabled;
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
            UiMessage::LibraryFolderChanged(path) => {
                self.settings.library_folder = path;
                self.refresh_cache_dir();
            }
            UiMessage::PickLibraryFolder => {}
            UiMessage::LibraryFolderPicked(path) => {
                if let Some(path) = path {
                    self.settings.library_folder = path;
                    self.refresh_cache_dir();
                }
            }
            UiMessage::SetAutoScanOnLaunch(enabled) => {
                self.settings.auto_scan_on_launch = enabled;
            }
            UiMessage::CachePathChanged(path) => {
                // A `..` is refused as it is typed, so it is never held, saved
                // or used. The other checks need the whole path ("/" is on the
                // way to "/srv/cache"), so they run where the path is used and
                // saved, and refuse it there.
                if !crate::config::cache_path_has_parent_dir(&path) {
                    self.settings.cache_path = path;
                    self.refresh_cache_dir();
                }
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
            UiMessage::TogglePreferencesSection(key) => {
                if !self.preferences_sections.remove(key) {
                    self.preferences_sections.insert(key.to_string());
                }
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
                self.selection.playlist_name_draft = name;
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
            UiMessage::DismissThemeNotice => {
                self.theme_notice = false;
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
            UiMessage::PlaylistExported(result) => {
                let strings = crate::ui::i18n::strings(self.settings.interface_language);
                match result {
                    Ok(path) => self.audio_notice = Some(strings.playlist_exported(&path)),
                    Err(error) => self.error_message = Some(strings.playlist_export_failed(&error)),
                }
            }
            UiMessage::DismissError => {
                self.error_message = None;
            }
            UiMessage::WindowFocusChanged(_) | UiMessage::Media(_) => {}
            #[cfg(target_os = "windows")]
            UiMessage::MediaWindowOpened(_) | UiMessage::MediaWindowHandle(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_typed_parent_dir_never_becomes_the_cache_path() {
        let settings = UserSettings {
            cache_path: "Cache".to_string(),
            ..UserSettings::default()
        };
        let mut state = UiState::new(settings);
        for typed in ["..", "../", "Cache/../..", "/srv/cache/.."] {
            state.update(UiMessage::CachePathChanged(typed.to_string()));
            assert_eq!(state.settings.cache_path, "Cache", "{typed:?} was taken");
        }
    }

    /// design/navigation.md: the identity element shows that Preferences are
    /// open, and the same control closes them.
    #[test]
    fn the_logo_closes_the_preferences_it_opened() {
        let mut state = UiState::new(UserSettings::default());
        state.update(UiMessage::ToggleLogoMenu);
        state.update(UiMessage::OpenPreferences);
        assert!(state.preferences_open && !state.menu_open);

        state.update(UiMessage::ToggleLogoMenu);
        assert!(!state.preferences_open, "the logo left Preferences open");
        assert!(!state.menu_open, "the logo opened its menu over Preferences");

        state.update(UiMessage::ToggleLogoMenu);
        assert!(state.menu_open);
    }

    /// design/settings-page.md: every section arrives closed and stays as the
    /// user left it while the page is open.
    #[test]
    fn preferences_reopen_on_general_with_every_section_closed() {
        let mut state = UiState::new(UserSettings::default());
        state.update(UiMessage::OpenPreferences);
        assert!(state.preferences_sections.is_empty());

        state.update(UiMessage::PreferencesTabSelected(PreferencesTab::Audio));
        state.update(UiMessage::TogglePreferencesSection("audio_output"));
        state.update(UiMessage::TogglePreferencesSection("vision"));
        state.update(UiMessage::TogglePreferencesSection("vision"));
        assert_eq!(state.preferences_sections, HashSet::from(["audio_output".to_string()]));

        state.update(UiMessage::ToggleLogoMenu);
        state.update(UiMessage::OpenPreferences);
        assert_eq!(state.preferences_tab, PreferencesTab::General);
        assert!(state.preferences_sections.is_empty());
    }

    #[test]
    fn a_category_index_finds_its_tab() {
        for (index, tab) in PreferencesTab::ALL.into_iter().enumerate() {
            assert_eq!(tab.index(), index);
        }
    }
}
