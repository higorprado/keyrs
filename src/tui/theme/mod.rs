mod palette;

use ratatui::prelude::*;

pub use palette::Palette;

pub struct Theme {
    pub palette: Palette,
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

impl Theme {
    pub fn new() -> Self {
        Self {
            palette: Palette::new(),
        }
    }

    // Panel border style
    pub fn panel_border(&self, focused: bool) -> Style {
        if focused {
            Style::default()
                .fg(self.palette.border_focused)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.palette.border_default)
        }
    }

    // Status badge style based on service state
    pub fn status_badge(&self, state: &str) -> Style {
        match state {
            "active" => Style::default()
                .fg(Color::Black)
                .bg(self.palette.bg_status_active)
                .add_modifier(Modifier::BOLD),
            "inactive" | "failed" => Style::default()
                .fg(Color::Black)
                .bg(self.palette.bg_status_inactive)
                .add_modifier(Modifier::BOLD),
            _ => Style::default()
                .fg(Color::Black)
                .bg(self.palette.bg_status_unknown)
                .add_modifier(Modifier::BOLD),
        }
    }

    // Checkbox style
    pub fn checkbox(&self, checked: bool) -> Style {
        if checked {
            Style::default().fg(self.palette.accent_success)
        } else {
            Style::default().fg(self.palette.text_muted)
        }
    }

    // Primary text
    pub fn text_primary(&self) -> Style {
        Style::default().fg(self.palette.text_primary)
    }

    // Secondary/dimmed text
    pub fn text_secondary(&self) -> Style {
        Style::default().fg(self.palette.text_secondary)
    }

    // Muted text (hints, labels)
    pub fn text_muted(&self) -> Style {
        Style::default().fg(self.palette.text_muted)
    }

    // Title style for focused panel
    pub fn title_focused(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_primary)
            .add_modifier(Modifier::BOLD)
    }

    // Title style for unfocused panel
    pub fn title_unfocused(&self) -> Style {
        Style::default().fg(self.palette.text_muted)
    }

    // Output text style for commands
    pub fn output_command(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_primary)
            .add_modifier(Modifier::BOLD)
    }

    // Output text style for regular text
    pub fn output_text(&self) -> Style {
        Style::default().fg(self.palette.text_primary)
    }

    // Key hint style (the key part like "Tab")
    pub fn key_hint(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_primary)
            .add_modifier(Modifier::BOLD)
    }

    // Confirm prompt style
    pub fn confirm_prompt(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_danger)
            .add_modifier(Modifier::BOLD)
    }

    // Status message style
    pub fn status_message(&self) -> Style {
        Style::default().fg(self.palette.accent_warning)
    }

    // Setting key style
    pub fn setting_key(&self) -> Style {
        Style::default().fg(self.palette.text_secondary)
    }

    // Setting value style
    pub fn setting_value(&self, enabled: bool) -> Style {
        if enabled {
            Style::default().fg(self.palette.accent_success)
        } else {
            Style::default().fg(self.palette.text_muted)
        }
    }

    // Panel title with optional focus indicator
    pub fn panel_title(&self, title: &str, focused: bool) -> Line<'_> {
        if focused {
            Line::styled(format!(" {} ", title), self.title_focused())
        } else {
            Line::styled(format!(" {} ", title), self.title_unfocused())
        }
    }
}

/// Global theme instance
pub static THEME: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();

pub fn theme() -> &'static Theme {
    THEME.get_or_init(Theme::new)
}
