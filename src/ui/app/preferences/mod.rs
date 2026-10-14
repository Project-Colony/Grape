mod accessibility;
mod audio;
mod general;
mod helpers;

use super::{Element, GrapeApp, Length, PreferencesTab, UiMessage, container, spacing};
use colony_ui::i18n::t;
use colony_ui::widgets::{self as kit, STANDARD_CATEGORIES};
use iced::widget::stack;

impl GrapeApp {
    /// Preferences, built from colony-ui's kit (design/settings-page.md): the
    /// header with Close, Colony's three categories then Audio, and the
    /// selected category scrolling on the right. It replaces the library in
    /// the content area; the top bar and the player stay.
    pub(crate) fn preferences_view(&self) -> Element<'_, UiMessage> {
        let typo = colony_ui::Typography::current();
        let content: Element<'_, UiMessage> = match self.ui.preferences_tab {
            PreferencesTab::General => self.general_preferences().into(),
            PreferencesTab::Appearance => self.kit_preferences().appearance().into(),
            PreferencesTab::Accessibility => self.accessibility_preferences().into(),
            PreferencesTab::Audio => self.audio_preferences().into(),
        };
        let labels = STANDARD_CATEGORIES
            .iter()
            .map(|key| t(key))
            .chain([self.strings().preferences_cat_audio]);
        let categories =
            kit::category_list(&typo, labels, self.ui.preferences_tab.index(), |index| {
                UiMessage::PreferencesTabSelected(PreferencesTab::ALL[index])
            });
        // Close sends what the logo sends: two ways out, one message.
        let page = kit::preferences_page(&typo, categories, content, UiMessage::ToggleLogoMenu);

        if !self.ui.theme_notice {
            return page;
        }
        // A theme change applies at once; the toast makes a switch to a
        // neighbouring variant noticeable. Bottom-left, over the page.
        let toast = kit::toast(&typo, t("theme_applied"), UiMessage::DismissThemeNotice);
        stack![page, container(toast).padding(spacing::REGION).align_bottom(Length::Fill)].into()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::InterfaceLanguage;
    use crate::ui::i18n::strings;
    use colony_ui::fonts::{APP_FAMILY, BYTES, DYSLEXIA_FAMILY};
    use colony_ui::{FontSize, TextSize, Typography};
    // The font reader iced's own text stack uses, as in style.rs.
    use iced::advanced::graphics::text::cosmic_text::skrifa::instance::{LocationRef, Size};
    use iced::advanced::graphics::text::cosmic_text::skrifa::{FontRef, MetadataProvider};

    /// design/i18n.md: French runs longer, and a layout that only fits the
    /// English breaks. Grape's own category sits in the kit's list, 180 wide
    /// at a text scale of 1.0 and growing with it, so its name has to fit
    /// there in both locales and both interface fonts at every text size, as
    /// colony-ui's own test holds Colony's three.
    #[test]
    fn grape_category_names_fit_the_category_list() {
        // colony-ui's category_list: 180 wide at 1.0, then each nav_item's
        // horizontal padding and the list's right gutter.
        const LIST_WIDTH: f32 = 180.0;
        const CHROME: f32 = 14.0 * 2.0 + 16.0;

        for (family, bytes) in [(APP_FAMILY, BYTES[0]), (DYSLEXIA_FAMILY, BYTES[3])] {
            let font = FontRef::new(bytes).expect("the font parses");
            let charmap = font.charmap();
            let advances = font.glyph_metrics(Size::unscaled(), LocationRef::default());
            let units_per_em =
                f32::from(font.metrics(Size::unscaled(), LocationRef::default()).units_per_em);
            let em = |label: &str| -> f32 {
                label
                    .chars()
                    .map(|c| {
                        let glyph = charmap.map(c).expect("the font has every letter");
                        advances.advance_width(glyph).unwrap_or(0.0)
                    })
                    .sum::<f32>()
                    / units_per_em
            };

            for language in [InterfaceLanguage::English, InterfaceLanguage::French] {
                let label = strings(language).preferences_cat_audio;
                for font_size in FontSize::ALL {
                    for text_size in TextSize::ALL {
                        let typo = Typography {
                            scale: font_size.factor() * text_size.factor(),
                            ..Typography::default()
                        };
                        let needed = f32::mul_add(em(label), typo.sz(13), CHROME);
                        let width = LIST_WIDTH * typo.scale;
                        assert!(
                            needed <= width,
                            "{label:?} in {family} at {:.4}x needs {needed:.1}, the list is {width:.1}",
                            typo.scale
                        );
                    }
                }
            }
        }
    }
}
