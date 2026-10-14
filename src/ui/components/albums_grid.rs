#![allow(dead_code)]

use crate::ui::i18n::UiStrings;
use crate::ui::message::UiMessage;
use crate::ui::state::Album;
use crate::ui::style;
use colony_ui::sz;
use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct AlbumsGrid {
    sort_label: String,
    albums: Vec<Album>,
    selected_album_id: Option<usize>,
    total_count: usize,
    has_query: bool,
    load_more_message: Option<UiMessage>,
    columns: usize,
    scroll_offset: usize,
    viewport_rows: usize,
}

impl AlbumsGrid {
    pub fn new(albums: Vec<Album>, total_count: usize) -> Self {
        Self {
            sort_label: "A-Z".to_string(),
            albums,
            selected_album_id: None,
            total_count,
            has_query: false,
            load_more_message: None,
            columns: 3,
            scroll_offset: 0,
            viewport_rows: 3,
        }
    }

    pub const fn with_query(mut self, has_query: bool) -> Self {
        self.has_query = has_query;
        self
    }

    pub fn with_sort_label(mut self, sort_label: impl Into<String>) -> Self {
        self.sort_label = sort_label.into();
        self
    }

    pub const fn with_selection(mut self, selected_album_id: Option<usize>) -> Self {
        self.selected_album_id = selected_album_id;
        self
    }

    pub fn with_layout(
        mut self,
        columns: usize,
        scroll_offset: usize,
        viewport_rows: usize,
    ) -> Self {
        self.columns = columns.max(1);
        self.scroll_offset = scroll_offset;
        self.viewport_rows = viewport_rows.max(1);
        self
    }

    pub fn with_load_more(mut self, message: Option<UiMessage>) -> Self {
        self.load_more_message = message;
        self
    }

    pub fn view(
        self,
        focused: bool,
        theme: style::ThemeTokens,
        strings: &UiStrings,
    ) -> Element<'static, UiMessage> {
        let header = row![
            text(strings.albums_count_label(self.total_count))
                .size(sz(16))
                .font(colony_ui::ui_font_bold())
                .style(move |_| style::text_style_primary(theme)),
            text(format!("{} ", self.sort_label))
                .size(sz(12))
                .font(colony_ui::ui_font())
                .style(move |_| style::text_style_muted(theme))
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let grid_content: Element<'static, UiMessage> = if self.albums.is_empty() {
            let (title_str, hint_str) = if self.has_query {
                (strings.albums_no_results, strings.albums_no_results_hint)
            } else {
                (strings.albums_empty, strings.albums_empty_hint)
            };
            let empty = column![
                text(title_str)
                    .size(sz(14))
                    .font(colony_ui::ui_font_medium())
                    .style(move |_| style::text_style_primary(theme)),
                text(hint_str)
                    .size(sz(12))
                    .font(colony_ui::ui_font())
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
            let rows = self
                .albums
                .chunks(self.columns)
                .map(|chunk| {
                    let cells = chunk
                        .iter()
                        .map(|album| {
                            let is_selected = Some(album.id) == self.selected_album_id;
                            let cover_content: Element<UiMessage> =
                                album.cover_path.as_ref().map_or_else(
                                    || {
                                        text(style::icons::ALBUM)
                                            .size(sz(26))
                                            .font(colony_ui::fonts::glyph_font())
                                            .style(move |_| style::text_style_muted(theme))
                                            .into()
                                    },
                                    |cover_path| {
                                        image(image::Handle::from_path(cover_path))
                                            .width(Length::Fill)
                                            .height(Length::Fill)
                                            .into()
                                    },
                                );
                            let cover = container(cover_content)
                                .width(Length::Fixed(120.0))
                                .height(Length::Fixed(120.0))
                                .center_x(Length::Fixed(120.0))
                                .center_y(Length::Fixed(120.0))
                                .style(move |_| {
                                    style::surface_style(theme, style::Surface::AlbumCover)
                                });

                            let title = text(album.title.clone())
                                .size(sz(14))
                                .font(colony_ui::ui_font_medium())
                                .style(move |_| style::text_style_primary(theme));
                            let artist = text(album.artist.clone())
                                .size(sz(12))
                                .font(colony_ui::ui_font())
                                .style(move |_| style::text_style_muted(theme));
                            let card = column![cover, title, artist]
                                .spacing(6)
                                .align_x(Alignment::Center)
                                .width(Length::Fill);

                            button(card)
                                .style(move |_, status| {
                                    style::button_style(
                                        theme,
                                        style::ButtonKind::AlbumCard {
                                            selected: is_selected,
                                            focused: focused && is_selected,
                                        },
                                        status,
                                    )
                                })
                                .on_press(UiMessage::SelectAlbum(album.clone()))
                                .width(Length::FillPortion(1))
                                .into()
                        })
                        .collect::<Vec<Element<UiMessage>>>();

                    row(cells).spacing(16).align_y(Alignment::Start).width(Length::Fill).into()
                })
                .collect::<Vec<Element<UiMessage>>>();
            let mut grid = column(rows).spacing(20).width(Length::Fill).align_x(Alignment::Start);
            if let Some(message) = self.load_more_message.clone() {
                let remaining = self.total_count.saturating_sub(self.albums.len());
                if remaining > 0 {
                    let label = text(strings.load_more_label(remaining))
                        .size(sz(12))
                        .font(colony_ui::ui_font_medium())
                        .style(move |_| style::text_style_primary(theme));
                    let button = button(label)
                        .style(move |_, status| {
                            style::button_style(theme, style::ButtonKind::Control, status)
                        })
                        .padding([6, 10])
                        .on_press(message);
                    grid = grid.push(container(button).center_x(Length::Fill));
                }
            }
            grid.into()
        };
        let content = column![header, grid_content]
            .spacing(12)
            .width(Length::Fill)
            .align_x(Alignment::Start);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(move |_| style::surface_style(theme, style::Surface::Panel))
            .into()
    }
}
