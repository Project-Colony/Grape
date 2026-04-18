use super::*;

impl GrapeApp {
    pub(crate) fn handle_track_selection(&mut self, track: &UiTrack) {
        let now_playing = Self::now_playing_from_ui_track(track);
        if let Some(queue_update) = self
            .queue_tracks_in_album_order()
            .and_then(|tracks| self.playlist_replace_with_tracks(tracks, track))
        {
            self.refresh_playback_queue(Some(queue_update.preferred_index));
            if queue_update.changed {
                self.persist_playlist();
            }
        } else {
            self.playlist_add(now_playing.clone());
        }
        let load_ok = {
            let Some(player) = &mut self.player else {
                self.ui.error_message = Some("Audio engine not available".to_string());
                return;
            };
            if let Err(err) = player.load(&track.path) {
                error!(error = %err, path = %track.path.display(), "Failed to load track");
                self.ui.error_message = Some(format!("Failed to load: {}", track.title));
                return;
            }
            player.play();
            true
        };
        if load_ok {
            self.ui.error_message = None;
            self.record_recently_played(&now_playing);
            self.maybe_notify_now_playing(&now_playing);
        }
    }

    pub(crate) fn now_playing_from_ui_track(track: &UiTrack) -> NowPlaying {
        NowPlaying {
            artist: track.artist.clone(),
            album: track.album.clone(),
            title: track.title.clone(),
            duration_secs: u32::try_from(track.duration.as_secs()).unwrap_or(u32::MAX),
            path: track.path.clone(),
        }
    }

    pub(crate) fn playback_queue_from_playlist(playlists: &PlaylistManager) -> PlaybackQueue {
        let mut queue = PlaybackQueue::default();
        let items = playlists.active().map(|playlist| playlist.items.clone()).unwrap_or_default();
        queue.set_queue(items);
        queue
    }

    pub(crate) fn ui_track_from_now_playing(&self, now_playing: &NowPlaying) -> UiTrack {
        // O(1) path-based lookup using the track index
        if let Some(&(artist_idx, album_idx, track_idx)) = self.track_index.get(&now_playing.path) {
            if let Some(artist) = self.catalog.artists.get(artist_idx) {
                if let Some(album) = artist.albums.get(album_idx) {
                    if let Some(track) = album.tracks.get(track_idx) {
                        let artist_name = track.artist.as_deref().unwrap_or(&artist.name);
                        return UiTrack {
                            id: track_idx,
                            title: track.title.clone(),
                            album: album.title.clone(),
                            artist: artist_name.to_string(),
                            track_number: Some(track.number as u32),
                            duration: Duration::from_secs(track.duration_secs as u64),
                            path: track.path.clone(),
                            cover_path: album.cover.as_ref().map(|cover| cover.cached_path.clone()),
                            normalized_title: Self::normalize_text(&track.title),
                            normalized_artist: Self::normalize_text(artist_name),
                            normalized_album: Self::normalize_text(&album.title),
                        };
                    }
                }
            }
        }
        // Fallback: fuzzy match by normalized title/artist/album
        let normalized_title = Self::normalize_text(&now_playing.title);
        if !normalized_title.is_empty() {
            let normalized_album = Self::normalize_text(&now_playing.album);
            let normalized_artist = Self::normalize_text(&now_playing.artist);
            for artist in &self.catalog.artists {
                let artist_match = normalized_artist.is_empty()
                    || Self::normalize_text(&artist.name) == normalized_artist;
                if !artist_match {
                    continue;
                }
                for album in &artist.albums {
                    let album_match = normalized_album.is_empty()
                        || Self::normalize_text(&album.title) == normalized_album;
                    if !album_match {
                        continue;
                    }
                    for (id, track) in album.tracks.iter().enumerate() {
                        if Self::normalize_text(&track.title) == normalized_title {
                            let artist_name = track.artist.as_deref().unwrap_or(&artist.name);
                            return UiTrack {
                                id,
                                title: track.title.clone(),
                                album: album.title.clone(),
                                artist: artist_name.to_string(),
                                track_number: Some(track.number as u32),
                                duration: Duration::from_secs(track.duration_secs as u64),
                                path: track.path.clone(),
                                cover_path: album
                                    .cover
                                    .as_ref()
                                    .map(|cover| cover.cached_path.clone()),
                                normalized_title: Self::normalize_text(&track.title),
                                normalized_artist: Self::normalize_text(artist_name),
                                normalized_album: Self::normalize_text(&album.title),
                            };
                        }
                    }
                }
            }
        }
        UiTrack {
            id: 0,
            title: now_playing.title.clone(),
            album: now_playing.album.clone(),
            artist: now_playing.artist.clone(),
            track_number: None,
            duration: Duration::from_secs(now_playing.duration_secs as u64),
            path: now_playing.path.clone(),
            cover_path: None,
            normalized_title: Self::normalize_text(&now_playing.title),
            normalized_artist: Self::normalize_text(&now_playing.artist),
            normalized_album: Self::normalize_text(&now_playing.album),
        }
    }

