use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::tui::app::{App, Pane};
use crate::tui::theme::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let t = theme();
    let focused = app.focused_pane == Pane::Output;

    let block = Block::default()
        .title(t.panel_title("OUTPUT LOG", focused))
        .borders(Borders::ALL)
        .border_style(t.panel_border(focused))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.output.is_empty() {
        let empty = Paragraph::new(Line::styled(
            "No output yet. Run a command to see output here.",
            t.text_muted(),
        ));
        frame.render_widget(empty, inner);
        return;
    }

    // Calculate visible lines based on inner area height
    let visible_height = inner.height.saturating_sub(1) as usize;
    let total_lines = app.output.len();

    // Clamp scroll position
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll = app.output_scroll.min(max_scroll);

    // Get the visible slice of output
    let start = scroll;
    let end = (start + visible_height).min(total_lines);

    let lines: Vec<Line> = app.output[start..end]
        .iter()
        .map(|l| {
            if l.starts_with('$') {
                Line::styled(l.clone(), t.output_command())
            } else {
                Line::styled(l.clone(), t.output_text())
            }
        })
        .collect();

    let output = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(output, inner);
}
