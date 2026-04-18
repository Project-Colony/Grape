use super::super::*;
use super::helpers::*;
use iced::widget::column;

impl GrapeApp {
    pub(super) fn appearance_preferences_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let language = self.language();
        let strings = self.strings();

        let accent_button = |accent: AccentColor| {
            let selected = self.ui.settings.accent_color == accent;
            button(
                row![
                    text("●")
                        .size(theme.size(14))
                        .style(move |_| style::text_style(style::accent_color_value(accent))),
                    text(accent.label(language))
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_primary(theme)),
                ]
                .spacing(spacing::MD)
                .align_y(Alignment::Center),
            )
            .style(move |_, status| {
                style::button_style(theme, style::ButtonKind::Tab { selected }, status)
            })
            .padding([spacing::MD, spacing::XL])
            .on_press(UiMessage::SetAccentColor(accent))
        };

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
            let theme_category = |label: &'static str,
                                  expanded: bool,
                                  message: UiMessage,
                                  options: Element<'static, UiMessage>|
             -> Element<'static, UiMessage> {
                let chevron = if expanded { "▾" } else { "▸" };
                let expanded_content: Element<'static, UiMessage> = if expanded {
                    container(
                        row![
                            text("↳")
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
                                .style(move |_| style::text_style_muted(theme)),
                            options,
                        ]
                        .spacing(spacing::LG)
                        .align_y(Alignment::Center),
                    )
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 24.0,
                    })
                    .width(Length::Fill)
                    .into()
                } else {
                    column![].into()
                };
                column![
                    button(
                        row![
                            text(label)
                                .size(theme.size(13))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_primary(theme)),
                            text(chevron)
                                .size(theme.size(13))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(spacing::XL)
                        .align_y(Alignment::Center),
                    )
                    .style(move |_, status| {
                        style::button_style(
                            theme,
                            style::ButtonKind::ListItem { selected: expanded, focused: false },
                            status,
                        )
                    })
                    .padding([spacing::LG, spacing::XXL])
                    .width(Length::Fill)
                    .on_press(message),
                    expanded_content,
                ]
                .spacing(spacing::MD)
                .into()
            };

            column![
                row![
                    theme_category(
                        "Catppuccin",
                        self.ui.theme_categories.catppuccin,
                        UiMessage::ToggleThemeCategory(ThemeCategory::Catppuccin),
                        row![
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::Latte,
                                ThemeMode::Latte.label(language),
                                UiMessage::SetThemeMode(ThemeMode::Latte),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::Frappe,
                                ThemeMode::Frappe.label(language),
                                UiMessage::SetThemeMode(ThemeMode::Frappe),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::Macchiato,
                                ThemeMode::Macchiato.label(language),
                                UiMessage::SetThemeMode(ThemeMode::Macchiato),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::Mocha,
                                ThemeMode::Mocha.label(language),
                                UiMessage::SetThemeMode(ThemeMode::Mocha),
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                    theme_category(
                        "Gruvbox",
                        self.ui.theme_categories.gruvbox,
                        UiMessage::ToggleThemeCategory(ThemeCategory::Gruvbox),
                        row![
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::GruvboxLight,
                                strings.theme_light_mode,
                                UiMessage::SetThemeMode(ThemeMode::GruvboxLight),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::GruvboxDark,
                                strings.theme_dark_mode,
                                UiMessage::SetThemeMode(ThemeMode::GruvboxDark),
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                    theme_category(
                        "Everblush",
                        self.ui.theme_categories.everblush,
                        UiMessage::ToggleThemeCategory(ThemeCategory::Everblush),
                        row![
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::EverblushLight,
                                strings.theme_light_mode,
                                UiMessage::SetThemeMode(ThemeMode::EverblushLight),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::EverblushDark,
                                strings.theme_dark_mode,
                                UiMessage::SetThemeMode(ThemeMode::EverblushDark),
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                    theme_category(
                        "Kanagawa",
                        self.ui.theme_categories.kanagawa,
                        UiMessage::ToggleThemeCategory(ThemeCategory::Kanagawa),
                        row![
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::KanagawaLight,
                                strings.theme_light_mode,
                                UiMessage::SetThemeMode(ThemeMode::KanagawaLight),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::KanagawaDark,
                                strings.theme_dark_mode,
                                UiMessage::SetThemeMode(ThemeMode::KanagawaDark),
                            ),
                            option_button(
                                theme,
                                self.ui.settings.theme_mode == ThemeMode::KanagawaJournal,
                                strings.theme_journal_mode,
                                UiMessage::SetThemeMode(ThemeMode::KanagawaJournal),
                            ),
                        ]
                        .spacing(spacing::LG)
                        .into(),
                    ),
                ]
                .spacing(spacing::XXL)
                .width(Length::Fill)
                .wrap(),
            ]
            .padding(SECTION_PADDING)
        };

        let appearance_accents_content = || {
            column![
                row![
                    accent_button(AccentColor::Red),
                    accent_button(AccentColor::Orange),
                    accent_button(AccentColor::Yellow),
                    accent_button(AccentColor::Green),
                    accent_button(AccentColor::Blue),
                    accent_button(AccentColor::Indigo),
                    accent_button(AccentColor::Violet),
                    accent_button(AccentColor::Amber),
                ]
                .spacing(spacing::LG)
                .wrap(),
                row![
                    setting_label(theme, strings.auto_accent_title, strings.auto_accent_subtitle),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.accent_auto,
                            UiMessage::SetAccentAuto(true),
                            UiMessage::SetAccentAuto(false),
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

        let appearance_effects_content = || {
            column![
                row![
                    setting_label(
                        theme,
                        strings.transparency_blur_title,
                        strings.transparency_blur_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.transparency_blur,
                            UiMessage::SetTransparencyBlur(true),
                            UiMessage::SetTransparencyBlur(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing::XXL),
                row![
                    setting_label(
                        theme,
                        strings.ui_animations_title,
                        strings.ui_animations_subtitle
                    ),
                    controls(
                        toggle_row(
                            theme,
                            strings,
                            self.ui.settings.ui_animations,
                            UiMessage::SetUiAnimations(true),
                            UiMessage::SetUiAnimations(false),
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

        let appearance_preview_content = || {
            column![
                container(
                    column![
                        text(strings.preview_card_title)
                            .size(theme.size(13))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme)),
                        text(strings.preview_theme_label(
                            self.ui.settings.theme_mode.label(language),
                            self.ui.settings.accent_color.label(language),
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
