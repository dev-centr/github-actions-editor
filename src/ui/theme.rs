//! Application theming
//!
//! Custom theme with GitHub-inspired colors.

use iced::color;
use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Theme};

/// GitHub-inspired color palette
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub text: Color,
    pub text_muted: Color,
    pub border: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            background: color!(0x0d1117),      // GitHub dark bg
            surface: color!(0x161b22),          // Elevated surface
            surface_hover: color!(0x21262d),    // Hover state
            primary: color!(0x238636),          // GitHub green
            primary_hover: color!(0x2ea043),    // Green hover
            success: color!(0x238636),          // Success green
            warning: color!(0xd29922),          // Warning yellow
            danger: color!(0xda3633),           // Danger red
            text: color!(0xe6edf3),             // Primary text
            text_muted: color!(0x7d8590),       // Muted text
            border: color!(0x30363d),           // Border color
        }
    }
}

/// Custom button styles
pub mod button_style {
    use super::*;

    pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
        let palette = Palette::default();
        
        match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(palette.primary)),
                text_color: Color::WHITE,
                border: Border {
                    color: palette.primary,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(palette.primary_hover)),
                text_color: Color::WHITE,
                border: Border {
                    color: palette.primary_hover,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(palette.primary)),
                text_color: Color::WHITE,
                border: Border {
                    color: palette.primary,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(palette.surface)),
                text_color: palette.text_muted,
                border: Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
        }
    }

    pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
        let palette = Palette::default();
        
        match status {
            button::Status::Active => button::Style {
                background: Some(Background::Color(palette.surface)),
                text_color: palette.text,
                border: Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(palette.surface_hover)),
                text_color: palette.text,
                border: Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            _ => button::Style::default(),
        }
    }

    pub fn danger(theme: &Theme, status: button::Status) -> button::Style {
        let palette = Palette::default();
        
        match status {
            button::Status::Active | button::Status::Hovered => button::Style {
                background: Some(Background::Color(palette.danger)),
                text_color: Color::WHITE,
                border: Border {
                    color: palette.danger,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            _ => button::Style::default(),
        }
    }
}

/// Custom container styles
pub mod container_style {
    use super::*;

    pub fn card(theme: &Theme) -> container::Style {
        let palette = Palette::default();
        container::Style {
            background: Some(Background::Color(palette.surface)),
            border: Border {
                color: palette.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }

    pub fn sidebar(theme: &Theme) -> container::Style {
        let palette = Palette::default();
        container::Style {
            background: Some(Background::Color(palette.background)),
            border: Border {
                color: palette.border,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Custom text input styles
pub mod input_style {
    use super::*;

    pub fn default(theme: &Theme, status: text_input::Status) -> text_input::Style {
        let palette = Palette::default();
        
        text_input::Style {
            background: Background::Color(palette.background),
            border: Border {
                color: palette.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            icon: palette.text_muted,
            placeholder: palette.text_muted,
            value: palette.text,
            selection: palette.primary,
        }
    }
}

