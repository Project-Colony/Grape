use iced::font::{Family, Weight};
use iced::widget::{button, container, progress_bar, text, text_input};
use iced::{Background, Border, Color, Font, Shadow};

use crate::config::{AccentColor, UserSettings};

pub const FONT_PROPO: &str = "JetBrainsMono Nerd Font Propo";
pub const FONT_MONO: &str = "JetBrainsMono Nerd Font Mono";

pub fn font_propo(weight: Weight) -> Font {
    Font {
        family: Family::Name(FONT_PROPO),
        weight,
        ..Font::DEFAULT
    }
}

pub fn font_mono(weight: Weight) -> Font {
    Font {
        family: Family::Name(FONT_MONO),
        weight,
        ..Font::DEFAULT
    }
}

/// The user's accent override, resolved from Colony's shared list.
///
/// Falls back to the active palette's own accent if a key ever goes missing
/// upstream, rather than drawing an unstyled control.
pub fn accent_color_value(accent: AccentColor) -> Color {
    colony_ui::accent_key_to_color(accent.colony_key())
        .unwrap_or_else(colony_ui::effective_accent)
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub panel: Color,
    pub elevated: Color,
    pub hover: Color,
    pub selected: Color,
    pub accent: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub border: Color,
    pub border_subtle: Color,
    pub avatar: Color,
    pub player_bar: Color,
    pub album_cover: Color,
    pub input_background: Color,
    pub input_border: Color,
}

