use iced::font::{Family, Weight};
use iced::widget::{button, container, progress_bar, text, text_input};
use iced::{Background, Border, Color, Font, Shadow};

use crate::config::{AccentColor, ThemeMode, UserSettings};

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

pub fn accent_color_value(accent: AccentColor) -> Color {
    match accent {
        AccentColor::Red => Color::from_rgb8(0xef, 0x4b, 0x5f),
        AccentColor::Orange => Color::from_rgb8(0xf2, 0x87, 0x4b),
        AccentColor::Yellow => Color::from_rgb8(0xf2, 0xc9, 0x4c),
        AccentColor::Blue => Color::from_rgb8(0x3d, 0x7c, 0xff),
        AccentColor::Indigo => Color::from_rgb8(0x63, 0x66, 0xf1),
        AccentColor::Violet => Color::from_rgb8(0xa0, 0x6c, 0xff),
        AccentColor::Green => Color::from_rgb8(0x2f, 0xd0, 0x8c),
        AccentColor::Amber => Color::from_rgb8(0xf2, 0xb3, 0x47),
    }
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
    fn with_high_contrast(self) -> Self {
        let background = self.background;
        let accent = self.accent;
        let text_primary = if is_dark(background) {
            Color::from_rgb8(0xf8, 0xf8, 0xf8)
        } else {
            Color::from_rgb8(0x10, 0x10, 0x10)
        };
        let text_muted = mix(text_primary, background, 0.35);
        let border = mix(text_primary, background, 0.2);
        let selected = mix(accent, background, 0.2);
        let hover = mix(accent, background, 0.35);
        Self {
            background,
            panel: mix(self.panel, background, 0.1),
            elevated: mix(self.elevated, background, 0.1),
            hover,
            selected,
            accent,
            text_primary,
            text_muted,
            border,
            border_subtle: border,
            avatar: self.avatar,
            player_bar: mix(self.player_bar, background, 0.1),
            album_cover: self.album_cover,
            input_background: mix(self.input_background, background, 0.1),
            input_border: border,
        }
    }

    /// Derive Grape's fifteen roles from the shared 38-field palette.
    ///
    /// Grape's vocabulary is a music player's — an avatar, a player bar, an
    /// album cover — and the shared palette has no such fields. Each one maps
    /// onto the shared role that means the same thing rather than getting a
    /// colour of its own, so a theme stays coherent across every Colony program
    /// without the shared palette growing app-specific entries.
    pub fn from_shared(p: colony_ui::ThemePalette) -> Self {
        Self {
            background: p.bg_primary,
            panel: p.bg_sidebar,
            elevated: p.bg_card,
            hover: p.bg_card_hover,
            selected: p.bg_selected,
            // The user's accent override, or the theme's own when it is "auto".
            accent: colony_ui::effective_accent(),
            text_primary: p.text_primary,
            text_muted: p.text_muted,
            border: p.border_subtle,
            border_subtle: p.divider,
            avatar: p.bg_card_pressed,
            player_bar: p.bg_sidebar,
            album_cover: p.bg_card,
            input_background: p.bg_input,
            input_border: p.border_subtle,
        }
    }
}

