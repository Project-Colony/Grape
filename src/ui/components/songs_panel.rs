#![allow(dead_code)]

use crate::ui::i18n::UiStrings;
use crate::ui::message::UiMessage;
use crate::ui::state::{SelectionState, Track};
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Padding, Renderer};

#[derive(Debug, Clone)]
pub struct SongsPanel {
    album: String,
    artist: String,
    tracks: Vec<Track>,
    selected_track_id: Option<usize>,
    total_count: usize,
    load_more_message: Option<UiMessage>,
    scroll_offset: usize,
    viewport_size: usize,
    genre_draft: String,
    year_draft: String,
    show_metadata_editor: bool,
}

impl SongsPanel {
    pub fn new(
        album: impl Into<String>,
        artist: impl Into<String>,
        tracks: Vec<Track>,
        total_count: usize,
    ) -> Self {
        Self {
            album: album.into(),
            artist: artist.into(),
            tracks,
            selected_track_id: None,
            total_count,
            load_more_message: None,
            scroll_offset: 0,
            viewport_size: 8,
            genre_draft: String::new(),
            year_draft: String::new(),
            show_metadata_editor: false,
        }
    }

    pub fn with_selection(mut self, selected_track_id: Option<usize>) -> Self {
        self.selected_track_id = selected_track_id;
        self
    }

    pub fn with_scroll(mut self, scroll_offset: usize, viewport_size: usize) -> Self {
        self.scroll_offset = scroll_offset.min(self.tracks.len());
        self.viewport_size = viewport_size.max(1);
        self
    }

    pub fn with_load_more(mut self, message: Option<UiMessage>) -> Self {
        self.load_more_message = message;
        self
    }

    pub fn with_metadata_editor(
        mut self,
        genre_draft: impl Into<String>,
        year_draft: impl Into<String>,
        show_metadata_editor: bool,
    ) -> Self {
        self.genre_draft = genre_draft.into();
        self.year_draft = year_draft.into();
        self.show_metadata_editor = show_metadata_editor;
        self
    }