    pub(crate) fn playlist_add(&mut self, now_playing: NowPlaying) {
        self.playlists.add(now_playing);
        let preferred_index =
            self.playlists.active().map(|playlist| playlist.items.len().saturating_sub(1));
        self.refresh_playback_queue(preferred_index);
        self.persist_playlist();
    }

    pub(crate) fn playlist_save_order(&mut self) {
        self.refresh_playback_queue(None);
        self.persist_playlist();
    }

    pub(crate) fn playlist_remove(&mut self, index: usize) -> Option<NowPlaying> {
        let removed = self.playlists.delete_item(index);
        if removed.is_some() {
            self.refresh_playback_queue(None);
            self.persist_playlist();
        }
        removed
    }

    pub(crate) fn playlist_reorder(&mut self, from: usize, to: usize) -> bool {
        let changed = self.playlists.move_item(from, to);
        if changed {
            self.refresh_playback_queue(None);
            self.persist_playlist();
        }
        changed
    }

    pub(crate) fn playlist_clear(&mut self) {
        self.playlists.clear();
        self.refresh_playback_queue(None);
        self.persist_playlist();
    }

    fn playlist_replace_with_tracks(
        &mut self,
        tracks: Vec<UiTrack>,
        selected_track: &UiTrack,
    ) -> Option<QueueUpdate> {
        let items: Vec<NowPlaying> = tracks.iter().map(Self::now_playing_from_ui_track).collect();
        let preferred_index = tracks.iter().position(|track| track.path == selected_track.path)?;
        let mut changed = true;
        if let Some(active) = self.playlists.active() {
            changed = active.items.len() != items.len()
                || !active
                    .items
                    .iter()
                    .zip(items.iter())
                    .all(|(current, next)| current.path == next.path);
        }
        if changed {
            self.playlists.set_items(items);
        }
        Some(QueueUpdate { preferred_index, changed })
    }

    fn queue_tracks_in_album_order(&self) -> Option<Vec<UiTrack>> {
        let mut tracks = self.current_tracks();
        if tracks.is_empty() {
            return None;
        }
        if self.ui.selection.selected_album.is_some() || self.ui.selection.selected_folder.is_some()
        {
            tracks.sort_by(|a, b| {
                a.track_number
                    .unwrap_or(u32::MAX)
                    .cmp(&b.track_number.unwrap_or(u32::MAX))
                    .then_with(|| {
                        Self::normalize_text(&a.title).cmp(&Self::normalize_text(&b.title))
                    })
            });
        }
        Some(tracks)
    }

    pub(crate) fn refresh_playback_queue(&mut self, preferred_index: Option<usize>) {
        let items = self
            .playlists
            .active()
            .map(|playlist| playlist.items.clone())
            .unwrap_or_default();
        let index = match preferred_index {
            Some(index) => index.min(items.len().saturating_sub(1)),
            None => {
                let current = self.playback_queue.current();
                current
                    .as_ref()
                    .and_then(|now_playing| {
                        items.iter().position(|item| item.path == now_playing.path)
                    })
                    .unwrap_or(0)
            }
        };
        self.playback_queue.set_queue(items);
        self.playback_queue.set_index(index);
    }

    pub(crate) fn persist_playlist(&self) {
        if let Err(err) = self.playlists.save() {
            warn!(error = %err, "Failed to persist playlist");
        }
    }

