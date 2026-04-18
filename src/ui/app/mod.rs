mod filters;
mod playback;
mod preferences;
mod selection;
mod update;
mod view;

use crate::config;
use crate::library::Catalog;
use crate::notifications;
use crate::player::{
    AudioFallback, AudioOptions, NowPlaying, PlaybackState as PlayerPlaybackState, Player,
};
use crate::playlist::{PlaybackQueue, PlaylistManager};
use crate::system_integration::{SystemAction, SystemIntegration, SystemIntegrationAvailability};
use crate::ui::components::albums_grid::AlbumsGrid;
use crate::ui::components::anchored_overlay::AnchoredOverlay;
use crate::ui::components::artists_panel::ArtistsPanel;
use crate::ui::components::audio_settings::eq_band_controls;
use crate::ui::components::folders_panel::FoldersPanel;
use crate::ui::components::genres_panel::GenresPanel;
use crate::ui::components::player_bar::PlayerBar;
use crate::ui::components::playlist_view::PlaylistView;
use crate::ui::components::queue_view::QueueView;
use crate::ui::components::songs_panel::SongsPanel;
use crate::ui::i18n::{self, UiStrings};
use crate::ui::message::{LibraryNavigation, PlaybackMessage, SearchMessage, UiMessage};
use crate::ui::state::{
    ActiveTab, Album as UiAlbum, Artist as UiArtist, Folder as UiFolder, Genre as UiGenre,
    LibraryFocus, ListLimits, PreferencesSection, PreferencesTab, ScanStage, ScanStatus,
    SearchFilter, SearchState, SelectionState, SortOption, ThemeCategory, Track as UiTrack,
    UiState, progress_ratio,
};
use crate::ui::style;
use iced::font::Weight;
use iced::widget::operation;
use iced::widget::{
    Id, button, container, image, pick_list, progress_bar, row, scrollable, slider, text,
    text_input,
};
use iced::{
    Alignment, Element, Length, Padding, Settings, Subscription, Task, Theme, event, keyboard,
    mouse, time, window,
};
use std::collections::HashMap;
use std::io;
use std::mem;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tracing::{error, info, warn};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::config::{
    AccentColor, AudioOutputDevice, AudioStabilityMode, CloseBehavior, DeclarativeAction, EqPreset,
    InterfaceDensity, InterfaceLanguage, MissingDeviceBehavior, StartupScreen,
    TextScale, ThemeMode, TimeFormat, UpdateChannel, VolumeLevel,
};

pub(crate) const ALBUMS_GRID_COLUMNS: usize = 3;
const ARTIST_FOCUS_ORDER: [LibraryFocus; 3] =
    [LibraryFocus::Artists, LibraryFocus::Albums, LibraryFocus::Songs];
const GENRE_FOCUS_ORDER: [LibraryFocus; 3] =
    [LibraryFocus::Genres, LibraryFocus::Albums, LibraryFocus::Songs];
const FOLDER_FOCUS_ORDER: [LibraryFocus; 2] = [LibraryFocus::Folders, LibraryFocus::Songs];
const PREFERENCES_GENERAL_SCROLL_ID: &str = "preferences-general";
const PREFERENCES_APPEARANCE_SCROLL_ID: &str = "preferences-appearance";
const PREFERENCES_ACCESSIBILITY_SCROLL_ID: &str = "preferences-accessibility";
const PREFERENCES_AUDIO_SCROLL_ID: &str = "preferences-audio";

/// Layout spacing constants used throughout the UI.
pub(crate) mod spacing {
    pub const XS: f32 = 2.0;
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
    pub const LG: f32 = 8.0;
    pub const XL: f32 = 10.0;
    pub const XXL: f32 = 12.0;
    pub const SECTION: f32 = 16.0;
    pub const PANEL: f32 = 20.0;
    pub const REGION: f32 = 24.0;
}

pub(crate) struct QueueUpdate {
    pub preferred_index: usize,
    pub changed: bool,
}

/// Index mapping track file path to (artist_index, album_index, track_index)
/// for O(1) lookups instead of iterating the full catalog.
pub(crate) type TrackIndex = HashMap<PathBuf, (usize, usize, usize)>;

pub struct GrapeApp {
    pub(crate) catalog: Catalog,
    pub(crate) track_index: TrackIndex,
    pub(crate) player: Option<Player>,
    pub(crate) playlists: PlaylistManager,
    pub(crate) playback_queue: PlaybackQueue,
    pub(crate) ui: UiState,
    pub(crate) system_integration: Option<SystemIntegration>,
    pub(crate) cover_preloads: Vec<image::Handle>,
    pub(crate) last_finished_track: Option<PathBuf>,
    pub(crate) last_notified_track: Option<PathBuf>,
    pub(crate) last_notification_time: Option<std::time::Instant>,
    pub(crate) was_playing_before_focus_loss: bool,
    pub(crate) gapless_preloaded: bool,
    pub(crate) last_session_save: Option<std::time::Instant>,
}

