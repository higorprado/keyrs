use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let t = theme();

    if let Some(prompt) = &app.confirm_prompt {
        render_confirm_prompt(frame, app, area, prompt, t);
    } else {
        render_status_and_hints(frame, app, area, t);
    }
}

fn render_confirm_prompt(frame: &mut Frame, app: &App, area: Rect, prompt: &str, t: &crate::tui::theme::Theme) {
    let lines = vec![
        Line::styled(format!("Status: {}", app.status), t.status_message()),
        Line::from(vec![
            Span::styled(format!("CONFIRM: {} ", prompt), t.confirm_prompt()),
            Span::styled("[", t.text_muted()),
            Span::styled("y", t.key_hint()),
            Span::styled("/", t.text_muted()),
            Span::styled("Enter", t.key_hint()),
            Span::styled(":yes  ", t.text_muted()),
            Span::styled("[", t.text_muted()),
            Span::styled("n", t.key_hint()),
            Span::styled("/", t.text_muted()),
            Span::styled("Esc", t.key_hint()),
            Span::styled(":no]", t.text_muted()),
        ]),
    ];

    let footer = Paragraph::new(lines);
    frame.render_widget(footer, area);
}

fn render_status_and_hints(frame: &mut Frame, app: &App, area: Rect, t: &crate::tui::theme::Theme) {
    // Build context-sensitive first line based on focused pane
    let line1 = match app.focused_pane {
        crate::tui::app::Pane::Commands => {
            if let Some(cmd) = app.selected_command() {
                let safety = if cmd.is_destructive() { "destructive" } else { "safe" };
                Line::from(vec![
                    Span::styled(format!("{} ", cmd.label), t.text_primary().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("({}) ~ ", safety), t.text_muted()),
                    Span::styled("Enter", t.key_hint()),
                    Span::styled(format!(" to run {}", cmd.command), t.text_muted()),
                ])
            } else {
                Line::styled(format!("Status: {}", app.status), t.status_message())
            }
        }
        crate::tui::app::Pane::Settings => {
            if let Some(entry) = app.selected_setting() {
                let hint = match entry {
                    crate::tui::app::SettingEntry::LayoutOptspec => "cycle ABC ↔ US",
                    crate::tui::app::SettingEntry::KeyboardOverride => "cycle keyboard type",
                    crate::tui::app::SettingEntry::Feature(_) => "toggle on/off",
                };
                Line::from(vec![
                    Span::styled("Toggle setting ~ ", t.text_muted()),
                    Span::styled("Enter", t.key_hint()),
                    Span::styled(format!(" {} ", hint), t.text_muted()),
                    Span::styled("s", t.key_hint()),
                    Span::styled(":save", t.text_muted()),
                ])
            } else {
                Line::styled(format!("Status: {}", app.status), t.status_message())
            }
        }
        crate::tui::app::Pane::Output => {
            Line::styled(format!("Output log ~ {} lines", app.output.len()), t.text_muted())
        }
    };

    // Line 2: Key hints for the 3-block layout
    let line2 = Line::from(vec![
        Span::styled("Tab", t.key_hint()),
        Span::styled(":", t.text_muted()),
        Span::styled("1-Commands", if app.focused_pane == crate::tui::app::Pane::Commands { t.key_hint() } else { t.text_muted() }),
        Span::styled(" ", t.text_muted()),
        Span::styled("2-Settings", if app.focused_pane == crate::tui::app::Pane::Settings { t.key_hint() } else { t.text_muted() }),
        Span::styled(" ", t.text_muted()),
        Span::styled("3-Output", if app.focused_pane == crate::tui::app::Pane::Output { t.key_hint() } else { t.text_muted() }),
        Span::styled("  ", t.text_muted()),
        Span::styled("arrows", t.key_hint()),
        Span::styled(":navigate  ", t.text_muted()),
        Span::styled("Enter", t.key_hint()),
        Span::styled(":action  ", t.text_muted()),
        Span::styled("q", t.key_hint()),
        Span::styled(":quit", t.text_muted()),
    ]);

    let footer = Paragraph::new(vec![line1, line2]);
    frame.render_widget(footer, area);
}