    pub(crate) fn refresh_cover_preloads(&mut self) {
        self.cover_preloads.clear();
        let mut handles = Vec::new();
        let albums = self.filtered_albums_from_catalog();
        for album in albums.into_iter().take(self.ui.list_limits.albums) {
            if let Some(path) = album.cover_path {
                handles.push(image::Handle::from_path(path));
            }
        }
        let tracks = self.current_tracks();
        for track in tracks.into_iter().take(self.ui.list_limits.tracks) {
            if let Some(path) = track.cover_path {
                handles.push(image::Handle::from_path(path));
            }
        }
        self.cover_preloads = handles;
    }

    pub(crate) fn load_from_queue(&mut self, now_playing: Option<NowPlaying>) {
        let Some(now_playing) = now_playing else {
            return;
        };
        let load_ok = {
            let Some(player) = &mut self.player else {
                self.ui.error_message = Some("Audio engine not available".to_string());
                return;
            };
            if let Err(err) = player.load(&now_playing.path) {
                error!(error = %err, path = %now_playing.path.display(), "Failed to load track");
                self.ui.error_message = Some(format!("Failed to load: {}", now_playing.title));
                return;
            }
            player.play();
            true
        };
        if load_ok {
            self.ui.error_message = None;
            self.record_recently_played(&now_playing);
            self.maybe_notify_now_playing(&now_playing);
        }
        self.ui.selection.selected_track = Some(self.ui_track_from_now_playing(&now_playing));
    }

    pub(crate) fn maybe_notify_now_playing(&mut self, now_playing: &NowPlaying) {
        if self.last_notified_track.as_ref().is_some_and(|path| path == &now_playing.path) {
            return;
        }

        // Throttle: suppress notifications that fire less than 2 seconds apart
        // (e.g. rapid seek or fast skip through queue).
        let now = std::time::Instant::now();
        if let Some(last) = self.last_notification_time {
            if now.duration_since(last) < Duration::from_secs(2) {
                return;
            }
        }

        if notifications::notify_now_playing(&self.ui.settings, now_playing) {
            self.last_notified_track = Some(now_playing.path.clone());
            self.last_notification_time = Some(now);
        }
    }

    pub(crate) fn handle_playback_message(&mut self, message: &PlaybackMessage) {
        match message {
            PlaybackMessage::TogglePlayPause => {
                let Some(player) = &mut self.player else {
                    return;
                };
                match player.state() {
                    PlayerPlaybackState::Playing => player.pause(),
                    PlayerPlaybackState::Paused | PlayerPlaybackState::Stopped => player.play(),
                }
            }
            PlaybackMessage::NextTrack => {
                if !self.ui.play_from_queue {
                    return;
                }
                let next_track = self.playback_queue.next();
                self.load_from_queue(next_track);
            }
            PlaybackMessage::PreviousTrack => {
                if !self.ui.play_from_queue {
                    return;
                }
                let previous_track = self.playback_queue.previous();
                self.load_from_queue(previous_track);
            }
            PlaybackMessage::SeekToRatio(ratio) => {
                let Some(player) = &mut self.player else {
                    return;
                };
                let duration = self.ui.playback.duration;
                if duration.is_zero() {
                    return;
                }
                let clamped_ratio = (f32::from(*ratio) / 1000.0).clamp(0.0, 1.0);
                let target = Duration::from_secs_f32(duration.as_secs_f32() * clamped_ratio);
                if let Err(err) = player.seek(target) {
                    error!(error = %err, "Failed to seek");
                    self.ui.error_message = Some("Seek not supported for this format".to_string());
                    return;
                }
                self.ui.playback.position = target;
                self.ui.playback.animated_progress = progress_ratio(target, duration);
            }
            PlaybackMessage::ToggleShuffle | PlaybackMessage::CycleRepeat => {}
        }
    }

