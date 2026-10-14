use super::super::{
    DeclarativeAction, Element, GrapeApp, InterfaceLanguage, Length, UiMessage, UiStrings, button,
    row, spacing, style, sz, text,
};
use colony_ui::Typography;
use colony_ui::preferences::Change;
use colony_ui::widgets::{self as kit, PreferencesView};
use iced::widget::{Column, column};
use std::collections::HashSet;

/// The kit's view of the shared preferences, with Grape's messages.
pub(super) type KitView<'p> =
    PreferencesView<'p, fn(Change) -> UiMessage, fn(&'static str) -> UiMessage>;

impl GrapeApp {
    pub(super) fn kit_preferences(&self) -> KitView<'_> {
        PreferencesView::new(
            colony_ui::Typography::current(),
            &self.ui.settings.standard,
            &self.ui.preferences_sections,
            UiMessage::Preference as fn(Change) -> UiMessage,
            UiMessage::TogglePreferencesSection as fn(&'static str) -> UiMessage,
        )
    }
}

/// One of Grape's own Preferences sections, drawn the way the kit draws
/// Colony's: a collapsible row, open while `key` is in `open`, starting with
/// what the section does and then its settings.
///
/// `key` must be unique across every category, Colony's keys included
/// ("theme", "vision"...), because one set holds them all.
pub(super) fn section<'a>(
    typo: &Typography,
    open: &HashSet<String>,
    key: &'static str,
    title: &str,
    description: &str,
    settings: impl IntoIterator<Item = Element<'a, UiMessage>>,
) -> Element<'a, UiMessage> {
    kit::collapsible_section(
        typo,
        title,
        open.contains(key),
        UiMessage::TogglePreferencesSection(key),
        column![
            kit::section_description(typo, description),
            Column::with_children(settings).spacing(spacing::SM).width(Length::Fill),
        ]
        .into(),
    )
}

/// A choice between a fixed set of values, labelled in the interface
/// language. The message carries the value, never the label.
pub(super) fn choice<'a, T, const N: usize>(
    typo: &Typography,
    values: [T; N],
    label: impl Fn(T) -> &'static str,
    selected: &T,
    on_select: fn(T) -> UiMessage,
) -> Element<'a, UiMessage>
where
    T: Copy + PartialEq + 'a,
{
    kit::dropdown(typo, values.map(|value| (value, label(value).to_string())), selected, on_select)
}

pub(super) fn action_button(
    theme: style::ThemeTokens,
    label: &'static str,
    message: UiMessage,
) -> iced::widget::Button<'static, UiMessage> {
    button(
        text(label)
            .size(sz(12))
            .font(colony_ui::ui_font_medium())
            .style(move |_| style::text_style_primary(theme)),
    )
    .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
    .padding([spacing::MD, spacing::XL])
    .on_press(message)
}

pub(super) fn action_controls(
    theme: style::ThemeTokens,
    strings: &'static UiStrings,
    language: InterfaceLanguage,
    pending_action: Option<DeclarativeAction>,
    action: DeclarativeAction,
) -> Element<'static, UiMessage> {
    if pending_action == Some(action) {
        row![
            action_button(
                theme,
                action.confirm_label(language),
                UiMessage::ConfirmDeclarativeAction(action)
            ),
            action_button(theme, strings.cancel, UiMessage::CancelDeclarativeAction),
        ]
        .spacing(spacing::LG)
        .into()
    } else {
        action_button(
            theme,
            action.button_label(language),
            UiMessage::RequestDeclarativeAction(action),
        )
        .into()
    }
}
