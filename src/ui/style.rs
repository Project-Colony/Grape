use iced::widget::{button, container, progress_bar, text, text_input};
use iced::{Background, Border, Color, Shadow};

use crate::config::UserSettings;

/// Every Nerd Font glyph Grape draws, written as a codepoint rather than
/// pasted (design/typography.md): a pasted glyph does not survive every
/// editor, terminal and diff tool intact.
///
/// Draw them in [`colony_ui::fonts::glyph_font`], never in the interface font:
/// OpenDyslexic has none of these, so a glyph set in `ui_font()` would turn
/// into a tofu box the moment the dyslexia font is on. A test holds every one
/// of them to the Nerd Font file colony-ui embeds.
pub mod icons {
    pub use colony_ui::widgets::icons::CHECK;

    pub const WINDOW_MINIMIZE: &str = "\u{f05b0}";
    pub const WINDOW_MAXIMIZE: &str = "\u{f05af}";
    pub const WINDOW_CLOSE: &str = "\u{f05ad}";
    /// Closes a page or removes a row.
    pub const CLOSE: &str = "\u{f0156}";
    pub const MOVE_UP: &str = "\u{f005d}";
    pub const MOVE_DOWN: &str = "\u{f0045}";
    pub const DRAG: &str = "\u{f01dd}";
    pub const DROP_HERE: &str = "\u{f0120}";
    /// Stands in for a missing album cover.
    pub const ALBUM: &str = "\u{f0387}";
    /// Stands in for a missing folder cover.
    pub const FOLDER: &str = "\u{f024b}";
    /// Stands in for the cover of the track playing.
    pub const NOW_PLAYING: &str = "\u{f0386}";
    pub const PREVIOUS: &str = "\u{f04ae}";
    pub const NEXT: &str = "\u{f04ad}";
    pub const PLAY: &str = "\u{f040a}";
    pub const PAUSE: &str = "\u{f03e4}";
    pub const SHUFFLE_ON: &str = "\u{f049d}";
    pub const SHUFFLE_OFF: &str = "\u{f049e}";
    pub const REPEAT_ALL: &str = "\u{f0456}";
    pub const REPEAT_ONE: &str = "\u{f0458}";
    pub const REPEAT_OFF: &str = "\u{f0457}";
    pub const VOLUME_OFF: &str = "\u{f0581}";
    pub const VOLUME_LOW: &str = "\u{f057f}";
    pub const VOLUME_MEDIUM: &str = "\u{f0580}";
    pub const VOLUME_HIGH: &str = "\u{f057e}";
    pub const QUEUE_ON: &str = "\u{f0cb8}";
    pub const QUEUE_OFF: &str = "\u{f0cb9}";
    pub const MINI_PLAYER_OPEN: &str = "\u{f0063}";
    pub const MINI_PLAYER_CLOSE: &str = "\u{f0062}";

    #[cfg(test)]
    pub const ALL: &[(&str, &str)] = &[
        ("CHECK", CHECK),
        ("WINDOW_MINIMIZE", WINDOW_MINIMIZE),
        ("WINDOW_MAXIMIZE", WINDOW_MAXIMIZE),
        ("WINDOW_CLOSE", WINDOW_CLOSE),
        ("CLOSE", CLOSE),
        ("MOVE_UP", MOVE_UP),
        ("MOVE_DOWN", MOVE_DOWN),
        ("DRAG", DRAG),
        ("DROP_HERE", DROP_HERE),
        ("ALBUM", ALBUM),
        ("FOLDER", FOLDER),
        ("NOW_PLAYING", NOW_PLAYING),
        ("PREVIOUS", PREVIOUS),
        ("NEXT", NEXT),
        ("PLAY", PLAY),
        ("PAUSE", PAUSE),
        ("SHUFFLE_ON", SHUFFLE_ON),
        ("SHUFFLE_OFF", SHUFFLE_OFF),
        ("REPEAT_ALL", REPEAT_ALL),
        ("REPEAT_ONE", REPEAT_ONE),
        ("REPEAT_OFF", REPEAT_OFF),
        ("VOLUME_OFF", VOLUME_OFF),
        ("VOLUME_LOW", VOLUME_LOW),
        ("VOLUME_MEDIUM", VOLUME_MEDIUM),
        ("VOLUME_HIGH", VOLUME_HIGH),
        ("QUEUE_ON", QUEUE_ON),
        ("QUEUE_OFF", QUEUE_OFF),
        ("MINI_PLAYER_OPEN", MINI_PLAYER_OPEN),
        ("MINI_PLAYER_CLOSE", MINI_PLAYER_CLOSE),
    ];
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
    pub focus_ring: bool,
    pub reduce_transparency: bool,
}

