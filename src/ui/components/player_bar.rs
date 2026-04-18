#![allow(dead_code)]

use crate::ui::components::anchored_overlay::AnchoredOverlay;
use crate::ui::components::seek_area::seek_area;
use crate::ui::message::{PlaybackMessage, UiMessage};
use crate::ui::state::{PlaybackState, RepeatMode};
use crate::ui::style;
use iced::font::Weight;
use iced::mouse;
use iced::widget::{button, column, container, image, progress_bar, row, slider, text};
use iced::{Alignment, Element, Length, Padding};

#[derive(Debug, Clone)]
pub struct PlayerBar {
    cover_path: Option<std::path::PathBuf>,
    title: String,
    artist: String,
    playback: PlaybackState,
    volume: u8,
    queue_active: bool,
    queue_message: Option<UiMessage>,
    inline_volume_bar_open: bool,
    inline_volume_visibility: f32,
    inline_volume_toggle_message: Option<UiMessage>,
    playback_speed: u8,
    speed_popup_open: bool,
    mini_player: bool,
    error_message: Option<String>,
}

impl PlayerBar {
    pub fn new(title: impl Into<String>, artist: impl Into<String>) -> Self {
        Self {
            cover_path: None,
            title: title.into(),
            artist: artist.into(),
            playback: PlaybackState::default(),
            volume: 70,
            queue_active: false,
            queue_message: None,
            inline_volume_bar_open: false,
            inline_volume_visibility: 0.0,
            inline_volume_toggle_message: None,
            playback_speed: 10,
            speed_popup_open: false,
            mini_player: false,
            error_message: None,
        }
    }

    pub fn with_cover(mut self, cover_path: Option<std::path::PathBuf>) -> Self {
        self.cover_path = cover_path;
        self
    }

    pub fn with_playback(mut self, playback: PlaybackState) -> Self {
        self.playback = playback;
        self
    }

    pub fn with_volume(mut self, volume: u8) -> Self {
        self.volume = volume.min(100);
        self
    }

    pub fn with_queue(mut self, queue_active: bool) -> Self {
        self.queue_active = queue_active;
        self
    }

    pub fn with_queue_action(mut self, message: Option<UiMessage>) -> Self {
        self.queue_message = message;
        self
    }

    pub fn with_inline_volume_bar(mut self, inline_volume_bar_open: bool) -> Self {
        self.inline_volume_bar_open = inline_volume_bar_open;
        self
    }

    pub fn with_inline_volume_visibility(mut self, inline_volume_visibility: f32) -> Self {
        self.inline_volume_visibility = inline_volume_visibility.clamp(0.0, 1.0);
        self
    }

    pub fn with_inline_volume_toggle(mut self, message: Option<UiMessage>) -> Self {
        self.inline_volume_toggle_message = message;
        self
    }

    pub fn with_playback_speed(mut self, speed: u8) -> Self {
        self.playback_speed = speed.clamp(5, 20);
        self
    }

    pub fn with_speed_popup(mut self, open: bool) -> Self {
        self.speed_popup_open = open;
        self
    }

    pub fn with_mini_player(mut self, mini: bool) -> Self {
        self.mini_player = mini;
        self
    }

    pub fn with_error_message(mut self, error: Option<String>) -> Self {
        self.error_message = error;
        self
    }

    pub fn view(self, theme: style::ThemeTokens) -> Element<'static, UiMessage> {
        let PlayerBar {
            cover_path,
            title,
            artist,
            playback,
            volume,
            queue_active,
            queue_message,
            inline_volume_bar_open,
            inline_volume_visibility,
            inline_volume_toggle_message,
            playback_speed,
            speed_popup_open,
            mini_player,
            error_message,
        } = self;

