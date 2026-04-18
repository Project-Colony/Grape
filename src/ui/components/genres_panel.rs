#![allow(dead_code)]

use crate::ui::i18n::UiStrings;
use crate::ui::message::UiMessage;
use crate::ui::state::Genre;
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct GenresPanel {
    total_count: usize,
    genres: Vec<Genre>,
    selected_genre_id: Option<usize>,
    load_more_message: Option<UiMessage>,
    scroll_offset: usize,
    viewport_size: usize,
}

impl GenresPanel {
    pub fn new(genres: Vec<Genre>, total_count: usize) -> Self {
        Self {
            total_count,
            genres,
            selected_genre_id: None,
            load_more_message: None,
            scroll_offset: 0,
            viewport_size: 8,
        }
    }

    pub fn with_selection(mut self, selected_genre_id: Option<usize>) -> Self {
        self.selected_genre_id = selected_genre_id;
        self
    }

    pub fn with_scroll(mut self, scroll_offset: usize, viewport_size: usize) -> Self {
        self.scroll_offset = scroll_offset.min(self.genres.len());
        self.viewport_size = viewport_size.max(1);
        self
    }

    pub fn with_load_more(mut self, message: Option<UiMessage>) -> Self {
        self.load_more_message = message;
        self
    }

    pub fn view(
        &self,
        focused: bool,
        theme: style::ThemeTokens,
        strings: &UiStrings,
    ) -> Element<'static, UiMessage> {
        let header = row![
            text(strings.genres_count_label(self.total_count))
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            text(strings.sort_az)
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Light))
                .style(move |_| style::text_style_muted(theme))
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let list_content: Element<'static, UiMessage> = if self.genres.is_empty() {
            let empty = column![
                text(strings.genres_empty)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text(strings.genres_empty_hint)
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
                .genres
                .iter()
                .map(|genre| {
                    let is_selected = Some(genre.id) == self.selected_genre_id;
                    let badge = container(
                        text(genre.name.chars().next().unwrap_or('?').to_string())
                            .size(theme.size_accessible(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme)),
                    )
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .center_x(Length::Fixed(24.0))
                    .center_y(Length::Fixed(24.0))
                    .style(move |_| style::surface_style(theme, style::Surface::Avatar));
                    let name = text(genre.name.clone())
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_primary(theme))
                        .size(theme.size(14));
                    let count = text(strings.tracks_count_label(genre.track_count))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                        .size(theme.size_accessible(12));
                    let details = column![name, count]
                        .spacing(2)
                        .align_x(Alignment::Start)
                        .width(Length::Fill);
                    let row_content = row![badge, details]
                        .spacing(10)
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
                        .on_press(UiMessage::SelectGenre(genre.clone()))
                        .width(Length::Fill)
                        .into()
                })
                .collect::<Vec<Element<UiMessage>>>();
            if let Some(message) = self.load_more_message.clone() {
                let remaining = self.total_count.saturating_sub(self.genres.len());
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
            let list = column(list_items).spacing(6).width(Length::Fill).align_x(Alignment::Start);
            scrollable(list).height(Length::Fill).into()
        };
        let content = column![header, list_content].spacing(12).height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(move |_| style::surface_style(theme, style::Surface::Sidebar))
            .into()
    }

    pub fn render(&self) -> String {
        let header = format!("{} Genres", self.total_count);
        let visible = self
            .genres
            .iter()
            .skip(self.scroll_offset)
            .take(self.viewport_size)
            .collect::<Vec<_>>();
        let mut lines = Vec::with_capacity(visible.len() + 1);
        lines.push(header);

        for genre in visible {
            let is_selected = Some(genre.id) == self.selected_genre_id;
            let name = if is_selected {
                format!("> {} <", genre.name)
            } else {
                genre.name.clone()
            };
            lines.push(format!("{:<18} {:>3} tracks", name, genre.track_count));
        }

        lines.join("\n")
    }
}
