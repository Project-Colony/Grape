use super::super::*;
use super::helpers::*;
use colony_ui::i18n::t;
use colony_ui::preferences::Change;
use colony_ui::TextSize;
use iced::widget::column;

impl GrapeApp {
    pub(super) fn accessibility_preferences_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        let standard = &self.ui.settings.standard;

        let vision_group = || {
            column![
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    t("settings_high_contrast"),
                    t("settings_high_contrast_desc"),
                    standard.high_contrast,
                    UiMessage::Preference(Change::HighContrast(!standard.high_contrast)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    t("settings_dyslexia_font"),
                    t("settings_dyslexia_font_desc"),
                    standard.dyslexia_font,
                    UiMessage::Preference(Change::DyslexiaFont(!standard.dyslexia_font)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.reduce_transparency_title,
                    strings.reduce_transparency_subtitle,
                    self.ui.settings.reduce_transparency,
                    UiMessage::SetReduceTransparency(!(self.ui.settings.reduce_transparency)),
                ),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let movement_group = || {
            column![colony_ui::widgets::functional_toggle(
                &self.typography(),
                t("settings_reduce_motion"),
                t("settings_reduce_motion_desc"),
                standard.reduce_motion,
                UiMessage::Preference(Change::ReduceMotion(!standard.reduce_motion)),
            )]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        // The four-step text size, which multiplies with Appearance's.
        let reading_group = || {
            column![colony_ui::widgets::setting_row(
                &self.typography(),
                t("settings_text_size_a11y"),
                t("settings_text_size_a11y_desc"),
                colony_ui::widgets::dropdown(
                    &self.typography(),
                    TextSize::ALL.map(|size| (size, t(size.label_key()).to_string())),
                    &standard.text_size_a11y,
                    |size| UiMessage::Preference(Change::TextSize(size)),
                ),
            )]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let navigation_group = || {
            column![
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.highlight_focus_title,
                    strings.highlight_focus_subtitle,
                    self.ui.settings.highlight_keyboard_focus,
                    UiMessage::SetHighlightKeyboardFocus(!(self.ui.settings.highlight_keyboard_focus)),
                ),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let playback_group = || {
            let playback_speed = self.ui.settings.default_playback_speed as f32 / 10.0;
            column![
                row![
                    setting_label(
                        theme,
                        strings.default_playback_speed_title,
                        strings.default_playback_speed_subtitle
                    ),
                    controls(
                        column![
                            slider(0.5..=2.0, playback_speed, |value| {
                                UiMessage::SetDefaultPlaybackSpeed((value * 10.0).round() as u8)
                            }),
                            text(format!("{:.1}x", playback_speed))
                                .size(sz(12))
                                .font(colony_ui::ui_font())
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(spacing::MD)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.pause_on_focus_title,
                    strings.pause_on_focus_subtitle,
                    self.ui.settings.pause_on_focus_loss,
                    UiMessage::SetPauseOnFocusLoss(!(self.ui.settings.pause_on_focus_loss)),
                ),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let accessibility_panel = scrollable(
            column![
                column![
                    text(strings.accessibility_title)
                        .size(sz(16))
                        .font(colony_ui::ui_font_bold())
                        .style(move |_| style::text_style_primary(theme)),
                    text(strings.accessibility_subtitle)
                        .size(sz(13))
                        .font(colony_ui::ui_font())
                        .style(move |_| style::text_style_muted(theme))
                ]
                .spacing(spacing::MD),
                section_header(
                    theme,
                    strings.vision_title,
                    self.ui.preferences_sections.accessibility_vision,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AccessibilityVision),
                ),
                if self.ui.preferences_sections.accessibility_vision {
                    vision_group()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.movement_title,
                    self.ui.preferences_sections.accessibility_movement,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AccessibilityMovement),
                ),
                if self.ui.preferences_sections.accessibility_movement {
                    movement_group()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.navigation_title,
                    self.ui.preferences_sections.accessibility_navigation,
                    UiMessage::TogglePreferencesSection(
                        PreferencesSection::AccessibilityNavigation
                    ),
                ),
                if self.ui.preferences_sections.accessibility_navigation {
                    navigation_group()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    t("settings_section_reading"),
                    self.ui.preferences_sections.accessibility_reading,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AccessibilityReading),
                ),
                if self.ui.preferences_sections.accessibility_reading {
                    reading_group()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.playback_title,
                    self.ui.preferences_sections.accessibility_playback,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AccessibilityPlayback),
                ),
                if self.ui.preferences_sections.accessibility_playback {
                    playback_group()
                } else {
                    column![]
                },
            ]
            .spacing(spacing::SECTION),
        )
        .on_scroll(|viewport| UiMessage::PreferencesScrolled {
            tab: PreferencesTab::Accessibility,
            offset_y: viewport.absolute_offset().y,
        })
        .id(Id::new(PREFERENCES_ACCESSIBILITY_SCROLL_ID))
        .height(Length::Fill);

        accessibility_panel.into()
    }
}