    pub(crate) fn apply_system_actions(&mut self, task: &mut Task<UiMessage>) {
        let Some(integration) = self.system_integration.as_mut() else {
            return;
        };
        let actions = integration.drain_actions();
        for action in actions {
            match action {
                SystemAction::Quit => {
                    let close_task = window::oldest().then(|id| {
                        if let Some(id) = id {
                            window::close(id)
                        } else {
                            Task::none()
                        }
                    });
                    Self::append_task(task, close_task);
                }
                SystemAction::TogglePlayPause => {
                    self.handle_playback_message(&PlaybackMessage::TogglePlayPause);
                }
                SystemAction::NextTrack => {
                    self.handle_playback_message(&PlaybackMessage::NextTrack);
                }
                SystemAction::PreviousTrack => {
                    self.handle_playback_message(&PlaybackMessage::PreviousTrack);
                }
            }
        }
    }

    pub(crate) fn sync_playback_state(&mut self) {
        let (is_playing, position) = match &self.player {
            Some(player) => {
                (matches!(player.state(), PlayerPlaybackState::Playing), player.position())
            }
            None => (false, Duration::ZERO),
        };
        self.ui.playback.is_playing = is_playing;
        self.ui.playback.position = position;
        self.ui.playback.duration = self
            .ui
            .selection
            .selected_track
            .as_ref()
            .map(|track| track.duration)
            .unwrap_or(Duration::ZERO);
    }

    pub(crate) fn maybe_auto_advance_track(&mut self) {
        if !self.ui.play_from_queue || !self.ui.playback.is_playing {
            return;
        }
        let Some(current_track) = self.ui.selection.selected_track.as_ref() else {
            return;
        };
        let duration = self.ui.playback.duration;
        if duration.is_zero() {
            // Zero-duration tracks cannot be tracked by position; skip forward
            // immediately to avoid getting stuck.
            if self
                .last_finished_track
                .as_ref()
                .is_some_and(|path| path == &current_track.path)
            {
                return;
            }
            self.last_finished_track = Some(current_track.path.clone());
            self.gapless_preloaded = false;
            let next_track = self.playback_queue.next();
            self.load_from_queue(next_track);
            return;
        }
        let position = self.ui.playback.position;

        // Gapless pre-load: when within 500ms of the end, append the next
        // track to the audio sink so it plays seamlessly when the current
        // source finishes.
        if self.ui.settings.gapless_playback && !self.gapless_preloaded && !duration.is_zero() {
            let remaining = duration.saturating_sub(position);
            if remaining <= Duration::from_millis(500) && remaining > Duration::ZERO {
                if let Some(next) = self.playback_queue.peek_next() {
                    let next_path = next.path.clone();
                    if let Some(player) = &mut self.player {
                        if player.append_gapless(&next_path).is_ok() {
                            self.gapless_preloaded = true;
                        }
                    }
                }
            }
        }

        let finished_grace = Duration::from_millis(150);
        let is_finished = position.saturating_add(finished_grace) >= duration;
        if !is_finished {
            if self
                .last_finished_track
                .as_ref()
                .is_some_and(|path| path == &current_track.path)
            {
                self.last_finished_track = None;
            }
            return;
        }
        if self
            .last_finished_track
            .as_ref()
            .is_some_and(|path| path == &current_track.path)
        {
            return;
        }
        self.last_finished_track = Some(current_track.path.clone());

        if self.gapless_preloaded {
            // The next track is already appended to the sink; just advance the
            // queue index and update UI state without reloading audio.
            self.gapless_preloaded = false;
            if let Some(next) = self.playback_queue.next() {
                if let Some(player) = &mut self.player {
                    player.current_track = Some(next.path.clone());
                    player.position = Duration::ZERO;
                    player.started_at = Some(std::time::Instant::now());
                }
                self.record_recently_played(&next);
                self.maybe_notify_now_playing(&next);
                self.ui.selection.selected_track = Some(self.ui_track_from_now_playing(&next));
            }
        } else {
            let next_track = self.playback_queue.next();
            self.load_from_queue(next_track);
        }
    }

    pub(crate) fn save_session_state(&self) {
        if !self.ui.settings.restore_last_session {
            return;
        }
        let active_tab = match self.ui.active_tab {
            ActiveTab::Artists => "artists",
            ActiveTab::Genres => "genres",
            ActiveTab::Albums => "albums",
            ActiveTab::Folders => "folders",
        };
        let session = config::SessionState {
            track_path: self.ui.selection.selected_track.as_ref().map(|t| t.path.clone()),
            position_secs: self.ui.playback.position.as_secs_f64(),
            active_tab: active_tab.to_string(),
            queue_index: self.playback_queue.index(),
        };
        if let Err(err) = config::save_session(&session) {
            warn!(error = %err, "Failed to save session state");
        }
    }

