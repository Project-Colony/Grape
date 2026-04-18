use crate::playlist::PlaybackQueue;
use crate::ui::i18n::UiStrings;
use crate::ui::message::UiMessage;
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length, Padding};

pub struct QueueView;

impl QueueView {
    pub fn view<'a>(
        theme: style::ThemeTokens,
        playback_queue: &'a PlaybackQueue,
        play_from_queue: bool,
        strings: &'a UiStrings,
    ) -> Element<'a, UiMessage> {
        let now_playing = Self::now_playing_panel(theme, playback_queue, strings);
        let up_next = Self::up_next_panel(theme, playback_queue, play_from_queue, strings);

        let split = row![now_playing, up_next].spacing(0).height(Length::Fill);

        container(split)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::AppBackground))
            .into()
    }

    fn now_playing_panel<'a>(
        theme: style::ThemeTokens,
        playback_queue: &'a PlaybackQueue,
        strings: &'a UiStrings,
    ) -> Element<'a, UiMessage> {
        let page_title = text(strings.queue_title)
            .size(theme.size(24))
            .font(style::font_propo(Weight::Semibold))
            .style(move |_| style::text_style_primary(theme));

        let section_label = text(strings.queue_now_playing)
            .size(theme.size_accessible(11))
            .font(style::font_propo(Weight::Semibold))
            .style(move |_| style::text_style_muted(theme));

        let body: Element<'a, UiMessage> = if let Some(current) = playback_queue.current() {
            let title = text(current.title.clone())
                .size(theme.size(22))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme));
            let artist = text(current.artist.clone())
                .size(theme.size(14))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_primary(theme));
            let album = text(current.album.clone())
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_muted(theme));
            let duration = text(fmt_track_duration(current.duration_secs))
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_muted(theme));

            column![title, artist, album, Space::new().height(Length::Fixed(12.0)), duration]
                .spacing(4)
                .into()
        } else {
            text(strings.queue_up_next_idle)
                .size(theme.size(14))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_muted(theme))
                .into()
        };

        container(
            column![page_title, section_label, body]
                .spacing(12)
                .height(Length::Fill),
        )
        .padding(24)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(move |_| style::surface_style(theme, style::Surface::Panel))
        .into()
    }

    fn up_next_panel<'a>(
        theme: style::ThemeTokens,
        playback_queue: &'a PlaybackQueue,
        play_from_queue: bool,
        strings: &'a UiStrings,
    ) -> Element<'a, UiMessage> {
        let items = playback_queue.items();
        let total = items.len();
        let current_index = playback_queue.index();
        let upcoming_start = if total == 0 { 0 } else { current_index.saturating_add(1) };
        let upcoming: &[crate::player::NowPlaying] = if upcoming_start < total {
            &items[upcoming_start..]
        } else {
            &[]
        };
        let upcoming_total_secs: u32 = upcoming.iter().map(|i| i.duration_secs).sum();

        let title = text(strings.queue_up_next)
            .size(theme.size(22))
            .font(style::font_propo(Weight::Semibold))
            .style(move |_| style::text_style_primary(theme));

        let stats = if upcoming.is_empty() {
            String::new()
        } else {
            format!(
                "{} pistes · {}",
                upcoming.len(),
                fmt_total_duration(upcoming_total_secs)
            )
        };
        let stats_label = text(stats)
            .size(theme.size_accessible(12))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| style::text_style_muted(theme));
        let header = column![title, stats_label].spacing(4);

        let play_from_queue_label = if play_from_queue {
            strings.queue_play_on
        } else {
            strings.queue_play_off
        };
        let toggle_btn = button(
            text(play_from_queue_label)
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_primary(theme)),
        )
        .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
        .padding([6, 10])
        .on_press(UiMessage::TogglePlayFromQueue);

        let clear_btn = button(
            text(strings.queue_clear)
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_primary(theme)),
        )
        .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
        .padding([6, 10])
        .on_press(UiMessage::ClearQueue);

        let close_btn = button(
            text("✕")
                .size(theme.size(14))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_muted(theme)),
        )
        .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
        .padding([4, 8])
        .on_press(UiMessage::CloseQueue);

        let actions = row![toggle_btn, clear_btn, Space::new().width(Length::Fill), close_btn]
            .align_y(Alignment::Center)
            .spacing(8);

        let list: Element<'a, UiMessage> = if upcoming.is_empty() {
            container(
                text(strings.queue_empty)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_muted(theme)),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            let mut rows: Vec<Element<'a, UiMessage>> = Vec::new();
            for (rel_index, item) in upcoming.iter().enumerate() {
                let absolute_index = upcoming_start + rel_index;
                rows.push(Self::track_row(theme, rel_index + 1, absolute_index, item, total));
            }
            scrollable(
                container(column(rows).spacing(8)).padding(Padding::ZERO.right(20)),
            )
            .height(Length::Fill)
            .into()
        };

        container(
            column![row![header, Space::new().width(Length::Fill), actions]
                .align_y(Alignment::Start)
                .spacing(12),
            list]
            .spacing(16)
            .height(Length::Fill),
        )
        .padding(24)
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .into()
    }

    fn track_row<'a>(
        theme: style::ThemeTokens,
        display_index: usize,
        absolute_index: usize,
        item: &'a crate::player::NowPlaying,
        total_items: usize,
    ) -> Element<'a, UiMessage> {
        let index_label = text(format!("{:02}", display_index))
            .size(theme.size_accessible(12))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| style::text_style_muted(theme));

        let title = text(item.title.clone())
            .size(theme.size(14))
            .font(style::font_propo(Weight::Semibold))
            .style(move |_| style::text_style_primary(theme));
        let subtitle = text(format!("{} · {}", item.artist, item.album))
            .size(theme.size_accessible(12))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| style::text_style_muted(theme));
        let track_info = column![title, subtitle].spacing(2);

        let duration = text(fmt_track_duration(item.duration_secs))
            .size(theme.size_accessible(12))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| style::text_style_muted(theme));

        let icon_button_maybe = |label: &'static str, message: Option<UiMessage>| {
            let btn = button(
                text(label)
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_muted(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .padding([2, 6]);
            if let Some(msg) = message {
                btn.on_press(msg)
            } else {
                btn
            }
        };

        let can_move_up = absolute_index > 0;
        let can_move_down = absolute_index + 1 < total_items;

        let move_up = icon_button_maybe(
            "↑",
            can_move_up.then_some(UiMessage::MoveQueueItemUp(absolute_index)),
        );
        let move_down = icon_button_maybe(
            "↓",
            can_move_down.then_some(UiMessage::MoveQueueItemDown(absolute_index)),
        );
        let remove = icon_button_maybe(
            "✕",
            Some(UiMessage::RemoveQueueItem(absolute_index)),
        );

        let actions = row![move_up, move_down, remove].spacing(4);

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
