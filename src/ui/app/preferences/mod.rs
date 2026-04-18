mod accessibility;
mod appearance;
mod audio;
mod general;
mod helpers;

use super::*;
use iced::widget::column;

impl GrapeApp {
    pub(crate) fn preferences_view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let strings = self.strings();
        let header = row![
            text(strings.preferences_title)
                .size(theme.size(22))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            button(
                text(strings.close)
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem { selected: false, focused: false },
                    status,
                )
            })
            .padding([spacing::MD, spacing::XL])
            .on_press(UiMessage::ClosePreferences)
        ]
        .align_y(Alignment::Center)
        .spacing(spacing::XXL);

        let menu_button = |tab: PreferencesTab, label: &'static str| {
            button(
                text(label)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem {
                        selected: self.ui.preferences_tab == tab,
                        focused: false,
                    },
                    status,
                )
            })
            .padding([spacing::MD, spacing::XL])
            .width(Length::Fill)
            .on_press(UiMessage::PreferencesTabSelected(tab))
        };
        let menu = column![
            menu_button(PreferencesTab::General, strings.tab_general),
            menu_button(PreferencesTab::Appearance, strings.tab_appearance),
            menu_button(PreferencesTab::Accessibility, strings.tab_accessibility),
            menu_button(PreferencesTab::Audio, strings.tab_audio),
        ]
        .spacing(spacing::MD)
        .width(Length::Fill);

        let content_panel: Element<'_, UiMessage> = match self.ui.preferences_tab {
            PreferencesTab::General => self.general_preferences_panel().into(),
            PreferencesTab::Appearance => self.appearance_preferences_panel().into(),
            PreferencesTab::Accessibility => self.accessibility_preferences_panel().into(),
            PreferencesTab::Audio => self.audio_preferences_panel().into(),
        };

        let body = row![
            container(menu)
                .padding(spacing::SECTION)
                .width(Length::Fixed(200.0))
                .style(move |_| style::surface_style(theme, style::Surface::Sidebar)),
            container(content_panel)
                .padding(spacing::PANEL)
                .width(Length::Fill)
                .style(move |_| style::surface_style(theme, style::Surface::Panel))
        ]
        .spacing(spacing::SECTION)
        .height(Length::Fill);

        column![header, body].spacing(spacing::SECTION).height(Length::Fill).into()
    }
}