    pub(crate) fn library_root(&self) -> Option<PathBuf> {
        let root = self.ui.settings.library_folder.trim();
        if root.is_empty() {
            warn!("Library folder is not set; skipping library scan");
            return None;
        }
        Some(PathBuf::from(root))
    }

    pub(crate) fn begin_scan(&mut self, root: PathBuf, use_cache: bool) -> Task<UiMessage> {
        if self.ui.scan_status.is_some() {
            return Task::none();
        }
        let settings = self.ui.settings.clone();
        let root_artist_label = self.strings().root_artist_label.to_string();
        self.ui.scan_status = Some(ScanStatus::new(root.clone()));
        Task::perform(
            async move {
                let scan_result = if use_cache {
                    crate::library::scan_library(&root, &settings, &root_artist_label)
                } else {
                    crate::library::scan_library_full(&root, &settings, &root_artist_label)
                };
                scan_result.map_err(|err| err.to_string())
            },
            UiMessage::LibraryScanCompleted,
        )
    }

    pub(crate) fn begin_scan_from_settings(&mut self, use_cache: bool) -> Task<UiMessage> {
        let Some(root) = self.library_root() else {
            return Task::none();
        };
        self.begin_scan(root, use_cache)
    }

    pub(crate) fn reset_audio_engine(&mut self) {
        info!("Resetting audio engine");
        let options = AudioOptions::from_settings(&self.ui.settings);
        match self.player.as_mut() {
            Some(player) => {
                if let Err(err) = player.reset(options) {
                    error!(error = %err, "Failed to reset audio player");
                    self.player = Player::new_with_settings(&config::UserSettings::default()).ok();
                }
            }
            None => {
                self.player = match Player::new_with_settings(&self.ui.settings) {
                    Ok(player) => Some(player),
                    Err(err) => {
                        error!(error = %err, "Failed to reinitialize audio player");
                        None
                    }
                };
            }
        }
        self.ui.playback = crate::ui::state::PlaybackState::default();
        self.ui.selection = SelectionState::default();
        self.playback_queue = Self::playback_queue_from_playlist(&self.playlists);
    }

    pub(crate) fn open_logs_folder(&self) {
        let path = match config::ensure_logs_dir() {
            Ok(path) => path,
            Err(err) => {
                error!(error = %err, "Failed to ensure logs directory");
                return;
            }
        };
        if let Err(err) = Self::open_path_in_shell(&path) {
            error!(error = %err, path = %path.display(), "Failed to open logs folder");
        } else {
            info!(path = %path.display(), "Opened logs folder");
        }
    }

