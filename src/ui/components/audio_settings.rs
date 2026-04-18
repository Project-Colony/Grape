#![allow(dead_code)]

use crate::config::UserSettings;
use crate::ui::message::UiMessage;
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{column, row, slider, text};
use iced::{Alignment, Element, Length};

pub fn eq_band_controls<'a>(
    theme: style::ThemeTokens,
    settings: &'a UserSettings,
) -> Element<'a, UiMessage> {
    let band_controls = settings
        .eq_model
        .bands
        .iter()
        .enumerate()
        .map(|(index, band)| {
            let gain = band.gain_db;
            row![
                column![
                    text(format!("{} Hz", band.frequency_hz))
                        .size(theme.size_accessible(12))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_primary(theme)),
                    text(format!("{:.1} dB", gain))
                        .size(theme.size_accessible(11))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                ]
                .spacing(2)
                .width(Length::Fixed(96.0)),
                slider(-12.0..=12.0, gain, move |value| {
                    UiMessage::SetEqBandGain(index, value)
                })
                .width(Length::Fill)
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
        });

    column(band_controls).spacing(10).into()
}
