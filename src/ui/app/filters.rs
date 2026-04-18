use super::*;

impl GrapeApp {
    pub(crate) fn year_matches(query: &str, year: Option<u32>) -> bool {
        year.map(|year| year.to_string().contains(query)).unwrap_or(false)
    }

    pub(crate) fn duration_matches(query: &str, duration: Duration) -> bool {
        let total_seconds = duration.as_secs();
        let total_minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        let mmss = format!("{minutes:02}:{seconds:02}");
        let hmmss = format!("{hours}:{minutes:02}:{seconds:02}");
        [total_seconds.to_string(), total_minutes.to_string(), mmss, hmmss]
            .iter()
            .any(|value| value.contains(query))
    }

    pub(crate) fn normalized_query(&self) -> Option<String> {
        let query = self.ui.search.query.trim();
        if query.is_empty() {
            None
        } else {
            Some(Self::normalize_text(query))
        }
    }

    pub(crate) fn albums_from_catalog(&self) -> Vec<UiAlbum> {
        let mut albums = Vec::new();
        let mut id = 0usize;

        for artist in &self.catalog.artists {
            let normalized_artist = Self::normalize_text(&artist.name);
            for album in &artist.albums {
                let normalized_title = Self::normalize_text(&album.title);

                let mut track_genres = Vec::new();
                let mut has_tracks_with_genre = false;
                let mut codec_set = std::collections::HashSet::new();

                for track in &album.tracks {
                    if let Some(genre) = track.genre.as_deref() {
                        for name in crate::library::split_genre_field(genre) {
                            has_tracks_with_genre = true;
                            let normalized = Self::normalize_text(name);
                            if !track_genres.contains(&normalized) {
                                track_genres.push(normalized);
                            }
                        }
                    }
                    if let Some(codec) = track.codec.as_deref() {
                        codec_set.insert(Self::normalize_text(codec));
                    }
                }

                albums.push(UiAlbum {
                    id,
                    title: album.title.clone(),
                    artist: artist.name.clone(),
                    year: if album.year == 0 { None } else { Some(album.year as u32) },
                    total_duration: Duration::from_secs(album.total_duration_secs as u64),
                    cover_path: album.cover.as_ref().map(|cover| cover.cached_path.clone()),
                    normalized_title,
                    normalized_artist: normalized_artist.clone(),
                    genre: album.genre.clone(),
                    track_genres,
                    has_tracks_with_genre,
                    codecs: codec_set.into_iter().collect(),
                });
                id += 1;
            }
        }

        albums
    }

    pub(crate) fn filtered_albums_from_catalog(&self) -> Vec<UiAlbum> {
        let mut albums = self.albums_from_catalog();
        if let Some(query) = self.normalized_query() {
            let filters = self.ui.search.filters;
            albums.retain(|album| {
                let mut matches = album.normalized_title.contains(&*query)
                    || album.normalized_artist.contains(&*query);
                if filters.genre {
                    matches |= album
                        .genre
                        .as_deref()
                        .map(|genre| Self::normalized_contains(&query, genre))
                        .unwrap_or(false);
                }
                if filters.year {
                    matches |= Self::year_matches(&query, album.year);
                }
                if filters.duration {
                    matches |= Self::duration_matches(&query, album.total_duration);
                }
                if filters.codec {
                    matches |= album.codecs.iter().any(|c| c.contains(&*query));
                }
                matches
            });
        }
        if self.ui.active_tab == ActiveTab::Genres {
            if let Some(selected_genre) = self.ui.selection.selected_genre.as_ref() {
                let normalized_genre = Self::normalize_text(selected_genre.name.trim());
                let normalized_unknown_genre =
                    Self::normalize_text(self.strings().genres_unknown_label);
                albums.retain(|album| {
                    album.track_genres.iter().any(|g| *g == normalized_genre)
                        || (!album.has_tracks_with_genre
                            && normalized_genre == normalized_unknown_genre)
                });
            }
        }
        match self.ui.search.sort {
            SortOption::Alphabetical => {
                albums.sort_by(|a, b| {
                    a.normalized_title
                        .cmp(&b.normalized_title)
                        .then_with(|| a.normalized_artist.cmp(&b.normalized_artist))
                });
            }
            SortOption::ByAlbum => {
                albums.sort_by(|a, b| {
                    a.normalized_artist
                        .cmp(&b.normalized_artist)
                        .then_with(|| a.normalized_title.cmp(&b.normalized_title))
                        .then_with(|| a.year.cmp(&b.year))
                });
            }
            SortOption::ByYear => {
                albums.sort_by(|a, b| {
                    let year_a = a.year.unwrap_or(u32::MAX);
                    let year_b = b.year.unwrap_or(u32::MAX);
                    year_a
                        .cmp(&year_b)
                        .then_with(|| a.normalized_title.cmp(&b.normalized_title))
                        .then_with(|| a.normalized_artist.cmp(&b.normalized_artist))
                });
            }
            SortOption::ByDuration => {
                albums.sort_by(|a, b| {
                    a.total_duration
                        .cmp(&b.total_duration)
                        .then_with(|| a.normalized_title.cmp(&b.normalized_title))
                        .then_with(|| a.normalized_artist.cmp(&b.normalized_artist))
                });
            }
        }
        albums
    }

