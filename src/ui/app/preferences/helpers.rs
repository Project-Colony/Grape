use super::super::*;
use iced::widget::column;

pub(super) const SECTION_PADDING: Padding = Padding {
    top: 4.0,
    right: 12.0,
    bottom: 0.0,
    left: 12.0,
};

pub(super) fn section_header(
    theme: style::ThemeTokens,
    label: &'static str,
    expanded: bool,
    message: UiMessage,
) -> iced::widget::Button<'static, UiMessage> {
    let chevron = if expanded { "▾" } else { "▸" };
    button(
        row![
            text(label)
                .size(theme.size(14))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            text(chevron)
                .size(theme.size(14))
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
    .on_press(message)
}

pub(super) fn section_hint(
    theme: style::ThemeTokens,
    label: &'static str,
) -> iced::widget::Text<'static> {
    text(label)
        .size(theme.size_accessible(12))
        .font(style::font_propo(Weight::Light))
        .style(move |_| style::text_style_muted(theme))
}

pub(super) fn setting_label(
    theme: style::ThemeTokens,
    title: &'static str,
    subtitle: &'static str,
) -> iced::widget::Column<'static, UiMessage> {
    column![
        text(title)
            .size(theme.size(13))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| style::text_style_primary(theme)),
        text(subtitle)
            .size(theme.size_accessible(12))
            .font(style::font_propo(Weight::Light))
            .style(move |_| style::text_style_muted(theme)),
    ]
    .spacing(spacing::XS)
    .width(Length::Fill)
}

pub(super) fn option_button(
    theme: style::ThemeTokens,
    selected: bool,
    label: &'static str,
    message: UiMessage,
) -> iced::widget::Button<'static, UiMessage> {
    button(
        text(label)
            .size(theme.size(12))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| style::text_style_primary(theme)),
    )
    .style(move |_, status| style::button_style(theme, style::ButtonKind::Tab { selected }, status))
    .padding([spacing::MD, spacing::XL])
    .on_press(message)
}

pub(super) fn option_button_optional(
    theme: style::ThemeTokens,
    selected: bool,
    label: &'static str,
    message: Option<UiMessage>,
) -> iced::widget::Button<'static, UiMessage> {
    let btn = button(
        text(label)
            .size(theme.size(12))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| style::text_style_primary(theme)),
    )
    .style(move |_, status| style::button_style(theme, style::ButtonKind::Tab { selected }, status))
    .padding([spacing::MD, spacing::XL]);
    if let Some(message) = message {
        btn.on_press(message)
    } else {
        btn
    }
}

pub(super) fn toggle_row(
    theme: style::ThemeTokens,
    strings: &'static UiStrings,
    enabled: bool,
    on_message: UiMessage,
    off_message: UiMessage,
) -> iced::widget::Row<'static, UiMessage> {
    row![
        option_button(theme, enabled, strings.enabled, on_message),
        option_button(theme, !enabled, strings.disabled, off_message),
    ]
    .spacing(spacing::LG)
}

pub(super) fn toggle_row_with_support(
    theme: style::ThemeTokens,
    strings: &'static UiStrings,
    enabled: bool,
    supported: bool,
    on_message: UiMessage,
    off_message: UiMessage,
) -> iced::widget::Row<'static, UiMessage> {
    let on_press = supported.then_some(on_message);
    let off_press = supported.then_some(off_message);
    row![
        option_button_optional(theme, enabled, strings.enabled, on_press),
        option_button_optional(theme, !enabled, strings.disabled, off_press),
    ]
    .spacing(spacing::LG)
}

pub(super) fn controls<'a>(content: Element<'a, UiMessage>) -> Element<'a, UiMessage> {
    container(content)
        .width(Length::FillPortion(2))
        .center_x(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 24.0,
        })
        .into()
}

pub(super) fn action_button(
    theme: style::ThemeTokens,
    label: &'static str,
    message: UiMessage,
) -> iced::widget::Button<'static, UiMessage> {
    button(
        text(label)
            .size(theme.size(12))
            .font(style::font_propo(Weight::Medium))
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
