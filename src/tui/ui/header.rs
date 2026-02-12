use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::tui::app::App;
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let t = theme();

    let display_path = app.service_ctl.display().to_string();
    let path_display = if display_path.len() > 45 {
        format!("...{}", &display_path[display_path.len() - 42..])
    } else {
        display_path
    };

    let state_upper = app.service_state.to_uppercase();
    let state_label = match app.service_state.as_str() {
        "active" => "RUNNING",
        "inactive" => "STOPPED",
        "failed" => "FAILED",
        _ => &state_upper,
    };

    let line = Line::from(vec![
        // App name
        Span::styled("keyrs ", t.text_primary().add_modifier(Modifier::BOLD)),
        // Status badge
        Span::styled(format!("[{}] ", state_label), t.status_badge(&app.service_state)),
        // Service controller path
        Span::styled(path_display, t.text_muted()),
        // Spacer
        Span::raw(" "),
        // Right-aligned quit hint
        Span::styled(
            "q:quit",
            t.key_hint(),
        ),
    ]);

    let header = Paragraph::new(line).alignment(Alignment::Left);

    frame.render_widget(header, area);
}
