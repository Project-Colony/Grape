#![allow(dead_code)]

use crate::ui::i18n::UiStrings;
use crate::ui::message::UiMessage;
use crate::ui::state::Album;
use crate::ui::style;
use iced::font::Weight;
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
            sort_label: "A–Z".to_string(),
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

    pub fn with_query(mut self, has_query: bool) -> Self {
        self.has_query = has_query;
        self
    }

    pub fn with_sort_label(mut self, sort_label: impl Into<String>) -> Self {
        self.sort_label = sort_label.into();
        self
    }

    pub fn with_selection(mut self, selected_album_id: Option<usize>) -> Self {
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
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            text(format!("{} ", self.sort_label))
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Light))
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
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text(hint_str)
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
            let rows = self
                .albums
                .chunks(self.columns)
                .map(|chunk| {
                    let cells = chunk
                        .iter()
                        .map(|album| {
                            let is_selected = Some(album.id) == self.selected_album_id;
                            let cover_content: Element<UiMessage> =
                                if let Some(cover_path) = &album.cover_path {
                                    image(image::Handle::from_path(cover_path))
                                        .width(Length::Fill)
                                        .height(Length::Fill)
                                        .into()
                                } else {
                                    text("♪")
                                        .size(theme.size(26))
                                        .font(style::font_propo(Weight::Medium))
                                        .style(move |_| style::text_style_muted(theme))
                                        .into()
                                };
                            let cover = container(cover_content)
                                .width(Length::Fixed(120.0))
                                .height(Length::Fixed(120.0))
                                .center_x(Length::Fixed(120.0))
                                .center_y(Length::Fixed(120.0))
                                .style(move |_| {
                                    style::surface_style(theme, style::Surface::AlbumCover)
                                });

                            let title = text(album.title.clone())
                                .size(theme.size(14))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_primary(theme));
                            let artist = text(album.artist.clone())
                                .size(theme.size_accessible(12))
                                .font(style::font_propo(Weight::Light))
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
                        .size(theme.size_accessible(12))
                        .font(style::font_propo(Weight::Medium))
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

    pub fn render(&self) -> String {
        let cover_width = 6usize;
        let cover_height = 3usize;
        let longest_label = self
            .albums
            .iter()
            .map(|album| album.title.len().max(album.artist.len()))
            .max()
            .unwrap_or(0);
        let cell_width = cover_width.max(longest_label).max(10);
        let cell_height = cover_height + 2;
        let total_rows = self.albums.len().div_ceil(self.columns);
        let scroll_offset = self.scroll_offset.min(total_rows.saturating_sub(1));

        let mut lines = Vec::new();
        lines.push(format!("Tri: {}", self.sort_label));

        let rows = self.albums.chunks(self.columns).collect::<Vec<_>>();
        let visible_rows = rows.iter().skip(scroll_offset).take(self.viewport_rows);

        for row in visible_rows {
            let cells = row
                .iter()
                .map(|album| self.build_cell(album, cover_width, cover_height))
                .collect::<Vec<_>>();

            for line_idx in 0..cell_height {
                let mut line = String::new();
                for (col, cell) in cells.iter().enumerate() {
                    let content = cell.get(line_idx).map(String::as_str).unwrap_or("");
                    line.push_str(&format!("{:<width$}", content, width = cell_width));
                    if col + 1 < self.columns {
                        line.push_str("  ");
                    }
                }
                lines.push(line.trim_end().to_string());
            }
        }

        lines.join("\n")
    }

    fn build_cell(&self, album: &Album, cover_width: usize, cover_height: usize) -> Vec<String> {
        let is_selected = Some(album.id) == self.selected_album_id;
        let cover_char = if album.cover_path.is_some() {
            if is_selected { '▓' } else { '▒' }
        } else if is_selected {
            '▓'
        } else {
            '█'
        };
        let cover_line = cover_char.to_string().repeat(cover_width);
        let mut cell = Vec::with_capacity(cover_height + 2);

        for _ in 0..cover_height {
            cell.push(cover_line.clone());
        }

        let title = if is_selected {
            format!("> {} <", album.title)
        } else {
            album.title.clone()
        };
        let artist = if is_selected {
            format!("> {} <", album.artist)
        } else {
            album.artist.clone()
        };

        cell.push(title);
        cell.push(artist);
        cell
    }
}
