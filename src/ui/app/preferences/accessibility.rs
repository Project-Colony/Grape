use super::super::*;
use super::helpers::*;
use colony_ui::i18n::t;
use colony_ui::preferences::Change;
use colony_ui::widgets as kit;
use iced::widget::Column;

impl GrapeApp {
    /// Vision, Motion, Navigation, Reading: the order design/settings-page.md
    /// gives a program with a Navigation section of its own.
    pub(super) fn accessibility_preferences(&self) -> Column<'_, UiMessage> {
        let typo = self.typography();
        let strings = self.strings();
        let settings = &self.ui.settings;
        let standard = &settings.standard;
        let open = &self.ui.preferences_sections;
        let prefs = self.kit_preferences();

        // The kit's Vision with Reduce transparency added, built from the same
        // parts under the same key, so it opens and closes like the kit's.
        let vision = section(
            &typo,
            open,
            "vision",
            t("settings_section_vision"),
            t("settings_vision_desc"),
            [
                kit::functional_toggle(
                    &typo,
                    t("settings_high_contrast"),
                    t("settings_high_contrast_desc"),
                    standard.high_contrast,
                    UiMessage::Preference(Change::HighContrast(!standard.high_contrast)),
                ),
                kit::functional_toggle(
                    &typo,
                    t("settings_dyslexia_font"),
                    t("settings_dyslexia_font_desc"),
                    standard.dyslexia_font,
                    UiMessage::Preference(Change::DyslexiaFont(!standard.dyslexia_font)),
                ),
                kit::functional_toggle(
                    &typo,
                    strings.preferences_reduce_transparency,
                    strings.preferences_reduce_transparency_desc,
                    settings.reduce_transparency,
                    UiMessage::SetReduceTransparency(!settings.reduce_transparency),
                ),
            ],
        );

        let navigation = section(
            &typo,
            open,
            "navigation",
            strings.preferences_section_navigation,
            strings.preferences_navigation_desc,
            [kit::functional_toggle(
                &typo,
                strings.preferences_highlight_focus,
                strings.preferences_highlight_focus_desc,
                settings.highlight_keyboard_focus,
                UiMessage::SetHighlightKeyboardFocus(!settings.highlight_keyboard_focus),
            )],
        );

        prefs
            .accessibility_heading()
            .push(vision)
            .push(prefs.motion_section())
            .push(navigation)
            .push(prefs.reading_section())
    }
}