        let cover_content: Element<UiMessage> = if let Some(ref path) = cover_path {
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            text("\u{f0386}")
                .size(theme.size(18))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_muted(theme))
                .into()
        };

        // Mini-player mode: compact layout
        if mini_player {
            let cover_mini = container(cover_content)
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .center_x(Length::Fixed(120.0))
                .center_y(Length::Fixed(120.0))
                .style(move |_| style::surface_style(theme, style::Surface::AlbumCover));

            let info = column![
                text(title.clone())
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text(artist.clone())
                    .size(theme.size_accessible(11))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme))
            ]
            .spacing(2)
            .align_x(Alignment::Start);

            let mini_controls = row![
                button(text("\u{f04ae}").font(style::font_propo(Weight::Medium)))
                    .style(move |_, status| {
                        style::button_style(theme, style::ButtonKind::Control, status)
                    })
                    .on_press(UiMessage::Playback(PlaybackMessage::PreviousTrack)),
                button(
                    text(play_pause_icon(playback.is_playing))
                        .font(style::font_propo(Weight::Medium)),
                )
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Control, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::TogglePlayPause)),
                button(text("\u{f04ad}").font(style::font_propo(Weight::Medium)))
                    .style(move |_, status| {
                        style::button_style(theme, style::ButtonKind::Control, status)
                    })
                    .on_press(UiMessage::Playback(PlaybackMessage::NextTrack)),
            ]
            .spacing(6)
            .align_y(Alignment::Center);

            let elapsed = format_duration(playback.position);
            let duration = format_duration(playback.duration);
            let time_label = text(format!("{elapsed} / {duration}"))
                .size(theme.size_accessible(10))
                .font(style::font_mono(Weight::Medium))
                .style(move |_| style::text_style_muted(theme));

            let expand_btn = button(
                text("\u{f0062}")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .on_press(UiMessage::ToggleMiniPlayer);

            let progress = seek_area(
                container(
                    progress_bar(0.0..=1.0, playback.animated_progress)
                        .style(move |_| style::progress_bar_style(theme)),
                )
                .width(Length::Fill)
                .height(Length::Fixed(3.0)),
            )
            .interaction(mouse::Interaction::Pointer)
            .on_press(|ratio| UiMessage::Playback(PlaybackMessage::SeekToRatio(ratio)));

            let content = column![
                row![cover_mini, info, mini_controls, time_label, expand_btn]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .width(Length::Fill),
                progress
            ]
            .spacing(4)
            .width(Length::Fill);

            return container(content)
                .padding([6, 12])
                .width(Length::Fill)
                .style(move |_| style::surface_style(theme, style::Surface::PlayerBar))
                .into();
        }

        // Full player bar
        let cover = container(cover_content)
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(80.0))
            .center_x(Length::Fixed(80.0))
            .center_y(Length::Fixed(80.0))
            .style(move |_| style::surface_style(theme, style::Surface::AlbumCover));

        let left = row![
            cover,
            column![
                text(title)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text(artist)
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme))
            ]
            .spacing(4)
            .align_x(Alignment::Start)
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .width(Length::FillPortion(3));

        let controls = row![
            button(text(shuffle_icon(playback.shuffle)).font(style::font_propo(Weight::Medium)),)
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Icon, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::ToggleShuffle)),
            button(text("\u{f04ae}").font(style::font_propo(Weight::Medium)))
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Control, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::PreviousTrack)),
            button(
                text(play_pause_icon(playback.is_playing)).font(style::font_propo(Weight::Medium)),
            )
            .style(move |_, status| {
                style::button_style(theme, style::ButtonKind::Control, status)
            })
            .on_press(UiMessage::Playback(PlaybackMessage::TogglePlayPause)),
            button(text("\u{f04ad}").font(style::font_propo(Weight::Medium)))
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Control, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::NextTrack)),
            button(text(repeat_icon(playback.repeat)).font(style::font_propo(Weight::Medium)))
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Icon, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::CycleRepeat)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .width(Length::FillPortion(4));

        let elapsed = format_duration(playback.position);
        let duration = format_duration(playback.duration);
        let progress = seek_area(
            container(
                progress_bar(0.0..=1.0, playback.animated_progress)
                    .style(move |_| style::progress_bar_style(theme)),
            )
            .width(Length::Fill),
        )
        .interaction(mouse::Interaction::Pointer)
        .on_press(|ratio| UiMessage::Playback(PlaybackMessage::SeekToRatio(ratio)));
        let progress_row = row![
            text(elapsed)
                .size(theme.size_accessible(12))
                .font(style::font_mono(Weight::Medium))
                .style(move |_| style::text_style_muted(theme)),
            progress,
            text(duration)
                .size(theme.size_accessible(12))
                .font(style::font_mono(Weight::Medium))
                .style(move |_| style::text_style_muted(theme))
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::Fill);

        // Speed control button
        let speed_display = format!("{}x", f32::from(playback_speed) / 10.0);
        let speed_label = text(speed_display)
            .size(theme.size_accessible(11))
            .font(style::font_mono(Weight::Medium))
            .style(move |_| {
                if speed_popup_open {
                    style::text_style_primary(theme)
                } else {
                    style::text_style_muted(theme)
                }
            });
        let speed_btn = button(speed_label)
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .on_press(UiMessage::ToggleSpeedPopup);

        // Build floating speed popup overlay
        let speed_widget: Element<UiMessage> = if speed_popup_open {
            let speed_options: Vec<u8> = vec![5, 7, 8, 10, 12, 15, 20];
            let mut speed_row = row![].spacing(4).align_y(Alignment::Center);
            for speed in speed_options {
                let label = format!("{}x", f32::from(speed) / 10.0);
                let is_current = speed == playback_speed;
                speed_row = speed_row.push(
                    button(
                        text(label)
                            .size(theme.size_accessible(10))
                            .font(style::font_mono(Weight::Medium))
                            .style(move |_| {
                                if is_current {
                                    style::text_style_primary(theme)
                                } else {
                                    style::text_style_muted(theme)
                                }
                            }),
                    )
                    .style(move |_, status| {
                        style::button_style(
                            theme,
                            style::ButtonKind::ListItem { selected: is_current, focused: false },
                            status,
                        )
                    })
                    .padding([4, 8])
                    .on_press(UiMessage::SetPlaybackSpeed(speed)),
                );
            }
            let speed_popup = container(speed_row)
                .padding([6, 10])
                .style(move |_| style::surface_style(theme, style::Surface::Panel));
            AnchoredOverlay::new(speed_btn, speed_popup)
                .above()
                .gap(8.0)
                .into()
        } else {
            speed_btn.into()
        };

        let queue_label = text(queue_icon(queue_active))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| {
                if queue_active {
                    style::text_style_primary(theme)
                } else {
                    style::text_style_muted(theme)
                }
            });
        let queue_control: Element<UiMessage> = if let Some(message) = queue_message {
            button(queue_label)
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(message)
                .into()
        } else {
            queue_label.into()
        };
        let volume_label = text(volume_icon(volume))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| style::text_style_muted(theme));
        let volume_control: Element<UiMessage> = if let Some(message) = inline_volume_toggle_message
        {
            button(volume_label)
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(message)
                .into()
        } else {
            volume_label.into()
        };
        let inline_volume_visibility = inline_volume_visibility.clamp(0.0, 1.0);
        let inline_volume_width = 70.0 * inline_volume_visibility;
        let inline_volume_control = container(
            slider(0.0..=100.0, f32::from(volume), |value| {
                UiMessage::SetDefaultVolume(value.round() as u8)
            })
            .width(Length::Fixed(inline_volume_width))
            .height(10.0),
        )
        .width(Length::Fixed(inline_volume_width))
        .height(Length::Fixed(10.0));
        let mut audio_icons = row![].spacing(8).align_y(Alignment::Center);
        if inline_volume_bar_open || inline_volume_visibility > 0.01 {
            audio_icons = audio_icons.push(inline_volume_control);
        }
        audio_icons = audio_icons
            .push(volume_control)
            .push(speed_widget)
            .push(queue_control);

        let mini_btn = button(
            text("\u{f0063}")
                .font(style::font_propo(Weight::Medium))
                .size(theme.size(14)),
        )
        .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
        .on_press(UiMessage::ToggleMiniPlayer);
        audio_icons = audio_icons.push(mini_btn);

        let mut right_content = column![progress_row].spacing(6).align_x(Alignment::End);
        right_content = right_content.push(audio_icons);

        // Error banner
        if let Some(error) = error_message {
            let error_row = row![
                text(error)
                    .size(theme.size_accessible(11))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                button(
                    text("\u{f0156}")
                        .font(style::font_propo(Weight::Medium))
                        .size(theme.size(12))
                )
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(UiMessage::DismissError)
            ]
            .spacing(8)
            .align_y(Alignment::Center);
            right_content = right_content.push(error_row);
        }

        let right = right_content.width(Length::FillPortion(5));

        let content = row![left, controls, right]
            .spacing(20)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        container(content)
            // Asymmetric padding: match vertical (10) on the left so the album
            // cover sits visually centered w.r.t. top/left/bottom; keep 16 on
            // the right so the controls/text cluster stays aerated.
            .padding(Padding::ZERO.top(10).right(16).bottom(10).left(10))
            .width(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::PlayerBar))
            .into()
    }

    pub fn render(&self) -> String {
        let artwork = if self.cover_path.is_some() {
            "🖼"
        } else {
            "♪"
        };
        let left = format!("[{}] {} — {}", artwork, self.title, self.artist);
        let controls = format!(
            "{} ⏮ {} ⏭ {}",
            shuffle_icon(self.playback.shuffle),
            play_pause_icon(self.playback.is_playing),
            repeat_icon(self.playback.repeat),
        );
        let elapsed = format_duration(self.playback.position);
        let duration = format_duration(self.playback.duration);
        let bar = build_progress_bar(self.playback.position, self.playback.duration, 24);
        let speed_label = format!("{}x", f32::from(self.playback_speed) / 10.0);
        let audio_icons = format!(
            "{} {} {}",
            volume_icon(self.volume),
            speed_label,
            queue_icon(self.queue_active)
        );

        vec![
            left,
            controls,
            format!("{} {} {}   {}", elapsed, bar, duration, audio_icons),
        ]
        .join("\n")
    }
}

