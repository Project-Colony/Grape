#![allow(dead_code)]

use crate::config::UserSettings;
use crate::ui::message::UiMessage;
use crate::ui::style;
use colony_ui::sz;
use iced::widget::{column, row, slider, text};
use iced::{Alignment, Element, Length};

pub fn eq_band_controls(
    theme: style::ThemeTokens,
    settings: &UserSettings,
) -> Element<'_, UiMessage> {
    let band_controls = settings.eq_model.bands.iter().enumerate().map(|(index, band)| {
        let gain = band.gain_db;
        row![
            column![
                text(format!("{} Hz", band.frequency_hz))
                    .size(sz(12))
                    .font(colony_ui::ui_font_medium())
                    .style(move |_| style::text_style_primary(theme)),
                text(format!("{gain:.1} dB"))
                    .size(sz(11))
                    .font(colony_ui::ui_font())
                    .style(move |_| style::text_style_muted(theme))
            ]
            .spacing(2)
            .width(Length::Fixed(sz(96))),
            slider(-12.0..=12.0, gain, move |value| { UiMessage::SetEqBandGain(index, value) })
                .width(Length::Fill)
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
    });

    column(band_controls).spacing(10).into()
}
