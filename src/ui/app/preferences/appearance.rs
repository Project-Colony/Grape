use super::super::*;
use super::helpers::*;
use colony_ui::i18n::t;
use colony_ui::preferences::Change;
use colony_ui::FontSize;
use iced::widget::column;

impl GrapeApp {
    pub(super) fn appearance_preferences_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let language = self.language();
        let strings = self.strings();
        let standard = &self.ui.settings.standard;


        let typography_group = || {
            column![
                colony_ui::widgets::setting_row(
                    &self.typography(),
                    t("settings_font_size"),
                    t("settings_font_size_desc"),
                    colony_ui::widgets::dropdown(
                        &self.typography(),
                        FontSize::ALL.map(|size| (size, t(size.label_key()).to_string())),
                        &standard.font_size,
                        |size| UiMessage::Preference(Change::FontSize(size)),
                    ),
                ),
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
                        .size(sz(12))
                        .font(colony_ui::ui_font())
                        .style(move |_| style::text_style_muted(theme))
                        .width(Length::Fill),
                    button(
                        text(strings.ok)
                            .size(sz(12))
                            .font(colony_ui::ui_font_medium())
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
                &standard.selected_theme,
                &standard.selected_variant,
                |family, variant| UiMessage::Preference(Change::Theme { family, variant }),
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
                // is the theme's own accent, which is not stored as a colour.
                colony_ui::widgets::accent_picker(
                    &self.typography(),
                    standard.picked_accent(),
                    |key| UiMessage::Preference(Change::Accent(key)),
                ),
                colony_ui::widgets::functional_toggle(
                    &self.typography(),
                    strings.auto_accent_title,
                    strings.auto_accent_subtitle,
                    standard.auto_accent,
                    UiMessage::Preference(Change::AutoAccent(!standard.auto_accent)),
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
                    t("settings_animations"),
                    t("settings_animations_desc"),
                    standard.animations,
                    UiMessage::Preference(Change::Animations(!standard.animations)),
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
                            .size(sz(13))
                            .font(colony_ui::ui_font_medium())
                            .style(move |_| style::text_style_primary(theme)),
                        text(strings.preview_theme_label(
                            // The variant's display name comes from Colony's
                            // shared labels, so it is translated once for every
                            // program rather than in each program's locale file.
                            colony_ui::theme::family(&standard.selected_theme)
                                .and_then(|f| f.variant(&standard.selected_variant))
                                .map_or("", |v| t(v.label_key)),
                            // Colony owns the accent names, so they are
                            // translated once for the whole ecosystem.
                            standard
                                .picked_accent()
                                .and_then(|key| {
                                    colony_ui::ACCENT_OVERRIDES.iter().find(|a| a.key == key)
                                })
                                .map_or(t("settings_default_accent"), |a| t(a.label_key)),
                            self.ui.settings.interface_density.label(language),
                        ))
                        .size(sz(12))
                        .font(colony_ui::ui_font())
                        .style(move |_| style::text_style_muted(theme)),
                        text(strings.preview_text_label(
                            t(standard.font_size.label_key()),
                            if self.ui.settings.transparency_blur {
                                strings.enabled_masc
                            } else {
                                strings.disabled_masc
                            },
                            if standard.animations {
                                strings.enabled_fem
                            } else {
                                strings.disabled_fem
                            },
                        ))
                        .size(sz(12))
                        .font(colony_ui::ui_font())
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
                        .size(sz(16))
                        .font(colony_ui::ui_font_bold())
                        .style(move |_| style::text_style_primary(theme)),
                    text(strings.appearance_subtitle)
                        .size(sz(13))
                        .font(colony_ui::ui_font())
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