    fn open_path_in_shell(path: &Path) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = Command::new("cmd");
            command.args(["/C", "start", ""]);
            command.arg(path);
            command
        };

        #[cfg(target_os = "macos")]
        let mut command = {
            let mut command = Command::new("open");
            command.arg(path);
            command
        };

        #[cfg(all(unix, not(target_os = "macos")))]
        let mut command = {
            let mut command = Command::new("xdg-open");
            command.arg(path);
            command
        };

        let status = command.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("open command failed with status {status}"),
            ))
        }
    }

    pub(crate) fn apply_audio_settings(&mut self) {
        let Some(player) = &mut self.player else {
            return;
        };
        if let Err(err) = player.apply_settings(&self.ui.settings) {
            error!(error = %err, "Failed to apply audio settings");
            return;
        }
        if let Some(fallback) = player.take_last_fallback_notice() {
            self.handle_audio_fallback(fallback);
        }
    }

    fn handle_audio_fallback(&mut self, fallback: AudioFallback) {
        let language = self.ui.settings.interface_language.resolved();
        self.ui.audio_notice = Some(fallback.notice(language));
        Self::apply_audio_fallback_to_settings(&mut self.ui.settings, &fallback);
        if matches!(fallback.behavior, MissingDeviceBehavior::PausePlayback) {
            if let Some(player) = &mut self.player {
                player.pause();
            }
        }
        if let Err(err) = config::save_settings(&self.ui.settings) {
            error!(error = %err, "Failed to persist audio fallback settings");
        }
    }

    pub(crate) fn apply_audio_fallback_to_settings(
        settings: &mut config::UserSettings,
        fallback: &AudioFallback,
    ) {
        let _ = fallback;
        settings.output_device = AudioOutputDevice::System;
        settings.output_sample_rate_hz = None;
    }

    pub(crate) fn handle_declarative_action(
        &mut self,
        action: DeclarativeAction,
    ) -> Task<UiMessage> {
        match action {
            DeclarativeAction::ReindexLibrary => {
                info!("Library reindex requested");
                return self.begin_scan_from_settings(true);
            }
            DeclarativeAction::ClearCache => {
                info!("Cache clear requested");
                if let Some(root) = self.library_root() {
                    let cache_path = config::library_cache_dir(&self.ui.settings, &root);
                    match config::clear_library_cache(&self.ui.settings, &root) {
                        Ok(()) => {
                            info!(path = %cache_path.display(), "Library cache cleared");
                            return self.begin_scan(root, true);
                        }
                        Err(err) => {
                            error!(error = %err, path = %cache_path.display(), "Failed to clear cache");
                        }
                    }
                }
            }
            DeclarativeAction::ResetAudioEngine => {
                info!("Audio engine reset requested");
                self.reset_audio_engine();
            }
        }
        Task::none()
    }

    pub(crate) fn record_recently_played(&mut self, now_playing: &NowPlaying) {
        use crate::ui::state::RecentTrack;
        let entry = RecentTrack {
            title: now_playing.title.clone(),
            artist: now_playing.artist.clone(),
            album: now_playing.album.clone(),
            path: now_playing.path.clone(),
            played_at: std::time::SystemTime::now(),
        };
        self.ui.recently_played.retain(|t| t.path != entry.path);
        self.ui.recently_played.insert(0, entry);
        if self.ui.recently_played.len() > 50 {
            self.ui.recently_played.truncate(50);
        }
    }

    pub(crate) fn export_playlist_m3u(
        name: &str,
        items: &[(String, String, u32, PathBuf)],
    ) -> Result<String, String> {
        let config_root = config::config_root();
        let export_dir = config_root.join("exports");
        std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
        let safe_name: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let path = export_dir.join(format!("{safe_name}.m3u"));
        let mut content = String::from("#EXTM3U\n");
        for (artist, title, duration_secs, file_path) in items {
            content.push_str(&format!(
                "#EXTINF:{},{} - {}\n{}\n",
                duration_secs,
                artist,
                title,
                file_path.display()
            ));
        }
        std::fs::write(&path, &content).map_err(|e| e.to_string())?;
        Ok(format!("Exported to {}", path.display()))
    }

    pub(crate) fn handle_album_metadata_save(&mut self) {
        let Some(selected_album) = self.ui.selection.selected_album.as_ref() else {
            return;
        };
        let Some(root) = self.library_root() else {
            return;
        };
        let Some((artist, album)) = self.album_entry_by_id(selected_album.id) else {
            return;
        };
        let genre_value = self.ui.album_genre_draft.trim();
        let genre = if genre_value.is_empty() {
            None
        } else {
            Some(genre_value.to_string())
        };
        let year_value = self.ui.album_year_draft.trim();
        let year = if year_value.is_empty() {
            None
        } else {
            match year_value.parse::<u16>() {
                Ok(value) if value > 0 => Some(value),
                Ok(_) => None,
                Err(error) => {
                    warn!(error = %error, "Invalid album year input");
                    return;
                }
            }
        };
        if let Err(error) = crate::library::persist_album_metadata_override(
            &root,
            &artist.name,
            &album.title,
            genre.clone(),
            year,
        ) {
            warn!(error = %error, "Failed to persist album metadata override");
            return;
        }
        if let Some(album) = self.album_entry_by_id_mut(selected_album.id) {
            album.genre = genre.clone();
            album.year = year.unwrap_or(0);
        }
        if let Some(selected_album) = self.ui.selection.selected_album.as_mut() {
            selected_album.year = year.map(|value| value as u32);
        }
        self.refresh_album_metadata_drafts();
    }
}
