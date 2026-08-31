use super::super::*;
use super::helpers::*;
use iced::widget::column;

impl GrapeApp {
    pub(super) fn appearance_preferences_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let language = self.language();
        let strings = self.strings();


        let typography_group = || {
            column![
                row![
                    setting_label(
                        theme,
                        strings.ui_text_scale_title,
                        strings.ui_text_scale_subtitle
                    ),
                    controls(
                        column![
                            slider(
                                0.0..=2.0,
                                self.ui.settings.text_scale.slider_value(),
                                |value| UiMessage::SetTextScale(TextScale::from_slider_value(
                                    value
                                )),
                            ),
                            text(self.ui.settings.text_scale.label(language))
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(spacing::MD)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.interface_density_title,
                        strings.interface_density_subtitle
                    ),
                    controls(
                        row![
                            option_button(
                                theme,
                                self.ui.settings.interface_density == InterfaceDensity::Compact,
                                InterfaceDensity::Compact.label(language),
                                UiMessage::SetInterfaceDensity(InterfaceDensity::Compact),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.interface_density == InterfaceDensity::Comfort,
                                InterfaceDensity::Comfort.label(language),
                                UiMessage::SetInterfaceDensity(InterfaceDensity::Comfort),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.interface_density == InterfaceDensity::Large,
                                InterfaceDensity::Large.label(language),
                                UiMessage::SetInterfaceDensity(InterfaceDensity::Large),
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

        let appearance_theme_content = || {

            // Colony's own picker, so Grape's Appearance page looks like every
            // other Colony program's: one row per family with its Nerd Font
            // glyph, then a card per variant filled with that variant's swatch
            // colours and crossed by its accent bar. A card that does not
            // resemble the theme it selects is a picker that lies.
            // The change is immediate; the toast exists because switching to a
            // neighbouring variant is easy to miss.
            let applied = self.ui.theme_notice.then(|| {
                row![
                    text(strings.theme_applied)
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
                    .style(move |_, status| style::button_style(
                        theme,
                        style::ButtonKind::Tab { selected: false },
                        status
                    ))
                    .padding([spacing::SM, spacing::LG])
                    .on_press(UiMessage::DismissThemeNotice),
                ]
                .spacing(spacing::MD)
                .align_y(Alignment::Center)
            });

            let picker = column![colony_ui::widgets::theme_picker(
                &self.typography(),
                &self.ui.settings.theme_family,
                &self.ui.settings.theme_variant,
                |family, variant| UiMessage::SetTheme(family.to_string(), variant.to_string()),
            )];
            let picker = match applied {
                Some(toast) => picker.push(toast),
                None => picker,
            };
            picker.spacing(spacing::MD).padding(SECTION_PADDING)
        };

        let appearance_accents_content = || {
            column![
                // Colony's accent row: eight filled circles from
                // tokens/accents.toml with a check on the selected one. `None`
                // is auto, which resolves to the theme's own accent rather than
                // being stored as a colour.
                colony_ui::widgets::accent_picker(
                    &self.typography(),
                    if self.ui.settings.accent_auto {
                        None
                    } else {
                        Some(self.ui.settings.accent_color.colony_key())
                    },
                    // Every key in ACCENT_OVERRIDES maps to a variant, so the
                    // fallback is unreachable; keeping the user's current
                    // choice is the harmless answer if that ever stops holding.
                    |key| {
                        UiMessage::SetAccentColor(
                            AccentColor::from_colony_key(key)
                                .unwrap_or(self.ui.settings.accent_color),
                        )
                    },
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.auto_accent_title,
                    strings.auto_accent_subtitle,
                    self.ui.settings.accent_auto,
                    UiMessage::SetAccentAuto(!(self.ui.settings.accent_auto)),
                ),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let appearance_effects_content = || {
            column![
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.transparency_blur_title,
                    strings.transparency_blur_subtitle,
                    self.ui.settings.transparency_blur,
                    UiMessage::SetTransparencyBlur(!(self.ui.settings.transparency_blur)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.ui_animations_title,
                    strings.ui_animations_subtitle,
                    self.ui.settings.ui_animations,
                    UiMessage::SetUiAnimations(!(self.ui.settings.ui_animations)),
                ),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let appearance_preview_content = || {
            column![
                container(
                    column![
                        text(strings.preview_card_title)
                            .size(theme.size(13))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme)),
                        text(strings.preview_theme_label(
                            // The variant's display name comes from Colony's
                            // shared labels, so it is translated once for every
                            // program rather than in each program's locale file.
                            colony_ui::theme::family(&self.ui.settings.theme_family)
                                .and_then(|f| f.variant(&self.ui.settings.theme_variant))
                                .map_or("", |v| colony_ui::i18n::t(v.label_key)),
                            // Colony owns the accent names, so they are
                            // translated once for the whole ecosystem.
                            colony_ui::ACCENT_OVERRIDES
                                .iter()
                                .find(|a| a.key == self.ui.settings.accent_color.colony_key())
                                .map_or("", |a| colony_ui::i18n::t(a.label_key)),
                            self.ui.settings.interface_density.label(language),
                        ))
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme)),
                        text(strings.preview_text_label(
                            self.ui.settings.text_scale.label(language),
                            if self.ui.settings.transparency_blur {
                                strings.enabled_masc
                            } else {
                                strings.disabled_masc
                            },
                            if self.ui.settings.ui_animations {
                                strings.enabled_fem
                            } else {
                                strings.disabled_fem
                            },
                        ))
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme)),
                    ]
                    .spacing(spacing::SM),
                )
                .padding(spacing::XXL)
                .width(Length::Fill)
                .style(move |_| style::surface_style(theme, style::Surface::Panel)),
            ]
            .spacing(spacing::XXL)
            .padding(SECTION_PADDING)
        };

        let appearance_panel = scrollable(
            column![
                column![
                    text(strings.appearance_title)
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text(strings.appearance_subtitle)
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                ]
                .spacing(spacing::MD),
                section_header(
                    theme,
                    strings.appearance_theme_title,
                    self.ui.preferences_sections.appearance_theme,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearanceTheme),
                ),
                if self.ui.preferences_sections.appearance_theme {
                    appearance_theme_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.appearance_accents_title,
                    self.ui.preferences_sections.appearance_accents,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearanceAccents),
                ),
                if self.ui.preferences_sections.appearance_accents {
                    appearance_accents_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.appearance_typography_title,
                    self.ui.preferences_sections.appearance_typography,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearanceTypography),
                ),
                if self.ui.preferences_sections.appearance_typography {
                    typography_group()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.appearance_effects_title,
                    self.ui.preferences_sections.appearance_effects,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearanceEffects),
                ),
                if self.ui.preferences_sections.appearance_effects {
                    appearance_effects_content()
                } else {
                    column![]
                },
                section_header(
                    theme,
                    strings.appearance_preview_title,
                    self.ui.preferences_sections.appearance_preview,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearancePreview),
                ),
                if self.ui.preferences_sections.appearance_preview {
                    appearance_preview_content()
                } else {
                    column![]
                },
            ]
            .spacing(spacing::SECTION),
        )
        .on_scroll(|viewport| UiMessage::PreferencesScrolled {
            tab: PreferencesTab::Appearance,
            offset_y: viewport.absolute_offset().y,
        })
        .id(Id::new(PREFERENCES_APPEARANCE_SCROLL_ID))
        .height(Length::Fill);

        appearance_panel.into()
    }
}