    pub fn view(
        &self,
        selection: &SelectionState,
        focused: bool,
        theme: style::ThemeTokens,
        strings: &UiStrings,
    ) -> Element<'static, UiMessage> {
        let selected_id = selection.selected_track.as_ref().map(|track| track.id);
        let header = row![
            text(strings.songs_count_label(self.total_count))
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            text(strings.songs_by_album)
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Light))
                .style(move |_| style::text_style_muted(theme))
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let metadata_editor: Element<'static, UiMessage> = if self.show_metadata_editor {
            let genre_input: iced::widget::TextInput<'_, UiMessage, iced::Theme, Renderer> =
                text_input(strings.songs_genre_placeholder, &self.genre_draft)
                    .style(move |_: &iced::Theme, status| style::text_input_style(theme, status))
                    .on_input(UiMessage::AlbumGenreChanged)
                    .padding([6, 10]);
            let year_input: iced::widget::TextInput<'_, UiMessage, iced::Theme, Renderer> =
                text_input(strings.songs_year_placeholder, &self.year_draft)
                    .style(move |_: &iced::Theme, status| style::text_input_style(theme, status))
                    .on_input(UiMessage::AlbumYearChanged)
                    .padding([6, 10]);
            let enrich_button = button(
                text(strings.songs_enrich)
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
            .padding([6, 10])
            .on_press(UiMessage::EnrichAlbumMetadata);
            let save_button = button(
                text(strings.songs_save)
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
            .padding([6, 10])
            .on_press(UiMessage::SaveAlbumMetadata);
            row![genre_input, year_input, enrich_button, save_button]
                .spacing(8)
                .align_y(Alignment::Center)
                .into()
        } else {
            row![].into()
        };
        let album_info = column![
            text(self.album.clone())
                .size(theme.size(18))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_primary(theme)),
            text(self.artist.clone())
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Light))
                .style(move |_| style::text_style_muted(theme)),
            metadata_editor
        ]
        .spacing(4)
        .align_x(Alignment::Start);
        let list_content: Element<'static, UiMessage> = if self.tracks.is_empty() {
            let empty = column![
                text(strings.songs_empty_title)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text(strings.songs_empty_subtitle)
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme)),
            ]
            .spacing(6)
            .align_x(Alignment::Center);
            container(empty)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            let mut list_items = self
                .tracks
                .iter()
                .enumerate()
                .map(|(index, track)| {
                    let is_selected = Some(track.id) == selected_id;
                    let number = format!("{:02}", track.track_number.unwrap_or((index + 1) as u32));
                    let number_label = text(number)
                        .size(theme.size_accessible(12))
                        .font(style::font_mono(Weight::Medium))
                        .style(move |_| style::text_style_muted(theme));
                    let title = text(track.title.clone())
                        .size(theme.size(14))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_primary(theme));
                    let artist = text(track.artist.clone())
                        .size(theme.size_accessible(12))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme));
                    let details = column![title, artist]
                        .spacing(2)
                        .width(Length::Fill)
                        .align_x(Alignment::Start);
                    let duration = text(format_duration(track.duration))
                        .size(theme.size_accessible(12))
                        .font(style::font_mono(Weight::Medium))
                        .style(move |_| style::text_style_muted(theme));
                    let row_content = row![number_label, details, duration]
                        .spacing(12)
                        .align_y(Alignment::Center)
                        .width(Length::Fill);

                    button(row_content)
                        .style(move |_, status| {
                            style::button_style(
                                theme,
                                style::ButtonKind::ListItem {
                                    selected: is_selected,
                                    focused: focused && is_selected,
                                },
                                status,
                            )
                        })
                        .on_press(UiMessage::SelectTrack(track.clone()))
                        .width(Length::Fill)
                        .into()
                })
                .collect::<Vec<Element<UiMessage>>>();
            if let Some(message) = self.load_more_message.clone() {
                let remaining = self.total_count.saturating_sub(self.tracks.len());
                if remaining > 0 {
                    let label = text(strings.load_more_label(remaining))
                        .size(theme.size_accessible(12))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_primary(theme));
                    list_items.push(
                        button(label)
                            .style(move |_, status| {
                                style::button_style(theme, style::ButtonKind::Control, status)
                            })
                            .padding([6, 10])
                            .on_press(message)
                            .into(),
                    );
                }
            }
            let list = column(list_items).spacing(8).width(Length::Fill).align_x(Alignment::Start);
            scrollable(container(list).padding(Padding::ZERO.right(6)))
                .height(Length::Fill)
                .into()
        };
        let content = column![header, album_info, list_content].spacing(12).height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(move |_| style::surface_style(theme, style::Surface::Panel))
            .into()
    }

    pub fn render(&self) -> String {
        let header = format!("{} Songs · {} — {}", self.total_count, self.album, self.artist);
        let visible = self
            .tracks
            .iter()
            .skip(self.scroll_offset)
            .take(self.viewport_size)
            .collect::<Vec<_>>();
        let max_title_len =
            self.tracks.iter().map(|track| track.title.len()).max().unwrap_or(0).max(12);
        let title_width = max_title_len + 4;
        let duration_width = self
            .tracks
            .iter()
            .map(|track| format_duration(track.duration).len())
            .max()
            .unwrap_or(0)
            .max(4);
        let mut lines = Vec::with_capacity(visible.len() * 2 + 1);
        lines.push(header);

        for (row, track) in visible.into_iter().enumerate() {
            let is_selected = Some(track.id) == self.selected_track_id;
            let number = track
                .track_number
                .map(|value| format!("{:>2}", value))
                .unwrap_or_else(|| format!("{:>2}", self.scroll_offset + row + 1));
            let duration = format_duration(track.duration);
            let title = if is_selected {
                format!("> {} <", track.title)
            } else {
                track.title.clone()
            };
            let artist = if is_selected {
                format!("> {} <", track.artist)
            } else {
                track.artist.clone()
            };
            lines.push(format!(
                "{}. {:<title_width$} {:>duration_width$}",
                number,
                title,
                duration,
                title_width = title_width,
                duration_width = duration_width
            ));
            lines.push(format!("    {}", artist));
        }

        lines.join("\n")
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}
