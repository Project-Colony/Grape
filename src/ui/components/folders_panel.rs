#![allow(dead_code)]

use crate::ui::i18n::UiStrings;
use crate::ui::message::UiMessage;
use crate::ui::state::Folder;
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderLayout {
    Grid,
    List,
}

#[derive(Debug, Clone)]
pub struct FoldersPanel {
    sort_label: String,
    folders: Vec<Folder>,
    selected_folder_id: Option<usize>,
    total_count: usize,
    load_more_message: Option<UiMessage>,
    layout: FolderLayout,
    columns: usize,
    scroll_offset: usize,
    viewport_rows: usize,
}

impl FoldersPanel {
    pub fn new(folders: Vec<Folder>, total_count: usize) -> Self {
        Self {
            sort_label: "By name".to_string(),
            folders,
            selected_folder_id: None,
            total_count,
            load_more_message: None,
            layout: FolderLayout::Grid,
            columns: 3,
            scroll_offset: 0,
            viewport_rows: 3,
        }
    }

    pub fn with_sort_label(mut self, sort_label: impl Into<String>) -> Self {
        self.sort_label = sort_label.into();
        self
    }

    pub fn with_selection(mut self, selected_folder_id: Option<usize>) -> Self {
        self.selected_folder_id = selected_folder_id;
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
        let sort_label = self.sort_label.clone();
        let header = row![
            text(strings.folders_count_label(self.total_count))
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            text(sort_label)
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Light))
                .style(move |_| style::text_style_muted(theme))
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let content: Element<'static, UiMessage> = if self.folders.is_empty() {
            let empty = column![
                text(strings.folders_empty)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text(strings.folders_empty_hint)
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme)),
            ]
            .spacing(6)
            .align_x(Alignment::Center);
            column![
                header,
                container(empty)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
            ]
            .spacing(12)
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into()
        } else {
            match self.layout {
                FolderLayout::Grid => self.grid_view(header.into(), focused, theme, strings),
                FolderLayout::List => self.list_view(header.into(), focused, theme, strings),
            }
        };

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(move |_| style::surface_style(theme, style::Surface::Panel))
            .into()
    }

    fn grid_view(
        &self,
        header: Element<'static, UiMessage>,
        focused: bool,
        theme: style::ThemeTokens,
        strings: &UiStrings,
    ) -> Element<'static, UiMessage> {
        let rows = self
            .folders
            .chunks(self.columns)
            .map(|chunk| {
                let cells = chunk
                    .iter()
                    .map(|folder| {
                        let is_selected = Some(folder.id) == self.selected_folder_id;
                        let cover_content: Element<UiMessage> =
                            if let Some(cover_path) = &folder.cover_path {
                                image(image::Handle::from_path(cover_path))
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .into()
                            } else {
                                text("▣")
                                    .size(theme.size(26))
                                    .font(style::font_propo(Weight::Medium))
                                    .style(move |_| style::text_style_muted(theme))
                                    .into()
                            };
                        let icon = container(cover_content)
                        .width(Length::Fixed(120.0))
                        .height(Length::Fixed(120.0))
                        .center_x(Length::Fixed(120.0))
                        .center_y(Length::Fixed(120.0))
                        .style(move |_| style::surface_style(theme, style::Surface::AlbumCover));
                        let title = text(folder.name.clone())
                            .size(theme.size(14))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme));
                        let count = text(strings.tracks_count_label(folder.track_count))
                            .size(theme.size_accessible(12))
                            .font(style::font_propo(Weight::Light))
                            .style(move |_| style::text_style_muted(theme));
                        let card = column![icon, title, count]
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
                            .on_press(UiMessage::SelectFolder(folder.clone()))
                            .width(Length::FillPortion(1))
                            .into()
                    })
                    .collect::<Vec<Element<UiMessage>>>();

                row(cells).spacing(16).align_y(Alignment::Start).width(Length::Fill).into()
            })
            .collect::<Vec<Element<UiMessage>>>();
        let mut grid = column(rows).spacing(20).width(Length::Fill).align_x(Alignment::Start);
        if let Some(message) = self.load_more_message.clone() {
            let remaining = self.total_count.saturating_sub(self.folders.len());
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
        column![header, grid]
            .spacing(12)
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into()
    }

    fn list_view(
        &self,
        header: Element<'static, UiMessage>,
        focused: bool,
        theme: style::ThemeTokens,
        strings: &UiStrings,
    ) -> Element<'static, UiMessage> {
        let mut list_items = self
            .folders
            .iter()
            .map(|folder| {
                let is_selected = Some(folder.id) == self.selected_folder_id;
                let cover_content: Element<UiMessage> =
                    if let Some(cover_path) = &folder.cover_path {
                        image(image::Handle::from_path(cover_path))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    } else {
                        text("▣")
                            .size(theme.size(16))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme))
                            .into()
                    };
                let icon = container(cover_content)
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(28.0))
                .center_x(Length::Fixed(28.0))
                .center_y(Length::Fixed(28.0))
                .style(move |_| style::surface_style(theme, style::Surface::AlbumCover));
                let title = text(folder.name.clone())
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme));
                let count = text(strings.tracks_count_label(folder.track_count))
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme));
                let details =
                    column![title, count].spacing(2).width(Length::Fill).align_x(Alignment::Start);
                let row_content =
                    row![icon, details].spacing(10).align_y(Alignment::Center).width(Length::Fill);

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
                    .on_press(UiMessage::SelectFolder(folder.clone()))
                    .width(Length::Fill)
                    .into()
            })
            .collect::<Vec<Element<UiMessage>>>();
        if let Some(message) = self.load_more_message.clone() {
            let remaining = self.total_count.saturating_sub(self.folders.len());
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
        column![header, list]
            .spacing(12)
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into()
    }

    pub fn render(&self) -> String {
        let header = format!("{} Folders · {}", self.total_count, self.sort_label);
        let mut lines = vec![header];

        match self.layout {
            FolderLayout::Grid => {
                let total_rows = self.folders.len().div_ceil(self.columns);
                let scroll_offset = self.scroll_offset.min(total_rows.saturating_sub(1));
                let rows = self.folders.chunks(self.columns).collect::<Vec<_>>();
                let visible_rows = rows.iter().skip(scroll_offset).take(self.viewport_rows);

                for row in visible_rows {
                    let row_labels = row
                        .iter()
                        .map(|folder| {
                            let name = if Some(folder.id) == self.selected_folder_id {
                                format!("> {} <", folder.name)
                            } else {
                                folder.name.clone()
                            };
                            format!("{: <14} {:>3}", name, folder.track_count)
                        })
                        .collect::<Vec<_>>();
                    lines.push(row_labels.join("  "));
                }
            }
            FolderLayout::List => {
                let visible = self
                    .folders
                    .iter()
                    .skip(self.scroll_offset)
                    .take(self.viewport_rows)
                    .collect::<Vec<_>>();
                for folder in visible {
                    let name = if Some(folder.id) == self.selected_folder_id {
                        format!("> {} <", folder.name)
                    } else {
                        folder.name.clone()
                    };
                    lines.push(format!("{:<20} {:>3} tracks", name, folder.track_count));
                }
            }
        }

        lines.join("\n")
    }
}