impl Palette {
    /// Grape's fifteen drawing roles, derived from Colony's shared 38-field
    /// palette. No hex lives in this file: a colour is written down once, in
    /// Project-Colony-Resources' `tokens/`, and reaches every program from
    /// there. A theme family added upstream needs no edit here.
    ///
    /// The mapping is by ROLE, not by hex. Where Grape's old value happened to
    /// coincide with a Colony field doing a different job, the role wins.
    fn from_colony(p: &colony_ui::ThemePalette) -> Self {
        Self {
            background: p.bg_primary,
            panel: p.bg_sidebar,
            elevated: p.bg_card_hover,
            hover: p.bg_card_pressed,
            selected: p.bg_selected,
            // Through the accessor, so a user accent override applies.
            accent: colony_ui::effective_accent(),
            text_primary: p.text_primary,
            text_muted: p.text_muted,
            border: p.border_subtle,
            border_subtle: p.divider,
            // Three surfaces Colony's own programs do not draw. Each takes the
            // shared field playing the same role rather than introducing a
            // program-specific palette: avatars and album covers are raised
            // cards, and the player bar is edge chrome, which is bg_sidebar's
            // job.
            avatar: p.bg_card_hover,
            album_cover: p.bg_card_hover,
            player_bar: p.bg_sidebar,
            input_background: p.bg_input,
            input_border: p.border_subtle,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeTokens {
    pub palette: Palette,
    pub scale: f32,
    pub accessible_scale: f32,
    pub focus_ring: bool,
    pub reduce_transparency: bool,
}

impl ThemeTokens {
    /// Publishes the user's choices to the shared theme state, then reads the
    /// palette back. High contrast is applied by `active_palette` itself --
    /// Colony derives it rather than shipping a second palette per theme.
    pub fn from_settings(settings: &UserSettings) -> Self {
        colony_ui::set_active_theme(&settings.theme_family, &settings.theme_variant);
        colony_ui::set_high_contrast(
            settings.increase_contrast || settings.accessibility_high_contrast,
        );
        colony_ui::set_active_accent(if settings.accent_auto {
            None
        } else {
            Some(accent_color_value(settings.accent_color))
        });

        Self {
            palette: Palette::from_colony(&colony_ui::active_palette()),
            scale: settings.text_scale.scale(),
            accessible_scale: settings.accessible_text_size.scale(),
            focus_ring: settings.highlight_keyboard_focus,
            reduce_transparency: settings.reduce_transparency,
        }
    }

    pub fn size(&self, base: u16) -> u32 {
        ((base as f32 * self.scale).round().max(10.0)) as u32
    }

    pub fn size_accessible(&self, base: u16) -> u32 {
        ((base as f32 * self.accessible_scale).round().max(10.0)) as u32
    }
}

pub fn accent(theme: ThemeTokens) -> Color {
    theme.palette.accent
}

pub fn text_primary(theme: ThemeTokens) -> Color {
    theme.palette.text_primary
}

pub fn text_muted(theme: ThemeTokens) -> Color {
    theme.palette.text_muted
}

pub fn text_style_primary(theme: ThemeTokens) -> text::Style {
    text::Style {
        color: Some(text_primary(theme)),
        ..text::Style::default()
    }
}

pub fn text_style_muted(theme: ThemeTokens) -> text::Style {
    text::Style {
        color: Some(text_muted(theme)),
        ..text::Style::default()
    }
}

fn mix(foreground: Color, background: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);
    Color {
        r: background.r + (foreground.r - background.r) * factor,
        g: background.g + (foreground.g - background.g) * factor,
        b: background.b + (foreground.b - background.b) * factor,
        a: 1.0,
    }
}

pub fn text_style(color: Color) -> text::Style {
    text::Style {
        color: Some(color),
        ..text::Style::default()
    }
}

pub fn accent_alpha(theme: ThemeTokens, alpha: f32) -> Color {
    Color {
        a: if theme.reduce_transparency { 1.0 } else { alpha },
        ..theme.palette.accent
    }
}

pub fn progress_bar_style(theme: ThemeTokens) -> progress_bar::Style {
    let palette = theme.palette;
    progress_bar::Style {
        background: Background::Color(mix(palette.text_primary, palette.background, 0.08)),
        bar: Background::Color(palette.accent),
        border: Border {
            radius: 6.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Surface {
    AppBackground,
    TopBar,
    Panel,
    Sidebar,
    PlayerBar,
    AlbumCover,
    Avatar,
}

pub fn surface_style(theme: ThemeTokens, surface: Surface) -> container::Style {
    let palette = theme.palette;
    let (background, border) = match surface {
        Surface::AppBackground => (
            palette.background,
            Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        ),
        Surface::TopBar => (
            palette.elevated,
            Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        ),
        Surface::Panel => (
            palette.panel,
            Border {
                radius: 12.0.into(),
                width: 1.0,
                color: palette.border,
            },
        ),
        Surface::Sidebar => (
            palette.elevated,
            Border {
                radius: 12.0.into(),
                width: 1.0,
                color: palette.border_subtle,
            },
        ),
        Surface::PlayerBar => (
            palette.player_bar,
            Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        ),
        Surface::AlbumCover => (
            palette.album_cover,
            Border {
                radius: 8.0.into(),
                width: 1.0,
                color: palette.border_subtle,
            },
        ),
        Surface::Avatar => (
            palette.avatar,
            Border {
                radius: 999.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        ),
    };

    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(text_primary(theme)),
        border,
        shadow: Shadow::default(),
        snap: cfg!(feature = "crisp"),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonKind {
    Tab { selected: bool },
    ListItem { selected: bool, focused: bool },
    AlbumCard { selected: bool, focused: bool },
    Control,
    Icon,
}

pub fn button_style(theme: ThemeTokens, kind: ButtonKind, status: button::Status) -> button::Style {
    let palette = theme.palette;
    let mut style = match kind {
        ButtonKind::Tab { selected } => button::Style {
            background: Some(Background::Color(if selected {
                palette.hover
            } else {
                Color::TRANSPARENT
            })),
            text_color: if selected {
                palette.accent
            } else {
                palette.text_muted
            },
            border: Border {
                radius: 8.0.into(),
                width: if selected { 1.0 } else { 0.0 },
                color: if selected {
                    palette.accent
                } else {
                    Color::TRANSPARENT
                },
            },
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
        ButtonKind::ListItem { selected, focused } => button::Style {
            background: Some(Background::Color(if selected {
                palette.selected
            } else {
                Color::TRANSPARENT
            })),
            text_color: palette.text_primary,
            border: Border {
                radius: 10.0.into(),
                width: if selected || focused { 1.0 } else { 0.0 },
                color: if selected || focused {
                    palette.accent
                } else {
                    Color::TRANSPARENT
                },
            },
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
        ButtonKind::AlbumCard { selected, focused } => button::Style {
            background: Some(Background::Color(if selected {
                palette.selected
            } else {
                Color::TRANSPARENT
            })),
            text_color: palette.text_primary,
            border: Border {
                radius: 12.0.into(),
                width: if selected || focused { 1.0 } else { 0.0 },
                color: if selected || focused {
                    palette.accent
                } else {
                    Color::TRANSPARENT
                },
            },
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
        ButtonKind::Control => button::Style {
            background: Some(Background::Color(palette.elevated)),
            text_color: palette.text_primary,
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: palette.border_subtle,
            },
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
        ButtonKind::Icon => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: palette.text_muted,
            border: Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
    };

    if matches!(
        kind,
        ButtonKind::ListItem { focused: true, .. } | ButtonKind::AlbumCard { focused: true, .. }
    ) && theme.focus_ring
    {
        style.border = Border {
            width: 2.0,
            color: palette.accent,
            ..style.border
        };
    }

    match status {
        button::Status::Hovered | button::Status::Pressed => {
            style.background = Some(Background::Color(palette.hover));
        }
        button::Status::Disabled => {
            style.background = Some(Background::Color(palette.elevated));
            style.text_color = palette.text_muted;
            style.border.color = palette.border_subtle;
        }
        button::Status::Active => {}
    }

    style
}

pub fn text_input_style(theme: ThemeTokens, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Background::Color(theme.palette.input_background),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: theme.palette.input_border,
        },
        icon: text_muted(theme),
        placeholder: text_muted(theme),
        value: text_primary(theme),
        selection: accent_alpha(theme, 0.25),
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: theme.palette.border,
                ..base.border
            },
            ..base
        },
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: accent(theme),
                ..base.border
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(theme.palette.elevated),
            border: Border {
                color: theme.palette.border,
                ..base.border
            },
            value: text_muted(theme),
            ..base
        },
    }
}
