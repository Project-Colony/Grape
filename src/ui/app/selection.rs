use super::*;

impl GrapeApp {
    pub(crate) fn move_library_focus(&mut self, direction: isize) {
        let order = self.focus_order();
        if order.is_empty() {
            return;
        }
        let current_index =
            order.iter().position(|focus| *focus == self.ui.library_focus).unwrap_or(0);
        let next_index = if direction >= 0 {
            (current_index + direction as usize).min(order.len().saturating_sub(1))
        } else {
            current_index.saturating_sub(direction.abs() as usize)
        };
        self.ui.library_focus = order[next_index];
        self.ensure_focus_selection();
    }

    pub(crate) fn ensure_focus_selection(&mut self) {
        match self.ui.library_focus {
            LibraryFocus::Artists => {
                if self.ui.selection.selected_artist.is_none() {
                    if let Some(artist) = self.filtered_artists_from_catalog().into_iter().next() {
                        self.apply_artist_selection(artist);
                    }
                }
            }
            LibraryFocus::Genres => {
                if self.ui.selection.selected_genre.is_none() {
                    if let Some(genre) = self.filtered_genres_from_catalog().into_iter().next() {
                        self.apply_genre_selection(genre);
                    }
                }
            }
            LibraryFocus::Albums => {
                if self.ui.selection.selected_album.is_none() {
                    if let Some(album) = self.filtered_albums_from_catalog().into_iter().next() {
                        self.apply_album_selection(album);
                    }
                }
            }
            LibraryFocus::Folders => {
                if self.ui.selection.selected_folder.is_none() {
                    if let Some(folder) = self.filtered_folders_from_catalog().into_iter().next() {
                        self.apply_folder_selection(folder);
                    }
                }
            }
            LibraryFocus::Songs => {
                if self.ui.selection.selected_track.is_none() {
                    if let Some(track) = self.current_tracks().into_iter().next() {
                        self.apply_track_selection(track);
                    }
                }
            }
        }
    }

    pub(crate) fn handle_library_navigation(&mut self, navigation: LibraryNavigation) {
        match navigation {
            LibraryNavigation::Left | LibraryNavigation::PreviousPanel => {
                self.move_library_focus(-1);
            }
            LibraryNavigation::Right | LibraryNavigation::NextPanel => {
                self.move_library_focus(1);
            }
            LibraryNavigation::Up => {
                self.move_library_selection(-1);
            }
            LibraryNavigation::Down => {
                self.move_library_selection(1);
            }
        }
    }

    pub(crate) fn move_library_selection(&mut self, step: isize) {
        match self.ui.library_focus {
            LibraryFocus::Artists => {
                let artists = self.filtered_artists_from_catalog();
                let current = self.ui.selection.selected_artist.as_ref().map(|artist| artist.id);
                if let Some(artist) =
                    Self::move_selection(&artists, current, step, |artist| artist.id)
                {
                    self.apply_artist_selection(artist);
                }
            }
            LibraryFocus::Genres => {
                let genres = self.filtered_genres_from_catalog();
                let current = self.ui.selection.selected_genre.as_ref().map(|genre| genre.id);
                if let Some(genre) = Self::move_selection(&genres, current, step, |genre| genre.id)
                {
                    self.apply_genre_selection(genre);
                }
            }
            LibraryFocus::Albums => {
                let albums = self.filtered_albums_from_catalog();
                let current = self.ui.selection.selected_album.as_ref().map(|album| album.id);
                let adjusted_step = step * ALBUMS_GRID_COLUMNS as isize;
                if let Some(album) =
                    Self::move_selection(&albums, current, adjusted_step, |album| album.id)
                {
                    self.apply_album_selection(album);
                }
            }
            LibraryFocus::Folders => {
                let folders = self.filtered_folders_from_catalog();
                let current = self.ui.selection.selected_folder.as_ref().map(|folder| folder.id);
                if let Some(folder) =
                    Self::move_selection(&folders, current, step, |folder| folder.id)
                {
                    self.apply_folder_selection(folder);
                }
            }
            LibraryFocus::Songs => {
                let tracks = self.current_tracks();
                let current = self.ui.selection.selected_track.as_ref().map(|track| track.id);
                if let Some(track) = Self::move_selection(&tracks, current, step, |track| track.id)
                {
                    self.apply_track_selection(track);
                }
            }
        }
    }

