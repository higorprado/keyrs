use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::tui::app::{self, App, Pane, SettingEntry};
use crate::tui::theme::theme;

const COL_WIDTH: usize = 24;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let t = theme();
    let focused = app.focused_pane == Pane::Settings;

    let block = Block::default()
        .title(t.panel_title("SETTINGS", focused))
        .borders(Borders::ALL)
        .border_style(t.panel_border(focused))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    render_settings_grid(frame, app, inner, focused);
}

fn render_settings_grid(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let t = theme();

    // Split area into sections:
    // - Row 1: Layout | Keyboard | [save hint]
    // - Row 2+: Features in aligned columns

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(4)])
        .split(area);

    // Row 1: Layout and Keyboard settings
    let layout_selected = matches!(app.selected_setting(), Some(SettingEntry::LayoutOptspec));
    let kb_selected = matches!(app.selected_setting(), Some(SettingEntry::KeyboardOverride));

    let mut row1_spans = vec![];

    // Layout
    row1_spans.push(Span::styled("Layout: ", t.text_muted()));
    row1_spans.push(Span::styled(
        format!("[{}]", app.settings.layout.optspec_layout),
        if layout_selected && focused {
            Style::default()
                .fg(t.palette.selection_fg)
                .bg(t.palette.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            t.setting_value(true)
        },
    ));
    row1_spans.push(Span::raw("   "));

    // Keyboard
    let kb_value = app.settings.keyboard.override_type.as_deref().unwrap_or("auto");
    row1_spans.push(Span::styled("Keyboard: ", t.text_muted()));
    row1_spans.push(Span::styled(
        format!("[{}]", kb_value),
        if kb_selected && focused {
            Style::default()
                .fg(t.palette.selection_fg)
                .bg(t.palette.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            t.setting_value(true)
        },
    ));
    row1_spans.push(Span::raw("   "));
    row1_spans.push(Span::styled("s:save  a:save+restart", t.key_hint()));

    let row1_para = Paragraph::new(Line::from(row1_spans));
    frame.render_widget(row1_para, chunks[0]);

    // Features section - use same ordering as app.rs
    let feature_keys = app::sorted_feature_keys(&app.settings.features);

    if feature_keys.is_empty() {
        let empty = Paragraph::new(Line::styled("No features configured", t.text_muted()));
        frame.render_widget(empty, chunks[1]);
        return;
    }

    // Calculate number of columns based on available width
    let num_cols = (chunks[1].width as usize / COL_WIDTH as u16 as usize).max(1).min(3);

    // Build rows of features (row-first order) - this matches linear navigation
    let mut lines: Vec<Line<'static>> = Vec::new();

    for row_start in (0..feature_keys.len()).step_by(num_cols) {
        let mut spans = Vec::new();

        for col in 0..num_cols {
            let idx = row_start + col;
            if idx >= feature_keys.len() {
                break;
            }

            let key = &feature_keys[idx];
            let global_idx = idx + 2; // +2 for Layout and Keyboard

            let is_selected = app.setting_index == global_idx;
            let sel = is_selected && focused;
            let enabled = app.settings.features.get(key).copied().unwrap_or(false);

            let checkbox = if enabled { "[x]" } else { "[ ]" };
            let short_key = shorten_feature_name(key);

            // Build item with fixed width padding - use separate spans for styling
            let prefix = if sel { ">" } else { " " };

            // Prefix
            spans.push(Span::styled(
                prefix,
                Style::default().fg(if sel { t.palette.accent_primary } else { t.palette.text_muted }),
            ));

            // Checkbox
            spans.push(Span::styled(checkbox, t.checkbox(enabled)));

            // Key name with padding
            let key_text = format!(" {}{}", short_key, " ".repeat(COL_WIDTH.saturating_sub(5 + short_key.len())));
            spans.push(Span::styled(
                key_text,
                Style::default().fg(if sel {
                    t.palette.accent_primary
                } else {
                    t.palette.text_secondary
                }),
            ));
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, chunks[1]);
}

fn shorten_feature_name(name: &str) -> String {
    name.trim_start_matches("Desktop")
        .trim_start_matches("Distro")
        .trim_start_matches("DistroUbuntuOr")
        .to_string()
}