impl ThemeTokens {
    /// Publishes the user's choices to the shared Colony state, then reads the
    /// palette back. `apply` sets the theme, the accent, high contrast, the
    /// dyslexia font, reduced motion and both text sizes, which is what
    /// `colony_ui::sz` and the font accessors read. High contrast is applied by
    /// `active_palette` itself: Colony derives it rather than shipping a second
    /// palette per theme. The locale goes with them, so the shared widgets'
    /// labels follow Grape's interface language.
    pub fn from_settings(settings: &UserSettings) -> Self {
        colony_ui::i18n::set_locale(settings.interface_language.locale());
        settings.standard.apply();

        Self {
            palette: Palette::from_colony(&colony_ui::active_palette()),
            focus_ring: settings.highlight_keyboard_focus,
            reduce_transparency: settings.reduce_transparency,
        }
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

/// A label inside a selectable item ([`ButtonKind::ListItem`],
/// [`ButtonKind::Tab`]): `contrast_on(accent)` while the item sits on the
/// accent, `idle` otherwise. `text_primary` on an accent fill is below 4.5:1
/// on almost every theme and 1.01:1 on Ayu Dark (design/navigation.md).
pub fn selectable_text(
    theme: ThemeTokens,
    selected: bool,
    idle: fn(ThemeTokens) -> text::Style,
) -> text::Style {
    if selected {
        text::Style {
            color: Some(colony_ui::contrast_on(theme.palette.accent)),
        }
    } else {
        idle(theme)
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
    /// The logo button, Grape's identity element. It sits on `bg_selected`
    /// while the menu or the Preferences it opened are showing.
    Identity { open: bool },
}

pub fn button_style(theme: ThemeTokens, kind: ButtonKind, status: button::Status) -> button::Style {
    let palette = theme.palette;
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let mut style = match kind {
        // design/navigation.md "Selection state", what
        // colony_ui::widgets::selection_style draws for the shared lists: the
        // selected item on the accent with its text in contrast_on(accent), a
        // hovered one on bg_card_hover, the rest on nothing in text_muted.
        // Hover never covers the selection. Labels that set their own colour
        // go through `selectable_text`.
        ButtonKind::Tab { selected } | ButtonKind::ListItem { selected, .. } => {
            let (background, text_color) = if selected {
                (palette.accent, colony_ui::contrast_on(palette.accent))
            } else if hovered {
                (palette.elevated, palette.text_muted)
            } else {
                (Color::TRANSPARENT, palette.text_muted)
            };
            button::Style {
                background: Some(Background::Color(background)),
                text_color,
                border: Border {
                    radius: 8.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: cfg!(feature = "crisp"),
            }
        }
        ButtonKind::Identity { open } => button::Style {
            background: Some(Background::Color(if hovered {
                palette.elevated
            } else if open {
                palette.selected
            } else {
                Color::TRANSPARENT
            })),
            text_color: palette.text_primary,
            border: Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
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

    // The keyboard focus ring. On the accent fill of a selected item an
    // accent ring would vanish, so it takes the fill's contrast colour.
    if let ButtonKind::ListItem { selected, focused: true } = kind {
        style.border = Border {
            width: if theme.focus_ring { 2.0 } else { 1.0 },
            color: if selected {
                colony_ui::contrast_on(palette.accent)
            } else {
                palette.accent
            },
            ..style.border
        };
    }
    if matches!(kind, ButtonKind::AlbumCard { focused: true, .. }) && theme.focus_ring {
        style.border = Border {
            width: 2.0,
            color: palette.accent,
            ..style.border
        };
    }

    let own_hover = matches!(
        kind,
        ButtonKind::Tab { .. } | ButtonKind::ListItem { .. } | ButtonKind::Identity { .. }
    );
    match status {
        button::Status::Hovered | button::Status::Pressed if !own_hover => {
            style.background = Some(Background::Color(palette.hover));
        }
        button::Status::Hovered | button::Status::Pressed => {}
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

#[cfg(test)]
mod tests {
    use colony_ui::fonts::{APP_FAMILY, BYTES};
    // The font reader iced's own text stack uses, so the check needs no
    // dependency of its own.
    use iced::advanced::graphics::text::cosmic_text::skrifa::{FontRef, MetadataProvider};

    /// design/typography.md: a codepoint JetBrainsMono Nerd Font does not
    /// carry is a tofu box on every user's machine, so each one is checked
    /// against the file colony-ui embeds rather than against whatever fonts the
    /// developer happens to have installed.
    #[test]
    fn every_glyph_grape_draws_is_in_the_nerd_font() {
        let charmap = FontRef::new(BYTES[0]).expect("the Regular file parses").charmap();
        for (name, glyph) in super::icons::ALL {
            for c in glyph.chars() {
                assert!(
                    charmap.map(c).is_some(),
                    "{name}: U+{:04X} is not in {APP_FAMILY}",
                    c as u32
                );
            }
        }
    }
}
