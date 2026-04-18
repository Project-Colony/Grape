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

    pub fn latte() -> Self {
        Self {
            background: Color::from_rgb8(0xef, 0xf1, 0xf5),
            panel: Color::from_rgb8(0xe6, 0xe9, 0xef),
            elevated: Color::from_rgb8(0xcc, 0xd0, 0xda),
            hover: Color::from_rgb8(0xbc, 0xc0, 0xcc),
            selected: Color::from_rgb8(0x72, 0x87, 0xfd),
            accent: Color::from_rgb8(0x1e, 0x66, 0xf5),
            text_primary: Color::from_rgb8(0x4c, 0x4f, 0x69),
            text_muted: Color::from_rgb8(0x6c, 0x6f, 0x85),
            border: Color::from_rgb8(0xac, 0xb0, 0xbe),
            border_subtle: Color::from_rgb8(0xbc, 0xc0, 0xcc),
            avatar: Color::from_rgb8(0xcc, 0xd0, 0xda),
            player_bar: Color::from_rgb8(0xdc, 0xe0, 0xe8),
            album_cover: Color::from_rgb8(0xcc, 0xd0, 0xda),
            input_background: Color::from_rgb8(0xe6, 0xe9, 0xef),
            input_border: Color::from_rgb8(0xac, 0xb0, 0xbe),
        }
    }

    pub fn frappe() -> Self {
        Self {
            background: Color::from_rgb8(0x30, 0x34, 0x46),
            panel: Color::from_rgb8(0x29, 0x2c, 0x3c),
            elevated: Color::from_rgb8(0x41, 0x45, 0x59),
            hover: Color::from_rgb8(0x51, 0x57, 0x6d),
            selected: Color::from_rgb8(0xba, 0xbb, 0xf1),
            accent: Color::from_rgb8(0x8c, 0xaa, 0xee),
            text_primary: Color::from_rgb8(0xc6, 0xd0, 0xf5),
            text_muted: Color::from_rgb8(0xa5, 0xad, 0xce),
            border: Color::from_rgb8(0x62, 0x68, 0x80),
            border_subtle: Color::from_rgb8(0x51, 0x57, 0x6d),
            avatar: Color::from_rgb8(0x41, 0x45, 0x59),
            player_bar: Color::from_rgb8(0x23, 0x26, 0x34),
            album_cover: Color::from_rgb8(0x41, 0x45, 0x59),
            input_background: Color::from_rgb8(0x29, 0x2c, 0x3c),
            input_border: Color::from_rgb8(0x62, 0x68, 0x80),
        }
    }

    pub fn macchiato() -> Self {
        Self {
            background: Color::from_rgb8(0x24, 0x27, 0x3a),
            panel: Color::from_rgb8(0x1e, 0x20, 0x30),
            elevated: Color::from_rgb8(0x36, 0x3a, 0x4f),
            hover: Color::from_rgb8(0x49, 0x4d, 0x64),
            selected: Color::from_rgb8(0xb7, 0xbd, 0xf8),
            accent: Color::from_rgb8(0x8a, 0xad, 0xf4),
            text_primary: Color::from_rgb8(0xca, 0xd3, 0xf5),
            text_muted: Color::from_rgb8(0xa5, 0xad, 0xcb),
            border: Color::from_rgb8(0x5b, 0x60, 0x78),
            border_subtle: Color::from_rgb8(0x49, 0x4d, 0x64),
            avatar: Color::from_rgb8(0x36, 0x3a, 0x4f),
            player_bar: Color::from_rgb8(0x18, 0x19, 0x26),
            album_cover: Color::from_rgb8(0x36, 0x3a, 0x4f),
            input_background: Color::from_rgb8(0x1e, 0x20, 0x30),
            input_border: Color::from_rgb8(0x5b, 0x60, 0x78),
        }
    }

    pub fn mocha() -> Self {
        Self {
            background: Color::from_rgb8(0x1e, 0x1e, 0x2e),
            panel: Color::from_rgb8(0x18, 0x18, 0x25),
            elevated: Color::from_rgb8(0x31, 0x32, 0x44),
            hover: Color::from_rgb8(0x45, 0x47, 0x5a),
            selected: Color::from_rgb8(0xb4, 0xbe, 0xfe),
            accent: Color::from_rgb8(0x89, 0xb4, 0xfa),
            text_primary: Color::from_rgb8(0xcd, 0xd6, 0xf4),
            text_muted: Color::from_rgb8(0xa6, 0xad, 0xc8),
            border: Color::from_rgb8(0x58, 0x5b, 0x70),
            border_subtle: Color::from_rgb8(0x45, 0x47, 0x5a),
            avatar: Color::from_rgb8(0x31, 0x32, 0x44),
            player_bar: Color::from_rgb8(0x11, 0x11, 0x1b),
            album_cover: Color::from_rgb8(0x31, 0x32, 0x44),
            input_background: Color::from_rgb8(0x18, 0x18, 0x25),
            input_border: Color::from_rgb8(0x58, 0x5b, 0x70),
        }
    }

    pub fn gruvbox_light() -> Self {
        Self {
            background: Color::from_rgb8(0xfb, 0xf1, 0xc7),
            panel: Color::from_rgb8(0xf2, 0xe5, 0xbc),
            elevated: Color::from_rgb8(0xeb, 0xdb, 0xb2),
            hover: Color::from_rgb8(0xd5, 0xc4, 0xa1),
            selected: Color::from_rgb8(0xbd, 0xae, 0x93),
            accent: Color::from_rgb8(0xd6, 0x5d, 0x0e),
            text_primary: Color::from_rgb8(0x3c, 0x38, 0x36),
            text_muted: Color::from_rgb8(0x66, 0x5c, 0x54),
            border: Color::from_rgb8(0xbd, 0xae, 0x93),
            border_subtle: Color::from_rgb8(0xd5, 0xc4, 0xa1),
            avatar: Color::from_rgb8(0xeb, 0xdb, 0xb2),
            player_bar: Color::from_rgb8(0xf9, 0xf5, 0xd7),
            album_cover: Color::from_rgb8(0xeb, 0xdb, 0xb2),
            input_background: Color::from_rgb8(0xf2, 0xe5, 0xbc),
            input_border: Color::from_rgb8(0xbd, 0xae, 0x93),
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            background: Color::from_rgb8(0x28, 0x28, 0x28),
            panel: Color::from_rgb8(0x1d, 0x20, 0x21),
            elevated: Color::from_rgb8(0x3c, 0x38, 0x36),
            hover: Color::from_rgb8(0x50, 0x49, 0x45),
            selected: Color::from_rgb8(0x66, 0x5c, 0x54),
            accent: Color::from_rgb8(0xfa, 0xbd, 0x2f),
            text_primary: Color::from_rgb8(0xeb, 0xdb, 0xb2),
            text_muted: Color::from_rgb8(0xbd, 0xae, 0x93),
            border: Color::from_rgb8(0x66, 0x5c, 0x54),
            border_subtle: Color::from_rgb8(0x50, 0x49, 0x45),
            avatar: Color::from_rgb8(0x3c, 0x38, 0x36),
            player_bar: Color::from_rgb8(0x1d, 0x20, 0x21),
            album_cover: Color::from_rgb8(0x3c, 0x38, 0x36),
            input_background: Color::from_rgb8(0x1d, 0x20, 0x21),
            input_border: Color::from_rgb8(0x66, 0x5c, 0x54),
        }
    }

    pub fn everblush_light() -> Self {
        Self {
            background: Color::from_rgb8(0xf2, 0xf4, 0xf4),
            panel: Color::from_rgb8(0xe5, 0xe9, 0xe8),
            elevated: Color::from_rgb8(0xd6, 0xdc, 0xda),
            hover: Color::from_rgb8(0xc8, 0xce, 0xcc),
            selected: Color::from_rgb8(0xb3, 0xb9, 0xb8),
            accent: Color::from_rgb8(0x3f, 0x7a, 0xbf),
            text_primary: Color::from_rgb8(0x2b, 0x2f, 0x30),
            text_muted: Color::from_rgb8(0x5f, 0x66, 0x65),
            border: Color::from_rgb8(0xc0, 0xc6, 0xc5),
            border_subtle: Color::from_rgb8(0xd6, 0xdc, 0xda),
            avatar: Color::from_rgb8(0xd6, 0xdc, 0xda),
            player_bar: Color::from_rgb8(0xee, 0xf1, 0xf1),
            album_cover: Color::from_rgb8(0xd6, 0xdc, 0xda),
            input_background: Color::from_rgb8(0xe5, 0xe9, 0xe8),
            input_border: Color::from_rgb8(0xc0, 0xc6, 0xc5),
        }
    }

    pub fn everblush_dark() -> Self {
        Self {
            background: Color::from_rgb8(0x14, 0x1b, 0x1e),
            panel: Color::from_rgb8(0x1b, 0x22, 0x25),
            elevated: Color::from_rgb8(0x23, 0x2a, 0x2d),
            hover: Color::from_rgb8(0x2c, 0x33, 0x36),
            selected: Color::from_rgb8(0x39, 0x41, 0x44),
            accent: Color::from_rgb8(0x67, 0xb0, 0xe8),
            text_primary: Color::from_rgb8(0xda, 0xda, 0xda),
            text_muted: Color::from_rgb8(0xb3, 0xb9, 0xb8),
            border: Color::from_rgb8(0x2c, 0x33, 0x36),
            border_subtle: Color::from_rgb8(0x23, 0x2a, 0x2d),
            avatar: Color::from_rgb8(0x23, 0x2a, 0x2d),
            player_bar: Color::from_rgb8(0x10, 0x16, 0x18),
            album_cover: Color::from_rgb8(0x23, 0x2a, 0x2d),
            input_background: Color::from_rgb8(0x1b, 0x22, 0x25),
            input_border: Color::from_rgb8(0x2c, 0x33, 0x36),
        }
    }

    pub fn kanagawa_light() -> Self {
        Self {
            background: Color::from_rgb8(0xf2, 0xec, 0xbc),
            panel: Color::from_rgb8(0xe8, 0xdd, 0xb0),
            elevated: Color::from_rgb8(0xe1, 0xd5, 0xa3),
            hover: Color::from_rgb8(0xd7, 0xc9, 0x95),
            selected: Color::from_rgb8(0xc8, 0xb4, 0x7c),
            accent: Color::from_rgb8(0xc9, 0x7c, 0x5d),
            text_primary: Color::from_rgb8(0x4c, 0x4b, 0x4b),
            text_muted: Color::from_rgb8(0x6d, 0x6b, 0x6b),
            border: Color::from_rgb8(0xc8, 0xb4, 0x7c),
            border_subtle: Color::from_rgb8(0xd7, 0xc9, 0x95),
            avatar: Color::from_rgb8(0xe1, 0xd5, 0xa3),
            player_bar: Color::from_rgb8(0xf7, 0xf1, 0xcf),
            album_cover: Color::from_rgb8(0xe1, 0xd5, 0xa3),
            input_background: Color::from_rgb8(0xe8, 0xdd, 0xb0),
            input_border: Color::from_rgb8(0xc8, 0xb4, 0x7c),
        }
    }

    pub fn kanagawa_dark() -> Self {
        Self {
            background: Color::from_rgb8(0x1f, 0x1f, 0x28),
            panel: Color::from_rgb8(0x1a, 0x1a, 0x22),
            elevated: Color::from_rgb8(0x2a, 0x2a, 0x37),
            hover: Color::from_rgb8(0x36, 0x36, 0x46),
            selected: Color::from_rgb8(0x54, 0x54, 0x6d),
            accent: Color::from_rgb8(0x7e, 0x9c, 0xd8),
            text_primary: Color::from_rgb8(0xdc, 0xd7, 0xba),
            text_muted: Color::from_rgb8(0xa6, 0xa6, 0x9c),
            border: Color::from_rgb8(0x54, 0x54, 0x6d),
            border_subtle: Color::from_rgb8(0x36, 0x36, 0x46),
            avatar: Color::from_rgb8(0x2a, 0x2a, 0x37),
            player_bar: Color::from_rgb8(0x16, 0x16, 0x1d),
            album_cover: Color::from_rgb8(0x2a, 0x2a, 0x37),
            input_background: Color::from_rgb8(0x1a, 0x1a, 0x22),
            input_border: Color::from_rgb8(0x54, 0x54, 0x6d),
        }
    }

    pub fn kanagawa_journal() -> Self {
        Self {
            background: Color::from_rgb8(0xf8, 0xf1, 0xd6),
            panel: Color::from_rgb8(0xf0, 0xe4, 0xbf),
            elevated: Color::from_rgb8(0xe8, 0xd7, 0xaa),
            hover: Color::from_rgb8(0xdf, 0xc8, 0x90),
            selected: Color::from_rgb8(0xcb, 0xb0, 0x7a),
            accent: Color::from_rgb8(0xb4, 0x69, 0x4e),
            text_primary: Color::from_rgb8(0x5a, 0x4b, 0x3b),
            text_muted: Color::from_rgb8(0x7a, 0x6a, 0x55),
            border: Color::from_rgb8(0xcb, 0xb0, 0x7a),
            border_subtle: Color::from_rgb8(0xdf, 0xc8, 0x90),
            avatar: Color::from_rgb8(0xe8, 0xd7, 0xaa),
            player_bar: Color::from_rgb8(0xfb, 0xf6, 0xe2),
            album_cover: Color::from_rgb8(0xe8, 0xd7, 0xaa),
            input_background: Color::from_rgb8(0xf0, 0xe4, 0xbf),
            input_border: Color::from_rgb8(0xcb, 0xb0, 0x7a),
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
        let palette = match mode {
            ThemeMode::Latte => Palette::latte(),
            ThemeMode::Frappe => Palette::frappe(),
            ThemeMode::Macchiato => Palette::macchiato(),
            ThemeMode::Mocha => Palette::mocha(),
            ThemeMode::GruvboxLight => Palette::gruvbox_light(),
            ThemeMode::GruvboxDark => Palette::gruvbox_dark(),
            ThemeMode::EverblushLight => Palette::everblush_light(),
            ThemeMode::EverblushDark => Palette::everblush_dark(),
            ThemeMode::KanagawaLight => Palette::kanagawa_light(),
            ThemeMode::KanagawaDark => Palette::kanagawa_dark(),
            ThemeMode::KanagawaJournal => Palette::kanagawa_journal(),
        };
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
        let mut tokens = Self::new(
            settings.theme_mode,
            scale,
            accessible_scale,
            high_contrast,
            focus_ring,
        );
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