impl GrapeApp {
    pub fn run(catalog: Catalog, library_root_override: Option<PathBuf>) -> iced::Result {
        let settings = Self::apply_font_settings(Settings::default());
        iced::application(
            move || Self::new(catalog.clone(), library_root_override.clone()),
            Self::update,
            Self::view,
        )
        .settings(settings)
        .title(Self::title)
        .subscription(Self::subscription)
        .theme(Self::theme)
        .run()
    }

    fn apply_font_settings(mut settings: Settings) -> Settings {
        settings.fonts = vec![
            include_bytes!(
                "../../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-Light.ttf"
            )
            .into(),
            include_bytes!(
                "../../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-Regular.ttf"
            )
            .into(),
            include_bytes!(
                "../../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-Medium.ttf"
            )
            .into(),
            include_bytes!(
                "../../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-SemiBold.ttf"
            )
            .into(),
            include_bytes!(
                "../../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-Bold.ttf"
            )
            .into(),
            include_bytes!(
                "../../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontMono-Regular.ttf"
            )
            .into(),
            include_bytes!(
                "../../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontMono-Medium.ttf"
            )
            .into(),
        ];
        settings.default_font = style::font_propo(Weight::Normal);
        settings
    }

    pub(crate) fn tab_label(&self, _tab: ActiveTab, label: &str) -> String {
        label.to_string()
    }

    pub(crate) fn theme_tokens(&self) -> style::ThemeTokens {
        style::ThemeTokens::from_settings(&self.ui.settings)
    }

    pub(crate) fn language(&self) -> InterfaceLanguage {
        self.ui.settings.interface_language.resolved()
    }

