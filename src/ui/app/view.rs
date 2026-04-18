use super::*;
use iced::widget::column;

impl GrapeApp {
    pub(crate) fn view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();

        // Mini-player mode: only show the player bar
        if self.ui.mini_player {
            let layout = column![self.player_bar()].width(Length::Fill);
            return container(layout)
                .width(Length::Fill)
                .height(Length::Shrink)
                .style(move |_| style::surface_style(theme, style::Surface::AppBackground))
                .into();
        }

        if self.ui.queue_open {
            return self.queue_view();
        }
        if self.ui.playlist_open {
            return self.playlist_view();
        }

        let content = if self.ui.preferences_open {
            self.preferences_view()
        } else {
            match self.ui.active_tab {
                ActiveTab::Artists | ActiveTab::Albums => row![
                    container(self.artists_panel())
                        .width(Length::FillPortion(2))
                        .height(Length::Fill),
                    container(self.albums_panel())
                        .width(Length::FillPortion(5))
                        .height(Length::Fill),
                    container(self.songs_panel())
                        .width(Length::FillPortion(3))
                        .height(Length::Fill),
                ],
                ActiveTab::Genres => row![
                    container(self.genres_panel())
                        .width(Length::FillPortion(2))
                        .height(Length::Fill),
                    container(self.albums_panel())
                        .width(Length::FillPortion(5))
                        .height(Length::Fill),
                    container(self.songs_panel())
                        .width(Length::FillPortion(3))
                        .height(Length::Fill),
                ],
                ActiveTab::Folders => row![
                    container(self.folders_panel())
                        .width(Length::FillPortion(7))
                        .height(Length::Fill),
                    container(self.songs_panel())
                        .width(Length::FillPortion(3))
                        .height(Length::Fill),
                ],
            }
            .spacing(spacing::SECTION)
            .height(Length::Fill)
            .into()
        };