    pub(crate) fn apply_artist_selection(&mut self, artist: UiArtist) {
        let artist_name = artist.name.clone();
        self.ui.selection.selected_artist = Some(artist);
        self.ui.selection.selected_album = None;
        self.ui.selection.selected_genre = None;
        self.ui.selection.selected_folder = None;
        self.ui.selection.selected_track = None;
        self.ui.library_focus = LibraryFocus::Artists;
        if let Some(album) = self
            .filtered_albums_from_catalog()
            .into_iter()
            .find(|album| album.artist == artist_name)
        {
            self.ui.selection.selected_album = Some(album.clone());
            self.ui.selection.selected_track =
                self.album_entry_by_id(album.id).and_then(|(artist, entry)| {
                    self.filtered_tracks_for_album(artist, entry).into_iter().next()
                });
        }
        self.refresh_album_metadata_drafts();
    }

    pub(crate) fn apply_album_selection(&mut self, album: UiAlbum) {
        let album_id = album.id;
        self.ui.selection.selected_album = Some(album);
        self.ui.selection.selected_folder = None;
        self.ui.selection.selected_track =
            self.album_entry_by_id(album_id).and_then(|(artist, entry)| {
                self.filtered_tracks_for_album(artist, entry).into_iter().next()
            });
        self.ui.library_focus = LibraryFocus::Songs;
        self.refresh_album_metadata_drafts();
    }

    pub(crate) fn apply_genre_selection(&mut self, genre: UiGenre) {
        self.ui.selection.selected_genre = Some(genre);
        self.ui.selection.selected_album = None;
        self.ui.selection.selected_folder = None;
        self.ui.selection.selected_track = None;
        self.ui.library_focus = LibraryFocus::Genres;
    }

    pub(crate) fn apply_folder_selection(&mut self, folder: UiFolder) {
        self.ui.selection.selected_folder = Some(folder);
        self.ui.selection.selected_genre = None;
        self.ui.selection.selected_album = None;
        self.ui.library_focus = LibraryFocus::Songs;
        self.ui.selection.selected_track = self
            .ui
            .selection
            .selected_folder
            .as_ref()
            .and_then(|folder| {
                self.folder_entry_by_id(folder.id)
                    .map(|(artist, entry)| self.filtered_tracks_for_album(artist, entry))
            })
            .and_then(|tracks| tracks.into_iter().next());
        self.refresh_album_metadata_drafts();
    }

    pub(crate) fn apply_track_selection(&mut self, track: UiTrack) {
        self.ui.selection.selected_track = Some(track);
        self.ui.library_focus = LibraryFocus::Songs;
    }

    pub(crate) fn activate_selection(&mut self) {
        if let Some(track) = self.ui.selection.selected_track.clone() {
            self.handle_track_selection(&track);
            return;
        }
        if let Some(album) = self.ui.selection.selected_album.clone() {
            if let Some((artist, entry)) = self.album_entry_by_id(album.id) {
                if let Some(track) =
                    self.filtered_tracks_for_album(artist, entry).into_iter().next()
                {
                    self.handle_track_selection(&track);
                    self.apply_track_selection(track);
                }
            }
        }
    }

    pub(crate) fn current_tracks(&self) -> Vec<UiTrack> {
        self.ui
            .selection
            .selected_album
            .as_ref()
            .and_then(|album| {
                self.album_entry_by_id(album.id).map(|(artist, entry)| (artist, entry))
            })
            .or_else(|| {
                self.ui.selection.selected_folder.as_ref().and_then(|folder| {
                    self.folder_entry_by_id(folder.id).map(|(artist, entry)| (artist, entry))
                })
            })
            .map(|(artist, album)| self.filtered_tracks_for_album(artist, album))
            .unwrap_or_default()
    }
}
