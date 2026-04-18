use super::*;

/// Side-effect flags computed from a message in a single pass.
struct MessageFlags {
    select_genre_album: bool,
    fetch_metadata: bool,
    reset_limits: bool,
    refresh_preloads: bool,
    persist: bool,
    refresh_system_integration: bool,
    refresh_audio: bool,
    selected_artist: Option<UiArtist>,
    selected_album: Option<UiAlbum>,
    selected_folder: Option<UiFolder>,
}

impl MessageFlags {
    fn from_message(message: &UiMessage) -> Self {
        let mut flags = Self {
            select_genre_album: false,
            fetch_metadata: false,
            reset_limits: false,
            refresh_preloads: false,
            persist: false,
            refresh_system_integration: false,
            refresh_audio: false,
            selected_artist: None,
            selected_album: None,
            selected_folder: None,
        };
        match message {
            // Selection messages
            UiMessage::SelectArtist(artist) => {
                flags.selected_artist = Some(artist.clone());
                flags.fetch_metadata = true;
                flags.reset_limits = true;
                flags.refresh_preloads = true;
            }
            UiMessage::SelectAlbum(album) => {
                flags.selected_album = Some(album.clone());
                flags.fetch_metadata = true;
                flags.refresh_preloads = true;
            }
            UiMessage::SelectGenre(_) => {
                flags.select_genre_album = true;
                flags.fetch_metadata = true;
                flags.reset_limits = true;
                flags.refresh_preloads = true;
            }
            UiMessage::SelectFolder(folder) => {
                flags.selected_folder = Some(folder.clone());
                flags.fetch_metadata = true;
                flags.reset_limits = true;
                flags.refresh_preloads = true;
            }

            // Tab/search changes
            UiMessage::TabSelected(_) => {
                flags.reset_limits = true;
                flags.refresh_preloads = true;
            }
            UiMessage::Search(SearchMessage::QueryChanged(_))
            | UiMessage::Search(SearchMessage::SortChanged(_))
            | UiMessage::Search(SearchMessage::ToggleFilter(_)) => {
                flags.reset_limits = true;
                flags.refresh_preloads = true;
            }

            // Load more
            UiMessage::LoadMoreArtists
            | UiMessage::LoadMoreAlbums
            | UiMessage::LoadMoreTracks
            | UiMessage::LoadMoreGenres
            | UiMessage::LoadMoreFolders => {
                flags.refresh_preloads = true;
            }
            UiMessage::LibraryScanCompleted(_) => {
                flags.refresh_preloads = true;
            }

            // Audio settings (persist + refresh audio)
            UiMessage::SetAudioOutputDevice(_)
            | UiMessage::SetMissingDeviceBehavior(_)
            | UiMessage::SetNormalizeVolume(_)
            | UiMessage::SetVolumeLevel(_)
            | UiMessage::SetEqEnabled(_)
            | UiMessage::SetEqPreset(_)
            | UiMessage::SetEqBandGain(_, _)
            | UiMessage::ResetEq
            | UiMessage::SetAudioStabilityMode(_)
            | UiMessage::SetAudioDebugLogs(_) => {
                flags.persist = true;
                flags.refresh_audio = true;
            }
            UiMessage::SetDefaultVolume(_) | UiMessage::VolumeUp | UiMessage::VolumeDown => {
                flags.persist = true;
                flags.refresh_audio = true;
            }
            UiMessage::SetPlaybackSpeed(_) => {
                flags.persist = true;
                flags.refresh_audio = true;
            }

            // System integration settings (persist + refresh system)
            UiMessage::SetNotificationsEnabled(_)
            | UiMessage::SetNowPlayingNotifications(_)
            | UiMessage::SetSystemTrayEnabled(_)
            | UiMessage::SetHardwareAcceleration(_)
            | UiMessage::SetLaunchAtStartup(_) => {
                flags.persist = true;
                flags.refresh_system_integration = true;
            }
            UiMessage::SetAdvancedShortcuts(_) => {
                flags.persist = true;
                flags.refresh_system_integration = true;
            }
            UiMessage::ResetPreferences => {
                flags.persist = true;
                flags.refresh_system_integration = true;
            }

            // Settings that only need persist
            UiMessage::SetThemeMode(_)
            | UiMessage::SetFollowSystemTheme(_)
            | UiMessage::SetAccentColor(_)
            | UiMessage::SetAccentAuto(_)
            | UiMessage::SetTextScale(_)
            | UiMessage::SetInterfaceDensity(_)
            | UiMessage::SetTransparencyBlur(_)
            | UiMessage::SetUiAnimations(_)
            | UiMessage::SetAccessibilityLargeText(_)
            | UiMessage::SetAccessibilityHighContrast(_)
            | UiMessage::SetAccessibilityReduceMotion(_)
            | UiMessage::SetIncreaseContrast(_)
            | UiMessage::SetReduceTransparency(_)
            | UiMessage::SetAccessibleTextSize(_)
            | UiMessage::SetReduceAnimations(_)
            | UiMessage::SetReduceTransitions(_)
            | UiMessage::SetHighlightKeyboardFocus(_)
            | UiMessage::SetDefaultPlaybackSpeed(_)
            | UiMessage::SetPauseOnFocusLoss(_)
            | UiMessage::SetGaplessPlayback(_)
            | UiMessage::SetCrossfadeSeconds(_)
            | UiMessage::SetAutomixEnabled(_)
            | UiMessage::SetRestoreLastSession(_)
            | UiMessage::SetOpenOn(_)
            | UiMessage::SetCloseBehavior(_)
            | UiMessage::SetInterfaceLanguage(_)
            | UiMessage::SetTimeFormat(_)
            | UiMessage::SetAutoCheckUpdates(_)
            | UiMessage::SetUpdateChannel(_)
            | UiMessage::SetAutoInstallUpdates(_)
            | UiMessage::LibraryFolderChanged(_)
            | UiMessage::LibraryFolderPicked(_)
            | UiMessage::SetAutoScanOnLaunch(_)
            | UiMessage::CachePathChanged(_)
            | UiMessage::SetLimitCpuDuringPlayback(_) => {
                flags.persist = true;
            }

            _ => {}
        }
        flags
    }
}

