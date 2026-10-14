use crate::playlist::PlaylistManager;
use crate::ui::i18n::UiStrings;
use crate::ui::message::UiMessage;
use crate::ui::state::SelectionState;
use crate::ui::style;
use colony_ui::sz;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Padding};

pub struct PlaylistView;

impl PlaylistView {
    pub fn view<'a>(
        theme: style::ThemeTokens,
        playlists: &'a PlaylistManager,
        selection: &'a SelectionState,
        strings: &'a UiStrings,
    ) -> Element<'a, UiMessage> {
        let sidebar = Self::sidebar(theme, playlists, selection, strings);
        let detail = Self::detail(theme, playlists, selection, strings);

        let split = row![sidebar, detail].spacing(0).height(Length::Fill);

        container(split)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::AppBackground))
            .into()
    }

    fn sidebar<'a>(
        theme: style::ThemeTokens,
        playlists: &'a PlaylistManager,
        selection: &'a SelectionState,
        strings: &'a UiStrings,
    ) -> Element<'a, UiMessage> {
        let title = text(strings.menu_playlist)
            .size(sz(20))
            .font(colony_ui::ui_font_bold())
            .style(move |_| style::text_style_primary(theme));

        let close = button(
            text(style::icons::CLOSE)
                .size(sz(14))
                .font(colony_ui::fonts::glyph_font())
                .style(move |_| style::text_style_muted(theme)),
        )
        .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
        .padding([4, 8])
        .on_press(UiMessage::ClosePlaylist);

        let header = row![title, Space::new().width(Length::Fill), close]
            .align_y(Alignment::Center)
            .spacing(8);

        let section_label = text(strings.playlist_existing)
            .size(sz(11))
            .font(colony_ui::ui_font_bold())
            .style(move |_| style::text_style_muted(theme));

        let rows: Vec<Element<'a, UiMessage>> = if playlists.playlists.is_empty() {
            vec![
                text(strings.playlist_empty)
                    .size(sz(12))
                    .font(colony_ui::ui_font_medium())
                    .style(move |_| style::text_style_muted(theme))
                    .into(),
            ]
        } else {
            playlists
                .playlists
                .iter()
                .enumerate()
                .map(|(index, playlist)| {
                    let selected = index == playlists.active_index;
                    let name = text(playlist.name.clone())
                        .size(sz(14))
                        .font(colony_ui::ui_font_bold())
                        .style(move |_| style::selectable_text(theme, selected, style::text_style_primary));
                    let count = text(strings.tracks_count_label(playlist.items.len()))
                        .size(sz(11))
                        .font(colony_ui::ui_font_medium())
                        .style(move |_| style::selectable_text(theme, selected, style::text_style_muted));
                    let entry = column![name, count].spacing(2);

                    button(entry)
                        .style(move |_, status| {
                            style::button_style(
                                theme,
                                style::ButtonKind::ListItem { selected, focused: false },
                                status,
                            )
                        })
                        .padding([8, 12])
                        .width(Length::Fill)
                        .on_press(UiMessage::SelectPlaylist(index))
                        .into()
                })
                .collect()
        };

        let list = scrollable(column(rows).spacing(4)).height(Length::Fill);

        let action_button = |label: &'a str, message: UiMessage| {
            button(
                text(label)
                    .size(sz(12))
                    .font(colony_ui::ui_font_medium())
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
            .padding([6, 10])
            .on_press(message)
        };

        let footer = column![
            text_input(strings.playlist_name_placeholder, &selection.playlist_name_draft)
                .style(move |_, status| style::text_input_style(theme, status))
                .font(colony_ui::ui_font())
                .size(sz(16))
                .on_input(UiMessage::PlaylistNameChanged),
            row![
                action_button(strings.playlist_create, UiMessage::CreatePlaylist),
                action_button(strings.playlist_rename, UiMessage::RenamePlaylist),
                action_button(strings.playlist_delete, UiMessage::DeletePlaylist),
            ]
            .spacing(6),
        ]
        .spacing(8);

        container(
            column![header, section_label, list, footer]
                .spacing(12)
                .height(Length::Fill),
        )
            .padding(20)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::Panel))
            .into()
    }

    fn detail<'a>(
        theme: style::ThemeTokens,
        playlists: &'a PlaylistManager,
        selection: &'a SelectionState,
        strings: &'a UiStrings,
    ) -> Element<'a, UiMessage> {
        let active = playlists.active();
        let body: Element<'a, UiMessage> = match active {
            Some(playlist) => Self::detail_for(theme, playlist, selection, strings),
            None => Self::detail_empty(theme, strings),
        };

        container(body)
            .padding(24)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .into()
    }

    fn detail_empty<'a>(
        theme: style::ThemeTokens,
        strings: &'a UiStrings,
    ) -> Element<'a, UiMessage> {
        container(
            text(strings.playlist_empty_hint)
                .size(sz(14))
                .font(colony_ui::ui_font_medium())
                .style(move |_| style::text_style_muted(theme)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn detail_for<'a>(
        theme: style::ThemeTokens,
        playlist: &'a crate::playlist::Playlist,
        selection: &'a SelectionState,
        strings: &'a UiStrings,
    ) -> Element<'a, UiMessage> {
        let total_secs: u32 = playlist.items.iter().map(|i| i.duration_secs).sum();
        let stats = format!(
            "{} · {}",
            strings.tracks_count_label(playlist.items.len()),
            fmt_total_duration(total_secs)
        );

        let title = text(playlist.name.clone())
            .size(sz(24))
            .font(colony_ui::ui_font_bold())
            .style(move |_| style::text_style_primary(theme));
        let stats_label = text(stats)
            .size(sz(12))
            .font(colony_ui::ui_font_medium())
            .style(move |_| style::text_style_muted(theme));
        let header = column![title, stats_label].spacing(4);

        let mut add_track_button = button(
            text(strings.playlist_add_track)
                .size(sz(12))
                .font(colony_ui::ui_font_medium())
                .style(move |_| style::text_style_primary(theme)),
        )
        .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
        .padding([6, 10]);
        if selection.selected_track.is_some() {
            add_track_button = add_track_button.on_press(UiMessage::AddSelectedTrackToPlaylist);
        }

        let action_button = |label: &'a str, message: UiMessage| {
            button(
                text(label)
                    .size(sz(12))
                    .font(colony_ui::ui_font_medium())
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
            .padding([6, 10])
            .on_press(message)
        };

        let actions = row![
            add_track_button,
            action_button(strings.playlist_save_order, UiMessage::SavePlaylistOrder),
            action_button("M3U", UiMessage::ExportPlaylistM3u),
        ]
        .spacing(8);

        let list: Element<'a, UiMessage> = if playlist.items.is_empty() {
            container(
                text(strings.playlist_empty_hint)
                    .size(sz(14))
                    .font(colony_ui::ui_font_medium())
                    .style(move |_| style::text_style_muted(theme)),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            let drag_source = selection.playlist_drag_source;
            let mut rows: Vec<Element<'a, UiMessage>> = Vec::new();
            for (index, item) in playlist.items.iter().enumerate() {
                rows.push(Self::track_row(theme, index, item, drag_source));
            }
            // Right padding leaves a gutter for the scrollbar so duration + row
            // action icons sit comfortably inside the panel instead of flush
            // against the scrollbar.
            let list = scrollable(
                container(column(rows).spacing(8)).padding(Padding::ZERO.right(20)),
            )
            .height(Length::Fill);
            if let Some(source) = drag_source {
                column![
                    text(strings.playlist_move_prompt(source + 1))
                        .size(sz(12))
                        .font(colony_ui::ui_font_medium())
                        .style(move |_| style::text_style_muted(theme)),
                    list,
                ]
                .spacing(8)
                .into()
            } else {
                list.into()
            }
        };

        column![header, actions, list].spacing(16).into()
    }

    fn track_row<'a>(
        theme: style::ThemeTokens,
        index: usize,
        item: &'a crate::player::NowPlaying,
        drag_source: Option<usize>,
    ) -> Element<'a, UiMessage> {
        let index_label = text(format!("{:02}", index + 1))
            .size(sz(12))
            .font(colony_ui::ui_font_medium())
            .style(move |_| style::text_style_muted(theme));

        let title = text(item.title.clone())
            .size(sz(14))
            .font(colony_ui::ui_font_bold())
            .style(move |_| style::text_style_primary(theme));
        let subtitle = text(format!("{} · {}", item.artist, item.album))
            .size(sz(12))
            .font(colony_ui::ui_font_medium())
            .style(move |_| style::text_style_muted(theme));
        let track_info = column![title, subtitle].spacing(2);

        let duration = text(fmt_track_duration(item.duration_secs))
            .size(sz(12))
            .font(colony_ui::ui_font_medium())
            .style(move |_| style::text_style_muted(theme));

        let icon_button = |label: &'static str, message: UiMessage| {
            button(
                text(label)
                    .size(sz(12))
                    .font(colony_ui::fonts::glyph_font())
                    .style(move |_| style::text_style_muted(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .padding([2, 6])
            .on_press(message)
        };

        let drag_handle = icon_button(style::icons::DRAG, UiMessage::StartPlaylistItemDrag(index));
        let remove = icon_button(style::icons::CLOSE, UiMessage::DeletePlaylistItem(index));

        let actions = if let Some(source) = drag_source {
            if source != index {
                let drop = icon_button(
                    style::icons::DROP_HERE,
                    UiMessage::MovePlaylistItemDrag { from: source, to: index },
                );
                row![drag_handle, drop, remove].spacing(4)
            } else {
                row![drag_handle, remove].spacing(4)
            }
        } else {
            row![drag_handle, remove].spacing(4)
        };

        row![
            index_label,
            track_info,
            Space::new().width(Length::Fill),
            duration,
            actions,
        ]
        .align_y(Alignment::Center)
        .spacing(12)
        .into()
    }
}

fn fmt_track_duration(secs: u32) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{}:{:02}", m, s)
}

fn fmt_total_duration(secs: u32) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{}h {:02}min", h, m)
    } else {
        format!("{}min", m)
    }
}