    pub(crate) fn artists_from_catalog(&self) -> Vec<UiArtist> {
        self.catalog
            .artists
            .iter()
            .enumerate()
            .map(|(id, artist)| {
                let normalized_name = Self::normalize_text(&artist.name);
                UiArtist {
                    id,
                    name: artist.name.clone(),
                    normalized_name,
                }
            })
            .collect()
    }

    pub(crate) fn filtered_artists_from_catalog(&self) -> Vec<UiArtist> {
        let mut artists = self.artists_from_catalog();
        if let Some(query) = self.normalized_query() {
            artists.retain(|artist| artist.normalized_name.contains(&*query));
        }
        artists.sort_by(|a, b| a.normalized_name.cmp(&b.normalized_name));
        artists
    }

    pub(crate) fn genres_from_catalog(&self) -> Vec<UiGenre> {
        self.catalog
            .genres(self.strings().genres_unknown_label)
            .into_iter()
            .enumerate()
            .map(|(id, genre)| {
                let normalized_name = Self::normalize_text(&genre.name);
                UiGenre {
                    id,
                    name: genre.name,
                    track_count: genre.track_count,
                    normalized_name,
                }
            })
            .collect()
    }

    pub(crate) fn filtered_genres_from_catalog(&self) -> Vec<UiGenre> {
        let mut genres = self.genres_from_catalog();
        if let Some(query) = self.normalized_query() {
            genres.retain(|genre| genre.normalized_name.contains(&*query));
        }
        genres.sort_by(|a, b| a.normalized_name.cmp(&b.normalized_name));
        genres
    }

    pub(crate) fn album_entry_by_id(
        &self,
        album_id: usize,
    ) -> Option<(&crate::library::Artist, &crate::library::Album)> {
        let mut id = 0usize;
        for artist in &self.catalog.artists {
            for album in &artist.albums {
                if id == album_id {
                    return Some((artist, album));
                }
                id += 1;
            }
        }
        None
    }

    pub(crate) fn album_entry_by_id_mut(
        &mut self,
        album_id: usize,
    ) -> Option<&mut crate::library::Album> {
        let mut id = 0usize;
        for artist in &mut self.catalog.artists {
            for album in &mut artist.albums {
                if id == album_id {
                    return Some(album);
                }
                id += 1;
            }
        }
        None
    }

