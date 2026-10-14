use super::super::*;
use super::helpers::*;
use colony_ui::widgets as kit;
use iced::widget::Column;

impl GrapeApp {
    pub(super) fn audio_preferences(&self) -> Column<'_, UiMessage> {
        let typo = self.typography();
        let theme = self.theme_tokens();
        let language = self.language();
        let strings = self.strings();
        let settings = &self.ui.settings;
        let open = &self.ui.preferences_sections;
        let reset_engine = DeclarativeAction::ResetAudioEngine;
        let muted_value = |value: String| {
            text(value)
                .size(sz(12))
                .font(colony_ui::ui_font())
                .style(move |_| style::text_style_muted(theme))
        };

        // The notice a missing device leaves, until it is read.
        let notice: Option<Element<'_, UiMessage>> = self.ui.audio_notice.as_deref().map(|label| {
            row![
                text(label)
                    .size(sz(12))
                    .font(colony_ui::ui_font())
                    .style(move |_| style::text_style_muted(theme))
                    .width(Length::Fill),
                action_button(theme, strings.ok, UiMessage::DismissAudioNotice),
            ]
            .spacing(spacing::XXL)
            .align_y(Alignment::Center)
            .into()
        });
        let output = section(
            &typo,
            open,
            "audio_output",
            strings.preferences_section_output,
            strings.preferences_output_desc,
            notice.into_iter().chain([
                kit::setting_row(
                    &typo,
                    strings.preferences_output_device,
                    strings.preferences_output_device_desc,
                    choice(
                        &typo,
                        [AudioOutputDevice::System, AudioOutputDevice::UsbHeadset],
                        |device| device.label(language),
                        &settings.output_device,
                        UiMessage::SetAudioOutputDevice,
                    ),
                ),
                kit::setting_row(
                    &typo,
                    strings.preferences_missing_device,
                    strings.preferences_missing_device_desc,
                    choice(
                        &typo,
                        [
                            MissingDeviceBehavior::SwitchToSystem,
                            MissingDeviceBehavior::PausePlayback,
                        ],
                        |behavior| behavior.label(language),
                        &settings.missing_device_behavior,
                        UiMessage::SetMissingDeviceBehavior,
                    ),
                ),
            ]),
        );

        let speed = f32::from(settings.default_playback_speed) / 10.0;
        let playback = section(
            &typo,
            open,
            "audio_playback",
            strings.preferences_section_playback,
            strings.preferences_playback_desc,
            [
                kit::functional_toggle(
                    &typo,
                    strings.preferences_gapless,
                    strings.preferences_gapless_desc,
                    settings.gapless_playback,
                    UiMessage::SetGaplessPlayback(!settings.gapless_playback),
                ),
                kit::setting_row(
                    &typo,
                    strings.preferences_default_playback_speed,
                    strings.preferences_default_playback_speed_desc,
                    row![
                        slider(0.5..=2.0, speed, |value| {
                            UiMessage::SetDefaultPlaybackSpeed((value * 10.0).round() as u8)
                        })
                        .step(0.1_f32),
                        muted_value(format!("{speed:.1}x")),
                    ]
                    .spacing(spacing::LG)
                    .align_y(Alignment::Center),
                ),
                kit::functional_toggle(
                    &typo,
                    strings.preferences_pause_on_focus,
                    strings.preferences_pause_on_focus_desc,
                    settings.pause_on_focus_loss,
                    UiMessage::SetPauseOnFocusLoss(!settings.pause_on_focus_loss),
                ),
            ],
        );

        let volume = section(
            &typo,
            open,
            "audio_volume",
            strings.preferences_section_volume,
            strings.preferences_volume_desc,
            [
                kit::setting_row(
                    &typo,
                    strings.preferences_volume_level,
                    strings.preferences_volume_level_desc,
                    choice(
                        &typo,
                        [VolumeLevel::Quiet, VolumeLevel::Normal, VolumeLevel::Loud],
                        |level| level.label(language),
                        &settings.volume_level,
                        UiMessage::SetVolumeLevel,
                    ),
                ),
                kit::setting_row(
                    &typo,
                    strings.preferences_default_volume,
                    strings.preferences_default_volume_desc,
                    row![
                        slider(0.0..=100.0, f32::from(settings.default_volume), |value| {
                            UiMessage::SetDefaultVolume(value.round().clamp(0.0, 100.0) as u8)
                        }),
                        muted_value(format!("{} %", settings.default_volume)),
                    ]
                    .spacing(spacing::LG)
                    .align_y(Alignment::Center),
                ),
            ],
        );

        let mut equalizer_settings = vec![
            kit::functional_toggle(
                &typo,
                strings.preferences_enable_eq,
                strings.preferences_enable_eq_desc,
                settings.eq_enabled,
                UiMessage::SetEqEnabled(!settings.eq_enabled),
            ),
            kit::setting_row(
                &typo,
                strings.preferences_eq_preset,
                strings.preferences_eq_preset_desc,
                choice(
                    &typo,
                    [
                        EqPreset::Flat,
                        EqPreset::Bass,
                        EqPreset::Treble,
                        EqPreset::Vocal,
                        EqPreset::Custom,
                    ],
                    |preset| preset.label(language),
                    &settings.eq_preset,
                    UiMessage::SetEqPreset,
                ),
            ),
            kit::setting_row(
                &typo,
                strings.preferences_reset_eq,
                strings.preferences_reset_eq_desc,
                action_button(theme, strings.preferences_reset_button, UiMessage::ResetEq),
            ),
        ];
        if settings.eq_enabled {
            equalizer_settings.push(eq_band_controls(theme, settings));
        }
        let equalizer = section(
            &typo,
            open,
            "audio_equalizer",
            strings.preferences_section_equalizer,
            strings.preferences_equalizer_desc,
            equalizer_settings,
        );

        let advanced = section(
            &typo,
            open,
            "audio_advanced",
            strings.preferences_section_advanced,
            strings.preferences_audio_advanced_desc,
            [
                kit::setting_row(
                    &typo,
                    strings.preferences_audio_stability,
                    strings.preferences_audio_stability_desc,
                    choice(
                        &typo,
                        [
                            AudioStabilityMode::Auto,
                            AudioStabilityMode::Stable,
                            AudioStabilityMode::LowLatency,
                        ],
                        |mode| mode.label(language),
                        &settings.audio_stability_mode,
                        UiMessage::SetAudioStabilityMode,
                    ),
                ),
                kit::setting_row(
                    &typo,
                    reset_engine.title(language),
                    reset_engine.description(language),
                    action_controls(theme, strings, language, self.ui.pending_action, reset_engine),
                ),
                kit::functional_toggle(
                    &typo,
                    strings.preferences_audio_logs,
                    strings.preferences_audio_logs_desc,
                    settings.audio_debug_logs,
                    UiMessage::SetAudioDebugLogs(!settings.audio_debug_logs),
                ),
            ],
        );

        kit::category_heading(&typo, strings.preferences_cat_audio, strings.preferences_audio_desc)
            .push(output)
            .push(playback)
            .push(volume)
            .push(equalizer)
            .push(advanced)
    }
}
