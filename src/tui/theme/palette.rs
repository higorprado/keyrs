use ratatui::prelude::*;

/// Catppuccin Mocha inspired color palette for the TUI
pub struct Palette {
    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    // Accent colors
    pub accent_primary: Color,
    pub accent_success: Color,
    pub accent_warning: Color,
    pub accent_danger: Color,

    // UI colors
    pub border_default: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,

    // Background colors for status
    pub bg_status_active: Color,
    pub bg_status_inactive: Color,
    pub bg_status_unknown: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            // Text - Catppuccin text shades
            text_primary: Color::Rgb(205, 214, 244),      // Text
            text_secondary: Color::Rgb(186, 194, 222),    // Subtext1
            text_muted: Color::Rgb(108, 112, 134),        // Overlay0

            // Accents
            accent_primary: Color::Rgb(137, 180, 250),    // Blue
            accent_success: Color::Rgb(166, 227, 161),    // Green
            accent_warning: Color::Rgb(249, 226, 175),    // Yellow
            accent_danger: Color::Rgb(243, 139, 168),     // Red

            // UI
            border_default: Color::Rgb(69, 71, 90),       // Surface1
            border_focused: Color::Rgb(137, 180, 250),    // Blue
            selection_bg: Color::Rgb(137, 180, 250),      // Blue
            selection_fg: Color::Rgb(30, 30, 46),         // Crust

            // Backgrounds
            bg_status_active: Color::Rgb(166, 227, 161),  // Green
            bg_status_inactive: Color::Rgb(243, 139, 168), // Red
            bg_status_unknown: Color::Rgb(249, 226, 175), // Yellow
        }
    }
}

impl Palette {
    pub fn new() -> Self {
        Self::default()
    }
}