fn shuffle_icon(active: bool) -> &'static str {
    if active { "\u{f049d}" } else { "\u{f049e}" }
}

fn play_pause_icon(is_playing: bool) -> &'static str {
    if is_playing { "\u{f03e4}" } else { "\u{f040a}" }
}

fn repeat_icon(mode: RepeatMode) -> &'static str {
    match mode {
        RepeatMode::Off => "\u{f0457}",
        RepeatMode::One => "\u{f0458}",
        RepeatMode::All => "\u{f0456}",
    }
}

fn volume_icon(volume: u8) -> &'static str {
    match volume {
        0 => "\u{f0581}",
        1..=33 => "\u{f057f}",
        34..=66 => "\u{f0580}",
        _ => "\u{f057e}",
    }
}

fn queue_icon(active: bool) -> &'static str {
    if active { "\u{f0cb8}" } else { "\u{f0cb9}" }
}

fn build_progress_bar(
    position: std::time::Duration,
    duration: std::time::Duration,
    width: usize,
) -> String {
    if width == 0 {
        return String::new();
    }
    let total = duration.as_secs_f32();
    let current = position.as_secs_f32().min(total);
    let ratio = if total > 0.0 { current / total } else { 0.0 };
    let filled = ((ratio * width as f32).round() as usize).min(width);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "━".repeat(filled), "─".repeat(empty))
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}
