use super::super::*;
use super::helpers::*;
use crate::config::AccessibleTextSize;
use iced::widget::column;

impl GrapeApp {
    pub(super) fn accessibility_preferences_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let language = self.language();
        let strings = self.strings();

        let vision_group = || {
            column![
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.large_text_title,
                    strings.large_text_subtitle,
                    self.ui.settings.accessibility_large_text,
                    UiMessage::SetAccessibilityLargeText(!(self.ui.settings.accessibility_large_text)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.high_contrast_title,
                    strings.high_contrast_subtitle,
                    self.ui.settings.accessibility_high_contrast,
                    UiMessage::SetAccessibilityHighContrast(!(self.ui.settings.accessibility_high_contrast)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.dyslexia_font_title,
                    strings.dyslexia_font_subtitle,
                    self.ui.settings.accessibility_dyslexia_font,
                    UiMessage::SetAccessibilityDyslexiaFont(!(self.ui.settings.accessibility_dyslexia_font)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.reduce_transparency_title,
                    strings.reduce_transparency_subtitle,
                    self.ui.settings.reduce_transparency,
                    UiMessage::SetReduceTransparency(!(self.ui.settings.reduce_transparency)),
                ),
                row![
                    setting_label(
                        theme,
                        strings.accessible_text_size_title,
                        strings.accessible_text_size_subtitle
                    ),
                    controls(
                        column![
                            slider(
                                0.0..=2.0,
                                self.ui.settings.accessible_text_size.slider_value(),
                                |value| {
                                    UiMessage::SetAccessibleTextSize(
                                        AccessibleTextSize::from_slider_value(value),
                                    )
                                },
                            ),
                            text(self.ui.settings.accessible_text_size.label(language))
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(spacing::MD)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let movement_group = || {
            column![
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.reduce_motion_title,
                    strings.reduce_motion_subtitle,
                    self.ui.settings.accessibility_reduce_motion,
                    UiMessage::SetAccessibilityReduceMotion(!(self.ui.settings.accessibility_reduce_motion)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.reduce_animations_title,
                    strings.reduce_animations_subtitle,
                    self.ui.settings.reduce_animations,
                    UiMessage::SetReduceAnimations(!(self.ui.settings.reduce_animations)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.reduce_transitions_title,
                    strings.reduce_transitions_subtitle,
                    self.ui.settings.reduce_transitions,
                    UiMessage::SetReduceTransitions(!(self.ui.settings.reduce_transitions)),
                ),
            ]
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
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
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
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text(strings.accessibility_subtitle)
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
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