        let mut layout = column![self.top_bar()]
            .spacing(spacing::SECTION)
            .padding(spacing::SECTION)
            .height(Length::Fill);
        if let Some(banner) = self.scan_banner() {
            layout = layout.push(banner);
        }
        layout = layout.push(content).push(self.player_bar());

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::AppBackground))
            .into()
    }

    pub(crate) fn top_bar(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        let logo_handle = image::Handle::from_bytes(
            include_bytes!("../../../assets/logo.png").as_slice(),
        );
        let logo_mark = image(logo_handle).width(28).height(28);
        let logo = row![
            logo_mark,
            text("Grape")
                .size(theme.size(20))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme))
        ]
        .spacing(spacing::LG)
        .align_y(Alignment::Center);
        let logo_button = button(logo)
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .padding([spacing::XS, spacing::MD])
            .on_press(UiMessage::ToggleLogoMenu);
        let menu_button = |label, message| {
            button(
                text(label)
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem { selected: false, focused: false },
                    status,
                )
            })
            .padding([spacing::SM, spacing::LG])
            .on_press(message)
        };
        let menu_toggle = |label: &'static str, enabled: bool, filter: SearchFilter| {
            let label = if enabled {
                format!("{label} ✓")
            } else {
                label.to_string()
            };
            button(
                text(label)
                    .size(theme.size(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem { selected: enabled, focused: false },
                    status,
                )
            })
            .padding([spacing::SM, spacing::LG])
            .on_press(UiMessage::Search(SearchMessage::ToggleFilter(filter)))
        };
        let logo_menu = container(
            column![
                menu_button(strings.menu_library, UiMessage::ShowLibrary),
                menu_button(strings.menu_playlist, UiMessage::OpenPlaylist),
                menu_button(strings.menu_queue, UiMessage::OpenQueue),
                menu_button(strings.menu_preferences, UiMessage::OpenPreferences),
                text(strings.filters)
                    .size(theme.size(11))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme)),
                menu_toggle(
                    strings.filter_genre,
                    self.ui.search.filters.genre,
                    SearchFilter::Genre
                ),
                menu_toggle(strings.filter_year, self.ui.search.filters.year, SearchFilter::Year),
                menu_toggle(
                    strings.filter_duration,
                    self.ui.search.filters.duration,
                    SearchFilter::Duration,
                ),
                menu_toggle(
                    strings.filter_codec,
                    self.ui.search.filters.codec,
                    SearchFilter::Codec
                ),
            ]
            .spacing(spacing::MD),
        )
        .padding([spacing::LG, spacing::XXL])
        .style(move |_| style::surface_style(theme, style::Surface::Panel));
        let logo_widget: Element<'_, UiMessage> = if self.ui.menu_open {
            AnchoredOverlay::new(logo_button, logo_menu).into()
        } else {
            logo_button.into()
        };
        let tabs = row![
            button(
                text(self.tab_label(ActiveTab::Artists, strings.tab_artists))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::Tab {
                        selected: self.ui.active_tab == ActiveTab::Artists,
                    },
                    status,
                )
            })
            .on_press(UiMessage::TabSelected(ActiveTab::Artists)),
            button(
                text(self.tab_label(ActiveTab::Genres, strings.tab_genres))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::Tab {
                        selected: self.ui.active_tab == ActiveTab::Genres,
                    },
                    status,
                )
            })
            .on_press(UiMessage::TabSelected(ActiveTab::Genres)),
            button(
                text(self.tab_label(ActiveTab::Albums, strings.tab_albums))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::Tab {
                        selected: self.ui.active_tab == ActiveTab::Albums,
                    },
                    status,
                )
            })
            .on_press(UiMessage::TabSelected(ActiveTab::Albums)),
            button(
                text(self.tab_label(ActiveTab::Folders, strings.tab_folders))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::Tab {
                        selected: self.ui.active_tab == ActiveTab::Folders,
                    },
                    status,
                )
            })
            .on_press(UiMessage::TabSelected(ActiveTab::Folders)),
        ]
        .spacing(spacing::XXL)
        .align_y(Alignment::Center);
        let search_input = text_input(strings.search_placeholder, &self.ui.search.query)
            .style(move |_, status| style::text_input_style(theme, status))
            .on_input(|value| UiMessage::Search(SearchMessage::QueryChanged(value)));
        let search = row![
            search_input,
            button(text("≡").font(style::font_propo(Weight::Medium)).size(theme.size(14)))
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(UiMessage::ToggleLogoMenu),
            button(text("—").font(style::font_propo(Weight::Medium)).size(theme.size(14)))
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(UiMessage::WindowMinimize),
            button(text("").font(style::font_propo(Weight::Medium)).size(theme.size(14)))
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(UiMessage::WindowToggleMaximize),
            button(text("✕").font(style::font_propo(Weight::Medium)).size(theme.size(14)))
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(UiMessage::WindowClose)
        ]
        .spacing(spacing::LG)
        .align_y(Alignment::Center);

        let layout = row![
            container(logo_widget).width(Length::Shrink),
            container(tabs).width(Length::Fill).center_x(Length::Fill),
            container(search).width(Length::Shrink)
        ]
        .spacing(spacing::REGION)
        .align_y(Alignment::Center);

        container(layout)
            .padding([spacing::XL, spacing::SECTION])
            .width(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::TopBar))
            .into()
    }

    pub(crate) fn scan_banner(&self) -> Option<Element<'_, UiMessage>> {
        let status = self.ui.scan_status.as_ref()?;
        let theme = self.theme_tokens();
        let strings = self.strings();
        let stage_label = match status.stage {
            ScanStage::Indexing => strings.scan_indexing,
        };
        let progress = container(
            progress_bar(0.0..=1.0, status.progress)
                .style(move |_| style::progress_bar_style(theme)),
        )
        .height(Length::Fixed(6.0));
        let content = column![
            text(stage_label)
                .size(theme.size(14))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            text(strings.scan_folder_label(status.root.display()))
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Light))
                .style(move |_| style::text_style_muted(theme)),
            progress
        ]
        .spacing(spacing::MD)
        .width(Length::Fill);
        Some(
            container(content)
                .padding(spacing::XXL)
                .width(Length::Fill)
                .style(move |_| style::surface_style(theme, style::Surface::Panel))
                .into(),
        )
    }

    pub(crate) fn artists_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        let selected_id = self.ui.selection.selected_artist.as_ref().map(|artist| artist.id);
        let (artists, total) =
            Self::apply_limit(self.filtered_artists_from_catalog(), self.ui.list_limits.artists);
        let load_more = (total > artists.len()).then_some(UiMessage::LoadMoreArtists);
        let panel = ArtistsPanel::new(artists, total)
            .with_selection(selected_id)
            .with_load_more(load_more);
        panel.view(
            &self.ui.selection,
            self.ui.library_focus == LibraryFocus::Artists,
            theme,
            strings,
        )
    }

    pub(crate) fn albums_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        let sort_label = match self.ui.search.sort {
            SortOption::Alphabetical => strings.sort_az,
            SortOption::ByAlbum => strings.sort_by_album,
            SortOption::ByYear => strings.sort_by_year,
            SortOption::ByDuration => strings.sort_by_duration,
        };
        let selected_id = self.ui.selection.selected_album.as_ref().map(|album| album.id);
        let (albums, total) =
            Self::apply_limit(self.filtered_albums_from_catalog(), self.ui.list_limits.albums);
        let load_more = (total > albums.len()).then_some(UiMessage::LoadMoreAlbums);
        let grid = AlbumsGrid::new(albums, total)
            .with_sort_label(sort_label)
            .with_selection(selected_id)
            .with_load_more(load_more)
            .with_query(self.normalized_query().is_some())
            .view(self.ui.library_focus == LibraryFocus::Albums, theme, strings);

        container(grid).width(Length::Fill).height(Length::Fill).into()
    }

    pub(crate) fn songs_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        let selected_album = self
            .ui
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
            });
        let (album_title, artist_name, tracks, show_metadata_editor) = match selected_album {
            Some((artist, album)) => (
                album.title.clone(),
                artist.name.clone(),
                self.filtered_tracks_for_album(artist, album),
                true,
            ),
            None => (
                strings.select_album.to_string(),
                strings.pick_track.to_string(),
                Vec::new(),
                false,
            ),
        };
        let (tracks, total) = Self::apply_limit(tracks, self.ui.list_limits.tracks);
        let load_more = (total > tracks.len()).then_some(UiMessage::LoadMoreTracks);
        let selected_id = self.ui.selection.selected_track.as_ref().map(|track| track.id);
        let panel = SongsPanel::new(album_title, artist_name, tracks, total)
            .with_selection(selected_id)
            .with_load_more(load_more)
            .with_metadata_editor(
                self.ui.album_genre_draft.clone(),
                self.ui.album_year_draft.clone(),
                show_metadata_editor,
            );
        panel.view(&self.ui.selection, self.ui.library_focus == LibraryFocus::Songs, theme, strings)
    }

    pub(crate) fn genres_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        let selected_id = self.ui.selection.selected_genre.as_ref().map(|genre| genre.id);
        let (genres, total) =
            Self::apply_limit(self.filtered_genres_from_catalog(), self.ui.list_limits.genres);
        let load_more = (total > genres.len()).then_some(UiMessage::LoadMoreGenres);
        let panel = GenresPanel::new(genres, total)
            .with_selection(selected_id)
            .with_load_more(load_more);
        panel.view(self.ui.library_focus == LibraryFocus::Genres, theme, strings)
    }

    pub(crate) fn folders_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        let sort_label = match self.ui.search.sort {
            SortOption::Alphabetical => strings.sort_az,
            SortOption::ByAlbum => strings.sort_by_album,
            SortOption::ByYear => strings.sort_by_year,
            SortOption::ByDuration => strings.sort_by_duration,
        };
        let selected_id = self.ui.selection.selected_folder.as_ref().map(|folder| folder.id);
        let (folders, total) =
            Self::apply_limit(self.filtered_folders_from_catalog(), self.ui.list_limits.folders);
        let load_more = (total > folders.len()).then_some(UiMessage::LoadMoreFolders);
        FoldersPanel::new(folders, total)
            .with_sort_label(sort_label)
            .with_selection(selected_id)
            .with_load_more(load_more)
            .view(self.ui.library_focus == LibraryFocus::Folders, theme, strings)
    }

    pub(crate) fn player_bar(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let (title, artist, cover_path) = self
            .ui
            .selection
            .selected_track
            .as_ref()
            .map(|track| (track.title.clone(), track.artist.clone(), track.cover_path.clone()))
            .or_else(|| {
                self.ui.selection.selected_album.as_ref().map(|album| {
                    (album.title.clone(), album.artist.clone(), album.cover_path.clone())
                })
            })
            .unwrap_or_else(|| {
                let strings = self.strings();
                (strings.select_album.to_string(), strings.pick_track.to_string(), None)
            });

        let queue_message = if self.ui.queue_open {
            Some(UiMessage::CloseQueue)
        } else {
            Some(UiMessage::OpenQueue)
        };
        PlayerBar::new(title, artist)
            .with_cover(cover_path)
            .with_playback(self.ui.playback)
            .with_volume(self.ui.settings.default_volume)
            .with_queue(self.ui.queue_open)
            .with_queue_action(queue_message)
            .with_inline_volume_bar(self.ui.inline_volume_bar_open)
            .with_inline_volume_visibility(self.ui.inline_volume_visibility)
            .with_inline_volume_toggle(Some(UiMessage::ToggleInlineVolumeBar))
            .with_playback_speed(self.ui.settings.default_playback_speed)
            .with_speed_popup(self.ui.speed_popup_open)
            .with_mini_player(self.ui.mini_player)
            .with_error_message(self.ui.error_message.clone())
            .view(theme)
    }

    pub(crate) fn playlist_view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        PlaylistView::view(theme, &self.playlists, &self.ui.selection, strings)
    }

    pub(crate) fn queue_view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        QueueView::view(theme, &self.playback_queue, self.ui.play_from_queue, strings)
    }

    pub(crate) fn preferences_scroll_id(tab: PreferencesTab) -> Id {
        match tab {
            PreferencesTab::General => Id::new(PREFERENCES_GENERAL_SCROLL_ID),
            PreferencesTab::Appearance => Id::new(PREFERENCES_APPEARANCE_SCROLL_ID),
            PreferencesTab::Accessibility => Id::new(PREFERENCES_ACCESSIBILITY_SCROLL_ID),
            PreferencesTab::Audio => Id::new(PREFERENCES_AUDIO_SCROLL_ID),
        }
    }

    pub(crate) fn restore_preferences_scroll(&self, tab: PreferencesTab) -> Task<UiMessage> {
        let offset_y = self.ui.preferences_scroll.offset_for(tab);
        operation::scroll_to(
            Self::preferences_scroll_id(tab),
            iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: offset_y },
        )
    }
}