    pub(crate) fn strings(&self) -> &'static UiStrings {
        i18n::strings(self.ui.settings.interface_language)
    }

    pub(crate) fn build_track_index(catalog: &Catalog) -> TrackIndex {
        let mut index = HashMap::new();
        for (artist_idx, artist) in catalog.artists.iter().enumerate() {
            for (album_idx, album) in artist.albums.iter().enumerate() {
                for (track_idx, track) in album.tracks.iter().enumerate() {
                    index.insert(track.path.clone(), (artist_idx, album_idx, track_idx));
                }
            }
        }
        index
    }

    pub(crate) fn normalize_text(value: &str) -> String {
        value
            .nfkd()
            .filter(|character| !is_combining_mark(*character))
            .collect::<String>()
            .to_lowercase()
    }

    pub(crate) fn normalized_contains(query: &str, value: &str) -> bool {
        Self::normalize_text(value).contains(query)
    }

    pub(crate) fn codec_matches(query: &str, codec: Option<&str>) -> bool {
        codec.map(|value| Self::normalized_contains(query, value)).unwrap_or(false)
    }

    pub(crate) fn apply_limit<T>(items: Vec<T>, limit: usize) -> (Vec<T>, usize) {
        let total = items.len();
        let limited = items.into_iter().take(limit).collect();
        (limited, total)
    }

    pub(crate) fn move_selection<T: Clone>(
        items: &[T],
        current_id: Option<usize>,
        step: isize,
        id_fn: impl Fn(&T) -> usize,
    ) -> Option<T> {
        if items.is_empty() {
            return None;
        }
        let current_index = current_id
            .and_then(|id| items.iter().position(|item| id_fn(item) == id))
            .unwrap_or(0);
        let next_index = if step >= 0 {
            (current_index + step as usize).min(items.len().saturating_sub(1))
        } else {
            current_index.saturating_sub(step.abs() as usize)
        };
        items.get(next_index).cloned()
    }

    pub(crate) fn focus_order(&self) -> &'static [LibraryFocus] {
        match self.ui.active_tab {
            ActiveTab::Artists | ActiveTab::Albums => &ARTIST_FOCUS_ORDER,
            ActiveTab::Genres => &GENRE_FOCUS_ORDER,
            ActiveTab::Folders => &FOLDER_FOCUS_ORDER,
        }
    }

    pub(crate) fn append_task(task: &mut Task<UiMessage>, next: Task<UiMessage>) {
        let current = mem::replace(task, Task::none());
        *task = Task::batch([current, next]);
    }

    fn new(catalog: Catalog, library_root_override: Option<PathBuf>) -> Self {
        let mut settings = config::load_settings();
        if let Some(root) = library_root_override {
            settings.library_folder = root.display().to_string();
        }
        let (system_integration, integration_changed) =
            SystemIntegration::sync(None, &mut settings);
        if integration_changed {
            if let Err(err) = config::save_settings(&settings) {
                error!(error = %err, "Failed to persist system integration settings");
            }
        }
        let mut player = match Player::new_with_settings(&settings) {
            Ok(player) => Some(player),
            Err(err) => {
                error!(error = %err, "Failed to initialize audio player");
                None
            }
        };
        let mut audio_notice = None;
        if let Some(player) = player.as_mut() {
            if let Some(fallback) = player.take_last_fallback_notice() {
                let language = settings.interface_language.resolved();
                audio_notice = Some(fallback.notice(language));
                Self::apply_audio_fallback_to_settings(&mut settings, &fallback);
                if let Err(err) = config::save_settings(&settings) {
                    error!(error = %err, "Failed to persist audio fallback settings");
                }
            }
        }
        let playlists = PlaylistManager::load_or_default();
        let playback_queue = Self::playback_queue_from_playlist(&playlists);
        let mut ui = UiState::new(settings);
        ui.audio_notice = audio_notice;
        if !catalog.artists.is_empty() {
            ui.needs_initial_scan = false;
        }
        if let Some(active) = playlists.active() {
            ui.selection.selected_playlist = Some(playlists.active_index);
            ui.selection.playlist_name_draft = active.name.clone();
        }
        let track_index = Self::build_track_index(&catalog);

        // Restore session state if the setting is enabled.
        let session = if ui.settings.restore_last_session {
            config::load_session()
        } else {
            None
        };

        let mut app = Self {
            catalog,
            track_index,
            player,
            playlists,
            playback_queue,
            ui,
            system_integration,
            cover_preloads: Vec::new(),
            last_finished_track: None,
            last_notified_track: None,
            last_notification_time: None,
            was_playing_before_focus_loss: false,
            gapless_preloaded: false,
            last_session_save: None,
        };

        // Apply startup screen setting.
        match app.ui.settings.open_on {
            StartupScreen::Library => {
                app.ui.active_tab = ActiveTab::Albums;
                app.ui.library_focus = LibraryFocus::Albums;
            }
            StartupScreen::Playlists => {
                // Stay on default tab but open playlists panel.
            }
            StartupScreen::LastScreen | StartupScreen::Home => {
                // LastScreen is handled by session restore below; Home uses the default.
            }
        }

        // Apply restored session: restore active tab and queue position.
        if let Some(session) = session {
            // Only restore the active tab if open_on is LastScreen.
            if app.ui.settings.open_on == StartupScreen::LastScreen {
                match session.active_tab.as_str() {
                    "genres" => app.ui.active_tab = ActiveTab::Genres,
                    "albums" => app.ui.active_tab = ActiveTab::Albums,
                    "folders" => app.ui.active_tab = ActiveTab::Folders,
                    _ => app.ui.active_tab = ActiveTab::Artists,
                }
                app.ui.library_focus = match app.ui.active_tab {
                    ActiveTab::Artists => LibraryFocus::Artists,
                    ActiveTab::Genres => LibraryFocus::Genres,
                    ActiveTab::Albums => LibraryFocus::Albums,
                    ActiveTab::Folders => LibraryFocus::Folders,
                };
            }
            app.playback_queue.set_index(session.queue_index);
            if let Some(track_path) = session.track_path {
                if let Some(now_playing) = app.playback_queue.current() {
                    if now_playing.path == track_path {
                        let ui_track = app.ui_track_from_now_playing(&now_playing);
                        app.ui.selection.selected_track = Some(ui_track);
                        app.ui.playback.position =
                            Duration::from_secs_f64(session.position_secs);
                    }
                }
            }
        }

        app
    }

    fn title(&self) -> String {
        "Grape".to_string()
    }

    fn theme(&self) -> Theme {
        let mode = if self.ui.settings.follow_system_theme {
            if config::system_prefers_dark() {
                self.ui.settings.theme_mode.dark_variant()
            } else {
                self.ui.settings.theme_mode.light_variant()
            }
        } else {
            self.ui.settings.theme_mode
        };
        match mode {
            ThemeMode::Latte
            | ThemeMode::GruvboxLight
            | ThemeMode::EverblushLight
            | ThemeMode::KanagawaLight
            | ThemeMode::KanagawaJournal => Theme::Light,
            ThemeMode::Frappe
            | ThemeMode::Macchiato
            | ThemeMode::Mocha
            | ThemeMode::GruvboxDark
            | ThemeMode::EverblushDark
            | ThemeMode::KanagawaDark => Theme::Dark,
        }
    }

    fn subscription(&self) -> Subscription<UiMessage> {
        let mut subscriptions = Vec::new();

        if self.ui.menu_open {
            subscriptions.push(event::listen_with(|event, status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::CloseMenu)
                }
                event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    if status == event::Status::Ignored =>
                {
                    Some(UiMessage::CloseMenu)
                }
                _ => None,
            }));
        }

        if self.ui.playlist_open {
            subscriptions.push(event::listen_with(|event, _status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::ClosePlaylist)
                }
                _ => None,
            }));
        }

        if self.ui.queue_open {
            subscriptions.push(event::listen_with(|event, _status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::CloseQueue)
                }
                _ => None,
            }));
        }

        if self.ui.preferences_open {
            subscriptions.push(event::listen_with(|event, _status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::ClosePreferences)
                }
                _ => None,
            }));
        }

        if self.ui.speed_popup_open {
            subscriptions.push(event::listen_with(|event, _status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::ToggleSpeedPopup)
                }
                _ => None,
            }));
        }

        if self.ui.mini_player {
            subscriptions.push(event::listen_with(|event, _status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::ToggleMiniPlayer)
                }
                _ => None,
            }));
        }

        if !self.ui.menu_open
            && !self.ui.playlist_open
            && !self.ui.queue_open
            && !self.ui.preferences_open
        {
            subscriptions.push(event::listen_with(|event, status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                    if status == event::Status::Ignored =>
                {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                            Some(UiMessage::NavigateLibrary(LibraryNavigation::Up))
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                            Some(UiMessage::NavigateLibrary(LibraryNavigation::Down))
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                            Some(UiMessage::NavigateLibrary(LibraryNavigation::Left))
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                            Some(UiMessage::NavigateLibrary(LibraryNavigation::Right))
                        }
                        keyboard::Key::Named(keyboard::key::Named::Tab) => {
                            if modifiers.shift() {
                                Some(UiMessage::NavigateLibrary(LibraryNavigation::PreviousPanel))
                            } else {
                                Some(UiMessage::NavigateLibrary(LibraryNavigation::NextPanel))
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::Enter) => {
                            Some(UiMessage::ActivateSelection)
                        }
                        keyboard::Key::Named(keyboard::key::Named::Space) => {
                            Some(UiMessage::Playback(PlaybackMessage::TogglePlayPause))
                        }
                        keyboard::Key::Character(ref ch) => match ch.as_str() {
                            "n" if !modifiers.shift() => {
                                Some(UiMessage::Playback(PlaybackMessage::NextTrack))
                            }
                            "p" if !modifiers.shift() => {
                                Some(UiMessage::Playback(PlaybackMessage::PreviousTrack))
                            }
                            "+" | "=" => Some(UiMessage::VolumeUp),
                            "-" => Some(UiMessage::VolumeDown),
                            "m" if !modifiers.shift() => Some(UiMessage::ToggleMiniPlayer),
                            _ => None,
                        },
                        _ => None,
                    }
                }
                _ => None,
            }));
        }

        // Listen for window focus changes (used for pause_on_focus_loss).
        subscriptions.push(event::listen_with(|event, _status, _| match event {
            event::Event::Window(window::Event::Focused) => {
                Some(UiMessage::WindowFocusChanged(true))
            }
            event::Event::Window(window::Event::Unfocused) => {
                Some(UiMessage::WindowFocusChanged(false))
            }
            _ => None,
        }));

        if self.ui.needs_initial_scan {
            subscriptions
                .push(time::every(Duration::from_millis(16)).map(|_| UiMessage::StartInitialScan));
        }

        if self.ui.scan_status.is_some() {
            subscriptions
                .push(time::every(Duration::from_millis(120)).map(|_| UiMessage::ScanTick));
        }

        let inline_volume_target = if self.ui.inline_volume_bar_open { 1.0 } else { 0.0 };
        let limit_cpu = self.ui.settings.limit_cpu_during_playback && self.ui.playback.is_playing;
        let animations_off = !self.ui.settings.ui_animations;
        if (self.ui.inline_volume_visibility - inline_volume_target).abs() > 0.01
            && !limit_cpu
            && !animations_off
        {
            subscriptions
                .push(time::every(Duration::from_millis(16)).map(|_| UiMessage::AnimationTick));
        }

        if self.ui.playback.is_playing {
            let tick_interval = if limit_cpu { 1000 } else { 225 };
            subscriptions
                .push(time::every(Duration::from_millis(tick_interval)).map(|_| UiMessage::PlaybackTick));
        }

        Subscription::batch(subscriptions)
    }
}