impl GrapeApp {
    pub(crate) fn update(&mut self, message: UiMessage) -> Task<UiMessage> {
        let mut flags = MessageFlags::from_message(&message);
        let mut task = Task::none();
        let mut handled_playback_tick = false;
        match &message {
            UiMessage::PlaybackTick => {
                self.sync_playback_state();
                self.ui.playback.update_animated_progress();
                self.maybe_auto_advance_track();
                handled_playback_tick = true;
                // Periodically persist session state (every ~5s).
                let now = std::time::Instant::now();
                let should_save = self
                    .last_session_save
                    .map(|last| now.duration_since(last) >= Duration::from_secs(5))
                    .unwrap_or(true);
                if should_save {
                    self.last_session_save = Some(now);
                    self.save_session_state();
                }
            }
            UiMessage::AnimationTick => {
                self.ui.update_inline_volume_visibility();
            }
            UiMessage::SelectTrack(track) => {
                self.handle_track_selection(track);
                self.save_session_state();
            }
            UiMessage::NavigateLibrary(navigation) => {
                self.handle_library_navigation(*navigation);
            }
            UiMessage::ActivateSelection => {
                self.activate_selection();
            }
            UiMessage::SelectPlaylist(index) => {
                if self.playlists.set_active(*index) {
                    if let Some(active) = self.playlists.active() {
                        self.ui.selection.selected_playlist = Some(*index);
                        self.ui.selection.playlist_name_draft = active.name.clone();
                    }
                    self.refresh_playback_queue(None);
                }
                self.ui.selection.playlist_drag_source = None;
            }
            UiMessage::Playback(playback_message) => {
                self.handle_playback_message(playback_message);
            }
            UiMessage::SaveAlbumMetadata => {
                self.handle_album_metadata_save();
            }
            UiMessage::EnrichAlbumMetadata => {
                if let Some(album) = self.ui.selection.selected_album.clone() {
                    task = self.request_album_metadata(&album, true, true);
                }
            }
            UiMessage::CreatePlaylist => {
                let strings = i18n::strings(self.ui.settings.interface_language);
                let (index, name) = self.playlists.create_playlist(
                    self.ui.selection.playlist_name_draft.clone(),
                    strings.playlist_default_name,
                );
                self.ui.selection.selected_playlist = Some(index);
                self.ui.selection.playlist_name_draft = name;
                self.ui.selection.playlist_drag_source = None;
                self.refresh_playback_queue(None);
                self.persist_playlist();
            }
            UiMessage::RenamePlaylist => {
                let strings = i18n::strings(self.ui.settings.interface_language);
                let index = self.playlists.active_index;
                if let Some(name) = self.playlists.rename_playlist(
                    index,
                    self.ui.selection.playlist_name_draft.clone(),
                    strings.playlist_default_name,
                ) {
                    self.ui.selection.playlist_name_draft = name;
                    self.persist_playlist();
                }
            }
            UiMessage::DeletePlaylist => {
                let index = self.playlists.active_index;
                if self.playlists.remove_playlist(index) {
                    if let Some(active) = self.playlists.active() {
                        self.ui.selection.selected_playlist = Some(self.playlists.active_index);
                        self.ui.selection.playlist_name_draft = active.name.clone();
                    }
                    self.ui.selection.playlist_drag_source = None;
                    self.refresh_playback_queue(None);
                    self.persist_playlist();
                }
            }
            UiMessage::MovePlaylistItemUp(index) => {
                if *index > 0 {
                    self.playlist_reorder(*index, (*index).saturating_sub(1));
                }
            }
            UiMessage::MovePlaylistItemDown(index) => {
                let can_move = self
                    .playlists
                    .active()
                    .map(|playlist| *index + 1 < playlist.items.len())
                    .unwrap_or(false);
                if can_move {
                    self.playlist_reorder(*index, *index + 1);
                }
            }
            UiMessage::StartPlaylistItemDrag(index) => {
                if self.ui.selection.playlist_drag_source == Some(*index) {
                    self.ui.selection.playlist_drag_source = None;
                } else {
                    self.ui.selection.playlist_drag_source = Some(*index);
                }
            }
            UiMessage::MovePlaylistItemDrag { from, to } => {
                if self
                    .playlists
                    .active()
                    .map(|playlist| *from < playlist.items.len() && *to < playlist.items.len())
                    .unwrap_or(false)
                {
                    self.playlist_reorder(*from, *to);
                }
                self.ui.selection.playlist_drag_source = None;
            }
            UiMessage::DeletePlaylistItem(index) => {
                self.playlist_remove(*index);
                self.ui.selection.playlist_drag_source = None;
            }
            UiMessage::SavePlaylistOrder => {
                self.playlist_save_order();
                self.ui.selection.playlist_drag_source = None;
            }
            UiMessage::AddSelectedTrackToPlaylist => {
                if let Some(track) = self.ui.selection.selected_track.as_ref() {
                    let now_playing = Self::now_playing_from_ui_track(track);
                    self.playlist_add(now_playing);
                }
            }
            UiMessage::ClearQueue => {
                self.playlist_clear();
            }
            UiMessage::MoveQueueItemUp(index) => {
                if *index > 0 {
                    self.playlist_reorder(*index, (*index).saturating_sub(1));
                }
            }
            UiMessage::MoveQueueItemDown(index) => {
                if *index + 1 < self.playback_queue.items().len() {
                    self.playlist_reorder(*index, *index + 1);
                }
            }
            UiMessage::RemoveQueueItem(index) => {
                self.playlist_remove(*index);
            }
            UiMessage::OpenPlaylist => {
                self.ui.playlist_open = true;
            }
            UiMessage::ClosePlaylist => {
                self.ui.playlist_open = false;
            }
            UiMessage::OpenPreferences => {
                task =
                    Task::batch([task, self.restore_preferences_scroll(self.ui.preferences_tab)]);
            }
            UiMessage::PreferencesTabSelected(tab) => {
                task = Task::batch([task, self.restore_preferences_scroll(*tab)]);
            }
            UiMessage::WindowMinimize => {
                task = window::oldest().then(|id| {
                    if let Some(id) = id {
                        window::minimize(id, true)
                    } else {
                        Task::none()
                    }
                });
            }
            UiMessage::WindowToggleMaximize => {
                task = window::oldest().then(|id| {
                    if let Some(id) = id {
                        window::toggle_maximize(id)
                    } else {
                        Task::none()
                    }
                });
            }
            UiMessage::WindowClose => {
                if self.ui.settings.close_behavior == CloseBehavior::MinimizeToTray
                    && self.ui.settings.system_tray_enabled
                {
                    // Minimize to tray instead of quitting.
                    task = window::oldest().then(|id| {
                        if let Some(id) = id {
                            window::minimize(id, true)
                        } else {
                            Task::none()
                        }
                    });
                } else {
                    task = window::oldest().then(|id| {
                        if let Some(id) = id {
                            window::close(id)
                        } else {
                            Task::none()
                        }
                    });
                }
            }
            UiMessage::LibraryFolderPicked(path) => {
                if let Some(path) = path {
                    let root = PathBuf::from(path);
                    if !root.is_dir() {
                        warn!(
                            path = %root.display(),
                            "Selected library folder is invalid"
                        );
                    } else {
                        task = self.begin_scan(root, false);
                    }
                }
            }
            UiMessage::PickLibraryFolder => {
                task = Task::perform(
                    async {
                        rfd::FileDialog::new().pick_folder().map(|path| path.display().to_string())
                    },
                    UiMessage::LibraryFolderPicked,
                );
            }
            UiMessage::ClearCache => {
                task = self.handle_declarative_action(DeclarativeAction::ClearCache);
            }
            UiMessage::ClearHistory => {
                if let Err(err) = config::clear_history() {
                    error!(error = %err, "Failed to clear local history");
                } else {
                    info!("Local history cleared");
                }
            }
            UiMessage::OpenLogsFolder => {
                self.open_logs_folder();
            }
            UiMessage::ReindexLibrary => {
                task = self.handle_declarative_action(DeclarativeAction::ReindexLibrary);
            }
            UiMessage::ResetAudioEngine => {
                task = self.handle_declarative_action(DeclarativeAction::ResetAudioEngine);
            }
            UiMessage::ConfirmDeclarativeAction(action) => {
                if self.ui.pending_action == Some(*action) {
                    task = self.handle_declarative_action(*action);
                }
            }
            UiMessage::StartInitialScan => {
                if self.ui.needs_initial_scan {
                    self.ui.needs_initial_scan = false;
                    task = self.begin_scan_from_settings(true);
                }
            }
            UiMessage::ScanTick => {
                if let Some(status) = self.ui.scan_status.as_mut() {
                    let next = status.progress + 0.02;
                    status.progress = if next >= 0.95 { 0.2 } else { next };
                }
            }
            UiMessage::LibraryScanCompleted(result) => {
                let scan_root = self.ui.scan_status.as_ref().map(|status| status.root.clone());
                self.ui.scan_status = None;
                match result {
                    Ok(catalog) => {
                        let catalog = catalog.clone();
                        let root = scan_root.or_else(|| self.library_root()).unwrap_or_default();
                        let has_root_album = catalog
                            .artists
                            .iter()
                            .any(|artist| artist.albums.iter().any(|album| album.path == root));
                        let total_artists = catalog.artists.len();
                        let total_albums: usize =
                            catalog.artists.iter().map(|a| a.albums.len()).sum();
                        let total_tracks: usize = catalog
                            .artists
                            .iter()
                            .flat_map(|a| &a.albums)
                            .map(|a| a.tracks.len())
                            .sum();
                        let albums_with_cover: usize = catalog
                            .artists
                            .iter()
                            .flat_map(|a| &a.albums)
                            .filter(|a| a.cover.is_some())
                            .count();
                        info!(
                            path = %root.display(),
                            root_tracks = has_root_album,
                            artists = total_artists,
                            albums = total_albums,
                            tracks = total_tracks,
                            albums_with_cover = albums_with_cover,
                            "Library scan completed"
                        );
                        self.track_index = Self::build_track_index(&catalog);
                        self.catalog = catalog;
                        self.ui.selection = SelectionState::default();
                        self.ui.album_genre_draft.clear();
                        self.ui.album_year_draft.clear();
                        self.ui.search = SearchState::default();
                        self.ui.list_limits = ListLimits::default();
                    }
                    Err(err) => {
                        error!(error = %err, "Failed to scan library");
                    }
                }
            }
            UiMessage::LoadMoreArtists => {
                self.ui.list_limits.artists += 50;
            }
            UiMessage::LoadMoreAlbums => {
                self.ui.list_limits.albums += 30;
            }
            UiMessage::LoadMoreTracks => {
                self.ui.list_limits.tracks += 50;
            }
            UiMessage::LoadMoreGenres => {
                self.ui.list_limits.genres += 50;
            }
            UiMessage::LoadMoreFolders => {
                self.ui.list_limits.folders += 50;
            }
            UiMessage::ExportPlaylistM3u => {
                if let Some(playlist) = self.playlists.active() {
                    let playlist_name = playlist.name.clone();
                    let items: Vec<(String, String, u32, PathBuf)> = playlist
                        .items
                        .iter()
                        .map(|item| {
                            (
                                item.artist.clone(),
                                item.title.clone(),
                                item.duration_secs,
                                item.path.clone(),
                            )
                        })
                        .collect();
                    task = Task::perform(
                        async move { Self::export_playlist_m3u(&playlist_name, &items) },
                        UiMessage::PlaylistExported,
                    );
                }
            }
            UiMessage::AlbumMetadataFetched {
                album_id,
                artist,
                result,
                enrichment_confirmed,
            } => match result {
                Ok(Some(metadata)) => {
                    if let Some(root) = self.library_root() {
                        if let Some(album) = self.album_entry_by_id_mut(*album_id) {
                            crate::library::merge_album_online_metadata(
                                &root,
                                artist,
                                album,
                                metadata,
                                *enrichment_confirmed,
                            );
                        }
                        let selected_year = self
                            .album_entry_by_id(*album_id)
                            .map(|(_, album)| album.year)
                            .filter(|year| *year > 0)
                            .map(|year| year as u32);
                        if let Some(selected_album) = self.ui.selection.selected_album.as_mut() {
                            if selected_album.id == *album_id {
                                selected_album.year = selected_year;
                            }
                        }
                        self.refresh_album_metadata_drafts();
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    warn!(error = %error, "Failed to fetch online album metadata");
                }
            },
            UiMessage::WindowFocusChanged(focused) => {
                if self.ui.settings.pause_on_focus_loss {
                    if !focused {
                        // Window lost focus: remember if we were playing, then pause.
                        self.was_playing_before_focus_loss = self.ui.playback.is_playing;
                        if self.ui.playback.is_playing {
                            if let Some(player) = &mut self.player {
                                player.pause();
                            }
                        }
                    } else if self.was_playing_before_focus_loss {
                        // Window regained focus: resume if we paused on focus loss.
                        self.was_playing_before_focus_loss = false;
                        if let Some(player) = &mut self.player {
                            player.play();
                        }
                    }
                }
            }
            _ => {}
        }
        self.ui.update(message);
        if flags.refresh_system_integration {
            let (system_integration, changed) =
                SystemIntegration::sync(self.system_integration.take(), &mut self.ui.settings);
            self.system_integration = system_integration;
            if changed {
                flags.persist = true;
            }
        }
        self.apply_system_actions(&mut task);
        if let Some(artist) = flags.selected_artist {
            self.apply_artist_selection(artist);
        }
        if let Some(album) = flags.selected_album {
            self.apply_album_selection(album);
        }
        if let Some(folder) = flags.selected_folder {
            self.apply_folder_selection(folder);
        }
        if flags.reset_limits {
            self.ui.list_limits = ListLimits::default();
        }
        if flags.select_genre_album && self.ui.active_tab == ActiveTab::Genres {
            if let Some(album) = self.filtered_albums_from_catalog().into_iter().next() {
                let album_id = album.id;
                self.ui.selection.selected_album = Some(album);
                self.ui.selection.selected_track =
                    self.album_entry_by_id(album_id).and_then(|(artist, entry)| {
                        self.filtered_tracks_for_album(artist, entry).into_iter().next()
                    });
            } else {
                self.ui.selection.selected_album = None;
                self.ui.selection.selected_track = None;
            }
        }
        if flags.fetch_metadata {
            if let Some(album) = self.ui.selection.selected_album.clone() {
                task = Task::batch([task, self.request_album_metadata(&album, false, false)]);
            }
        }
        if flags.persist {
            if let Err(err) = config::save_settings(&self.ui.settings) {
                error!(error = %err, "Failed to save preferences");
            }
        }
        if flags.refresh_audio {
            self.apply_audio_settings();
        }
        if flags.refresh_preloads {
            self.refresh_cover_preloads();
        }
        if handled_playback_tick {
            if !self.ui.settings.ui_animations
                || self.ui.settings.reduce_animations
                || self.ui.settings.accessibility_reduce_motion
            {
                self.ui.playback.animated_progress =
                    progress_ratio(self.ui.playback.position, self.ui.playback.duration);
            }
        } else {
            self.sync_playback_state();
            if !self.ui.settings.ui_animations
                || self.ui.settings.reduce_animations
                || self.ui.settings.accessibility_reduce_motion
            {
                self.ui.playback.animated_progress =
                    progress_ratio(self.ui.playback.position, self.ui.playback.duration);
            } else {
                self.ui.playback.update_animated_progress();
            }
        }
        task
    }
}
