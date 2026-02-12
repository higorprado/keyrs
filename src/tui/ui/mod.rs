mod footer;
mod header;
mod output_panel;
mod settings_panel;

use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::tui::app::App;

pub fn draw_ui(frame: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // Compact header
            Constraint::Length(6),   // Service control block (status + commands)
            Constraint::Min(8),      // Settings block
            Constraint::Length(12),  // Output block (1.5x larger)
            Constraint::Length(2),   // Footer
        ])
        .split(frame.area());

    header::render(frame, app, root[0]);
    render_service_control(frame, app, root[1]);
    settings_panel::render(frame, app, root[2]);
    output_panel::render(frame, app, root[3]);
    footer::render(frame, app, root[4]);
}

fn render_service_control(frame: &mut Frame, app: &App, area: Rect) {
    let t = crate::tui::theme::theme();
    let focused = app.focused_pane == crate::tui::app::Pane::Commands;

    let block = Block::default()
        .title(t.panel_title("SERVICE CONTROL", focused))
        .borders(Borders::ALL)
        .border_style(t.panel_border(focused))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into: status line (2) + commands (remaining)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(2)])
        .split(inner);

    // Status line
    let status_state = match app.service_state.as_str() {
        "active" => "* active",
        "inactive" => "o inactive",
        "failed" => "x failed",
        _ => "? unknown",
    };

    let status_line = Line::from(vec![
        Span::styled("Status: ", t.text_muted()),
        Span::styled(
            status_state,
            if app.service_state == "active" {
                Style::default().fg(t.palette.accent_success)
            } else if app.service_state == "failed" {
                Style::default().fg(t.palette.accent_danger)
            } else {
                Style::default().fg(t.palette.accent_warning)
            },
        ),
    ]);

    let status_para = Paragraph::new(status_line);
    frame.render_widget(status_para, chunks[0]);

    // Commands as horizontal bar with selection highlight
    let selected_idx = app.command_index;
    let mut spans = vec![];

    for (i, cmd) in app.commands.iter().enumerate() {
        let is_selected = i == selected_idx && focused;
        let destructive = cmd.is_destructive();

        let label = if destructive {
            format!("[{}]!", cmd.label)
        } else {
            format!("[{}]", cmd.label)
        };

        let style = if is_selected {
            Style::default()
                .fg(t.palette.selection_fg)
                .bg(t.palette.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else if destructive {
            Style::default().fg(t.palette.accent_danger)
        } else {
            t.text_secondary()
        };

        spans.push(Span::styled(label, style));
        spans.push(Span::raw("  "));
    }

    let commands_line = Line::from(spans);
    let commands_para = Paragraph::new(commands_line);
    frame.render_widget(commands_para, chunks[1]);
}