fn is_dark(color: Color) -> bool {
    let luminance = 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
    luminance < 0.5
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

#[derive(Debug, Clone, Copy)]
pub struct ThemeTokens {
    pub palette: Palette,
    pub scale: f32,
    pub accessible_scale: f32,
    pub focus_ring: bool,
    pub reduce_transparency: bool,
}

impl ThemeTokens {
    pub fn new(
        mode: ThemeMode,
        scale: f32,
        accessible_scale: f32,
        high_contrast: bool,
        focus_ring: bool,
    ) -> Self {
        let (family, variant) = mode.keys();
        let palette = Palette::from_shared(colony_ui::resolve(family, variant));
        let palette = if high_contrast {
            palette.with_high_contrast()
        } else {
            palette
        };
        Self {
            palette,
            scale,
            accessible_scale,
            focus_ring,
            reduce_transparency: false,
        }
    }

    pub fn from_settings(settings: &UserSettings) -> Self {
        let high_contrast = settings.increase_contrast || settings.accessibility_high_contrast;
        let focus_ring = settings.highlight_keyboard_focus;
        let scale = settings.text_scale.scale();
        let accessible_scale = settings.accessible_text_size.scale();
        let mut tokens =
            Self::new(settings.theme_mode, scale, accessible_scale, high_contrast, focus_ring);
        tokens.reduce_transparency = settings.reduce_transparency;
        if !settings.accent_auto {
            tokens.palette.accent = accent_color_value(settings.accent_color);
        }
        tokens
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
            text_color: if selected { palette.accent } else { palette.text_muted },
            border: Border {
                radius: 8.0.into(),
                width: if selected { 1.0 } else { 0.0 },
                color: if selected { palette.accent } else { Color::TRANSPARENT },
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
            border: Border { color: accent(theme), ..base.border },
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

#[cfg(test)]
mod shared_palette_tests {
    use super::*;
    use crate::config::ThemeMode;

    /// Every theme Grape now offers is a real catalog entry, and there are far
    /// more than the eleven it used to hardcode.
    #[test]
    fn the_picker_offers_the_whole_shared_catalog() {
        let mut count = 0;
        for family in colony_ui::THEME_FAMILIES {
            for variant in family.variants {
                let mode =
                    ThemeMode::lookup(family.key, variant.key).expect("a catalog entry resolves");
                assert_eq!(mode.keys(), (family.key, variant.key));
                count += 1;
            }
        }
        assert_eq!(count, 57, "Grape used to offer 11 themes");
    }

    /// The eleven names Grape wrote into preferences.json before the move must
    /// still load, or every existing user silently loses their theme.
    #[test]
    fn legacy_preference_values_still_deserialize() {
        for (stored, want) in [
            ("\"Latte\"", ("catppuccin", "latte")),
            ("\"Mocha\"", ("catppuccin", "mocha")),
            ("\"Dark\"", ("catppuccin", "mocha")),
            ("\"GruvboxDark\"", ("gruvbox", "dark")),
            ("\"KanagawaJournal\"", ("kanagawa", "journal")),
            ("\"catppuccin/frappe\"", ("catppuccin", "frappe")),
        ] {
            let mode: ThemeMode = serde_json::from_str(stored).expect(stored);
            assert_eq!(mode.keys(), want, "{stored}");
        }
    }

    #[test]
    fn an_unknown_stored_theme_falls_back_instead_of_failing() {
        let mode: ThemeMode = serde_json::from_str("\"no_such/theme\"").expect("never errors");
        assert_eq!(mode, ThemeMode::default());
    }

    #[test]
    fn light_and_dark_counterparts_stay_in_the_family() {
        let latte = ThemeMode::lookup("catppuccin", "latte").unwrap();
        assert_eq!(latte.dark_variant().family, "catppuccin");
        assert!(latte.dark_variant().is_dark());
        assert!(!latte.dark_variant().light_variant().is_dark());

        // A single-variant family has no counterpart and must stay itself.
        let synth = ThemeMode::lookup("synthwave", "dark").unwrap();
        assert_eq!(synth.light_variant(), synth);
    }

    /// The palettes were byte-identical copies of colony-ui's before the move,
    /// so the shared values must still be what Grape renders.
    #[test]
    fn catppuccin_latte_still_has_its_own_colours() {
        let p = Palette::from_shared(colony_ui::resolve("catppuccin", "latte"));
        assert_eq!(p.background, colony_ui::hex(0xeff1f5));
        assert_eq!(p.text_primary, colony_ui::hex(0x4c4f69));
        assert_eq!(p.text_muted, colony_ui::hex(0x6c6f85));
    }

    #[test]
    fn a_light_theme_stays_light_and_a_dark_one_dark() {
        let latte = Palette::from_shared(colony_ui::resolve("catppuccin", "latte"));
        let mocha = Palette::from_shared(colony_ui::resolve("catppuccin", "mocha"));
        assert!(!is_dark(latte.background));
        assert!(is_dark(mocha.background));
    }
}