    pub(crate) fn refresh_album_metadata_drafts(&mut self) {
        let draft = self
            .ui
            .selection
            .selected_album
            .as_ref()
            .and_then(|selected| self.album_entry_by_id(selected.id))
            .map(|(_, album)| {
                (
                    album.genre.clone().unwrap_or_default(),
                    if album.year > 0 {
                        album.year.to_string()
                    } else {
                        String::new()
                    },
                )
            })
            .unwrap_or_else(|| (String::new(), String::new()));
        self.ui.album_genre_draft = draft.0;
        self.ui.album_year_draft = draft.1;
    }

    pub(crate) fn request_album_metadata(
        &self,
        album: &UiAlbum,
        enrichment_confirmed: bool,
        force_refresh: bool,
    ) -> Task<UiMessage> {
        let Some(root) = self.library_root() else {
            return Task::none();
        };
        let settings = self.ui.settings.clone();
        let artist = album.artist.clone();
        let title = album.title.clone();
        let album_id = album.id;
        let artist_for_request = artist.clone();
        Task::perform(
            async move {
                let mut last_err = String::new();
                for attempt in 0..3u32 {
                    if attempt > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            500 * 2u64.pow(attempt - 1),
                        ))
                        .await;
                    }
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(15),
                        crate::library::fetch_album_online_metadata(
                            &root,
                            &settings,
                            &artist_for_request,
                            &title,
                            force_refresh,
                        ),
                    )
                    .await
                    {
                        Ok(Ok(result)) => return Ok(result),
                        Ok(Err(err)) => {
                            last_err = err.to_string();
                        }
                        Err(_) => {
                            last_err = "metadata fetch timed out".to_string();
                        }
                    }
                }
                Err(last_err)
            },
            move |result| UiMessage::AlbumMetadataFetched {
                album_id,
                artist,
                result,
                enrichment_confirmed,
            },
        )
    }

    pub(crate) fn folder_entry_by_id(
        &self,
        folder_id: usize,
    ) -> Option<(&crate::library::Artist, &crate::library::Album)> {
        let mut id = 0usize;
        for artist in &self.catalog.artists {
            for album in &artist.albums {
                if album.tracks.is_empty() {
                    continue;
                }
                if id == folder_id {
                    return Some((artist, album));
                }
                id += 1;
            }
        }
        None
    }

    pub(crate) fn tracks_for_album(
        &self,
        artist: &crate::library::Artist,
        album: &crate::library::Album,
    ) -> Vec<UiTrack> {
        album
            .tracks
            .iter()
            .enumerate()
            .map(|(id, track)| {
                let artist_name = track.artist.as_deref().unwrap_or(&artist.name);
                let normalized_title = Self::normalize_text(&track.title);
                let normalized_artist = Self::normalize_text(artist_name);
                let normalized_album = Self::normalize_text(&album.title);
                UiTrack {
                    id,
                    title: track.title.clone(),
                    album: album.title.clone(),
                    artist: artist_name.to_string(),
                    track_number: Some(track.number as u32),
                    duration: std::time::Duration::from_secs(track.duration_secs as u64),
                    path: track.path.clone(),
                    cover_path: album.cover.as_ref().map(|cover| cover.cached_path.clone()),
                    normalized_title,
                    normalized_artist,
                    normalized_album,
                }
            })
            .collect()
    }

    pub(crate) fn filtered_tracks_for_album(
        &self,
        artist: &crate::library::Artist,
        album: &crate::library::Album,
    ) -> Vec<UiTrack> {
        let mut tracks = self.tracks_for_album(artist, album);
        let album_year = if album.year == 0 { None } else { Some(album.year as u32) };
        if let Some(query) = self.normalized_query() {
            let filters = self.ui.search.filters;
            tracks.retain(|track| {
                let mut matches = track.normalized_title.contains(&*query)
                    || track.normalized_artist.contains(&*query)
                    || track.normalized_album.contains(&*query);
                if filters.genre {
                    matches |= album
                        .genre
                        .as_deref()
                        .map(|genre| Self::normalized_contains(&query, genre))
                        .unwrap_or(false);
                }
                if filters.year {
                    matches |= Self::year_matches(&query, album_year);
                }
                if filters.duration {
                    matches |= Self::duration_matches(&query, track.duration);
                }
                if filters.codec {
                    matches |= album
                        .tracks
                        .get(track.id)
                        .map(|entry| Self::codec_matches(&query, entry.codec.as_deref()))
                        .unwrap_or(false);
                }
                matches
            });
        }
        match self.ui.search.sort {
            SortOption::Alphabetical => {
                tracks.sort_by(|a, b| {
                    a.normalized_title
                        .cmp(&b.normalized_title)
                        .then_with(|| a.track_number.cmp(&b.track_number))
                });
            }
            SortOption::ByAlbum => {
                tracks.sort_by(|a, b| {
                    a.track_number
                        .unwrap_or(u32::MAX)
                        .cmp(&b.track_number.unwrap_or(u32::MAX))
                        .then_with(|| a.normalized_title.cmp(&b.normalized_title))
                });
            }
            SortOption::ByYear => {
                // All tracks within a single album share the same year, so the
                // primary sort is by track number (the year comparison was a
                // no-op comparing the same variable to itself).
                tracks.sort_by(|a, b| {
                    a.track_number
                        .unwrap_or(u32::MAX)
                        .cmp(&b.track_number.unwrap_or(u32::MAX))
                        .then_with(|| a.normalized_title.cmp(&b.normalized_title))
                });
            }
            SortOption::ByDuration => {
                tracks.sort_by(|a, b| {
                    a.duration
                        .cmp(&b.duration)
                        .then_with(|| a.normalized_title.cmp(&b.normalized_title))
                        .then_with(|| a.track_number.cmp(&b.track_number))
                });
            }
        }
        tracks
    }

    pub(crate) fn folders_from_catalog(&self) -> Vec<UiFolder> {
        let mut folders = Vec::new();
        let mut id = 0usize;
        for artist in &self.catalog.artists {
            for album in &artist.albums {
                if album.tracks.is_empty() {
                    continue;
                }
                let album_folder = if album.year > 0 {
                    format!("{:04} - {}", album.year, album.title)
                } else {
                    album.title.clone()
                };
                let name = format!("{}/{}", artist.name, album_folder);
                let normalized_name = Self::normalize_text(&name);
                let mut codec_set = std::collections::HashSet::new();
                for track in &album.tracks {
                    if let Some(codec) = track.codec.as_deref() {
                        codec_set.insert(Self::normalize_text(codec));
                    }
                }
                folders.push(UiFolder {
                    id,
                    name,
                    track_count: album.tracks.len(),
                    normalized_name,
                    genre: album.genre.clone(),
                    year: if album.year == 0 { None } else { Some(album.year as u32) },
                    total_duration: Duration::from_secs(album.total_duration_secs as u64),
                    cover_path: album.cover.as_ref().map(|cover| cover.cached_path.clone()),
                    codecs: codec_set.into_iter().collect(),
                });
                id += 1;
            }
        }
        folders
    }

    pub(crate) fn filtered_folders_from_catalog(&self) -> Vec<UiFolder> {
        let mut folders = self.folders_from_catalog();
        if let Some(query) = self.normalized_query() {
            let filters = self.ui.search.filters;
            folders.retain(|folder| {
                let mut matches = folder.normalized_name.contains(&*query);
                if filters.genre {
                    matches |= folder
                        .genre
                        .as_deref()
                        .map(|genre| Self::normalized_contains(&query, genre))
                        .unwrap_or(false);
                }
                if filters.year {
                    matches |= Self::year_matches(&query, folder.year);
                }
                if filters.duration {
                    matches |= Self::duration_matches(&query, folder.total_duration);
                }
                if filters.codec {
                    matches |= folder.codecs.iter().any(|c| c.contains(&*query));
                }
                matches
            });
        }
        folders.sort_by(|a, b| a.normalized_name.cmp(&b.normalized_name));
        folders
    }
}
