use super::super::*;
use super::helpers::*;
use iced::widget::column;

impl GrapeApp {
    pub(super) fn audio_preferences_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let language = self.language();
        let strings = self.strings();
        let reset_audio_action = DeclarativeAction::ResetAudioEngine;
        let volume_value = self.ui.settings.default_volume as f32;
        let crossfade_value = self.ui.settings.crossfade_seconds as f32;

        let audio_output_content = || {
            let notice = self.ui.audio_notice.as_deref().map(|label| {
                row![
                    text(label)
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                        .width(Length::Fill),
                    button(
                        text(strings.ok)
                            .size(theme.size(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme)),
                    )
                    .style(move |_, status| {
                        style::button_style(
                            theme,
                            style::ButtonKind::Tab { selected: false },
                            status,
                        )
                    })
                    .padding([spacing::MD, spacing::XL])
                    .on_press(UiMessage::DismissAudioNotice)
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL)
            });
            column![
                section_hint(theme, strings.audio_output_hint),
                if let Some(notice) = notice { notice } else { row![] },
                row![
                    setting_label(
                        theme,
                        strings.output_device_title,
                        strings.output_device_subtitle
                    ),
                    controls(
                        row![
                            option_button(
                                theme,
                                self.ui.settings.output_device == AudioOutputDevice::System,
                                AudioOutputDevice::System.label(language),
                                UiMessage::SetAudioOutputDevice(AudioOutputDevice::System)
                            ),
                            option_button(
                                theme,
                                self.ui.settings.output_device == AudioOutputDevice::UsbHeadset,
                                AudioOutputDevice::UsbHeadset.label(language),
                                UiMessage::SetAudioOutputDevice(AudioOutputDevice::UsbHeadset)
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.missing_device_title,
                        strings.missing_device_subtitle
                    ),
                    controls(
                        row![
                            option_button(
                                theme,
                                self.ui.settings.missing_device_behavior
                                    == MissingDeviceBehavior::SwitchToSystem,
                                MissingDeviceBehavior::SwitchToSystem.label(language),
                                UiMessage::SetMissingDeviceBehavior(
                                    MissingDeviceBehavior::SwitchToSystem
                                )
                            ),
                            option_button(
                                theme,
                                self.ui.settings.missing_device_behavior
                                    == MissingDeviceBehavior::PausePlayback,
                                MissingDeviceBehavior::PausePlayback.label(language),
                                UiMessage::SetMissingDeviceBehavior(
                                    MissingDeviceBehavior::PausePlayback
                                )
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let audio_playback_content = || {
            column![
                section_hint(theme, strings.audio_playback_hint),
                row![
                    setting_label(theme, strings.gapless_title, strings.gapless_subtitle),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.gapless_playback,
                            UiMessage::SetGaplessPlayback(true),
                            UiMessage::SetGaplessPlayback(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.crossfade_title, strings.crossfade_subtitle),
                    controls(
                        column![
                            slider(0.0..=12.0, crossfade_value, |value| {
                                UiMessage::SetCrossfadeSeconds(value.round().clamp(0.0, 12.0) as u8)
                            }),
                            text(format!("{} s", self.ui.settings.crossfade_seconds))
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_muted(theme))
                        ]
                        .spacing(spacing::MD)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.automix_title, strings.automix_subtitle),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.automix_enabled,
                            UiMessage::SetAutomixEnabled(true),
                            UiMessage::SetAutomixEnabled(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let audio_volume_content = || {
            column![
                section_hint(theme, strings.audio_volume_hint),
                row![
                    setting_label(
                        theme,
                        strings.normalize_volume_title,
                        strings.normalize_volume_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.normalize_volume,
                            UiMessage::SetNormalizeVolume(true),
                            UiMessage::SetNormalizeVolume(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.volume_level_title, strings.volume_level_subtitle),
                    controls(
                        row![
                            option_button(
                                theme,
                                self.ui.settings.volume_level == VolumeLevel::Quiet,
                                VolumeLevel::Quiet.label(language),
                                UiMessage::SetVolumeLevel(VolumeLevel::Quiet)
                            ),
                            option_button(
                                theme,
                                self.ui.settings.volume_level == VolumeLevel::Normal,
                                VolumeLevel::Normal.label(language),
                                UiMessage::SetVolumeLevel(VolumeLevel::Normal)
                            ),
                            option_button(
                                theme,
                                self.ui.settings.volume_level == VolumeLevel::Loud,
                                VolumeLevel::Loud.label(language),
                                UiMessage::SetVolumeLevel(VolumeLevel::Loud)
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.default_volume_title,
                        strings.default_volume_subtitle
                    ),
                    controls(
                        column![
                            slider(0.0..=100.0, volume_value, |value| {
                                UiMessage::SetDefaultVolume(value.round().clamp(0.0, 100.0) as u8)
                            }),
                            text(format!("{} %", self.ui.settings.default_volume))
                                .size(theme.size(13))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_muted(theme))
                        ]
                        .spacing(spacing::MD)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let audio_equalizer_content = || {
            let band_controls = if self.ui.settings.eq_enabled {
                eq_band_controls(theme, &self.ui.settings)
            } else {
                column![].into()
            };
            column![
                section_hint(theme, strings.equalizer_hint),
                row![
                    setting_label(theme, strings.enable_eq_title, strings.enable_eq_subtitle),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.eq_enabled,
                            UiMessage::SetEqEnabled(true),
                            UiMessage::SetEqEnabled(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.eq_preset_title, strings.eq_preset_subtitle),
                    controls(
                        column![
                            row![
                                option_button(
                                    theme,
                                    self.ui.settings.eq_preset == EqPreset::Flat,
                                    EqPreset::Flat.label(language),
                                    UiMessage::SetEqPreset(EqPreset::Flat)
                                ),
                                option_button(
                                    theme,
                                    self.ui.settings.eq_preset == EqPreset::Bass,
                                    EqPreset::Bass.label(language),
                                    UiMessage::SetEqPreset(EqPreset::Bass)
                                ),
                                option_button(
                                    theme,
                                    self.ui.settings.eq_preset == EqPreset::Treble,
                                    EqPreset::Treble.label(language),
                                    UiMessage::SetEqPreset(EqPreset::Treble)
                                ),
                            ]
                            .spacing(spacing::LG),
                            row![
                                option_button(
                                    theme,
                                    self.ui.settings.eq_preset == EqPreset::Vocal,
                                    EqPreset::Vocal.label(language),
                                    UiMessage::SetEqPreset(EqPreset::Vocal)
                                ),
                                option_button(
                                    theme,
                                    self.ui.settings.eq_preset == EqPreset::Custom,
                                    EqPreset::Custom.label(language),
                                    UiMessage::SetEqPreset(EqPreset::Custom)
                                ),
                            ]
                            .spacing(spacing::LG),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.reset_eq_title, strings.reset_eq_subtitle),
                    controls(action_button(theme, strings.reset_button, UiMessage::ResetEq).into()),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                if self.ui.settings.eq_enabled {
                    band_controls
                } else {
                    column![].into()
                },
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let audio_advanced_content = || {
            column![
                section_hint(theme, strings.audio_advanced_hint),
                row![
                    setting_label(
                        theme,
                        strings.audio_stability_title,
                        strings.audio_stability_subtitle
                    ),
                    controls(
                        row![
                            option_button(
                                theme,
                                self.ui.settings.audio_stability_mode == AudioStabilityMode::Auto,
                                AudioStabilityMode::Auto.label(language),
                                UiMessage::SetAudioStabilityMode(AudioStabilityMode::Auto)
                            ),
                            option_button(
                                theme,
                                self.ui.settings.audio_stability_mode == AudioStabilityMode::Stable,
                                AudioStabilityMode::Stable.label(language),
                                UiMessage::SetAudioStabilityMode(AudioStabilityMode::Stable)
                            ),
                            option_button(
                                theme,
                                self.ui.settings.audio_stability_mode
                                    == AudioStabilityMode::LowLatency,
                                AudioStabilityMode::LowLatency.label(language),
                                UiMessage::SetAudioStabilityMode(AudioStabilityMode::LowLatency)
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        reset_audio_action.title(language),
                        reset_audio_action.description(language)
                    ),
                    controls(action_controls(
                        theme,
                        strings,
                        language,
                        self.ui.pending_action,
                        reset_audio_action
                    )),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(theme, strings.audio_logs_title, strings.audio_logs_subtitle),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.audio_debug_logs,
                            UiMessage::SetAudioDebugLogs(true),
                            UiMessage::SetAudioDebugLogs(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let audio_panel = scrollable(
            column![
                column![
                    text(strings.audio_title)
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text(strings.audio_subtitle)
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                ]
                .spacing(spacing::MD),
                section_header(
                    theme,
                    strings.audio_output_title,
                    self.ui.preferences_sections.audio_output,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioOutput),
                ),
                if self.ui.preferences_sections.audio_output {
                    audio_output_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.audio_playback_title,
                    self.ui.preferences_sections.audio_playback,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioPlayback),
                ),
                if self.ui.preferences_sections.audio_playback {
                    audio_playback_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.audio_volume_title,
                    self.ui.preferences_sections.audio_volume,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioVolume),
                ),
                if self.ui.preferences_sections.audio_volume {
                    audio_volume_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.equalizer_title,
                    self.ui.preferences_sections.audio_equalizer,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioEqualizer),
                ),
                if self.ui.preferences_sections.audio_equalizer {
                    audio_equalizer_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.audio_advanced_title,
                    self.ui.preferences_sections.audio_advanced,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioAdvanced),
                ),
                if self.ui.preferences_sections.audio_advanced {
                    audio_advanced_content()
                } else {
                    column![]
                },
            ]
            .spacing(spacing::SECTION),
        )
        .on_scroll(|viewport| UiMessage::PreferencesScrolled {
            tab: PreferencesTab::Audio,
            offset_y: viewport.absolute_offset().y,
        })
        .id(Id::new(PREFERENCES_AUDIO_SCROLL_ID))
        .height(Length::Fill);

        audio_panel.into()
    }
}
