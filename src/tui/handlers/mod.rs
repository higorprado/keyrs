use std::io;

use crossterm::event::KeyCode;

use crate::tui::app::{App, Pane, PendingAction};

/// Handle input and return true if the app should quit
pub fn handle_input(app: &mut App, key: KeyCode) -> io::Result<bool> {
    if app.confirm_prompt.is_some() {
        return handle_confirmation(app, key);
    }

    // Global navigation: Tab cycles panes, 1/2/3 jump to specific pane
    match key {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Tab => {
            app.cycle_pane_forward();
            return Ok(false);
        }
        KeyCode::BackTab => {
            app.cycle_pane_backward();
            return Ok(false);
        }
        KeyCode::Char('1') => {
            app.focused_pane = Pane::Commands;
            return Ok(false);
        }
        KeyCode::Char('2') => {
            app.focused_pane = Pane::Settings;
            return Ok(false);
        }
        KeyCode::Char('3') => {
            app.focused_pane = Pane::Output;
            return Ok(false);
        }
        _ => {}
    }

    // Pane-specific input
    handle_pane_input(app, key);

    Ok(false)
}

fn handle_confirmation(app: &mut App, key: KeyCode) -> io::Result<bool> {
    match key {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            if let Some(action) = app.pending_action.take() {
                app.clear_confirm();
                match action {
                    PendingAction::RunCommand(index) => app.run_command_index(index),
                    PendingAction::SaveAndRestart => app.save_settings(true),
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.set_status("Cancelled");
            app.clear_confirm();
        }
        _ => {}
    }
    Ok(false)
}

fn handle_pane_input(app: &mut App, key: KeyCode) {
    match app.focused_pane {
        Pane::Commands => handle_commands_input(app, key),
        Pane::Settings => handle_settings_input(app, key),
        Pane::Output => handle_output_input(app, key),
    }
}

fn handle_commands_input(app: &mut App, key: KeyCode) {
    // Commands are displayed horizontally, so use Left/Right to navigate
    match key {
        KeyCode::Left | KeyCode::Char('h') => {
            if app.command_index > 0 {
                app.command_index -= 1;
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.command_index + 1 < app.commands.len() {
                app.command_index += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => app.run_selected_command(),
        _ => {}
    }
}

fn handle_settings_input(app: &mut App, key: KeyCode) {
    // Settings are a vertical list, use Up/Down to navigate
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.setting_index > 0 {
                app.setting_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.setting_index + 1 < app.setting_entries.len() {
                app.setting_index += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => app.change_selected_setting(),
        KeyCode::Char('s') => app.save_settings(false),
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.start_confirm(
                "Save settings and restart service?",
                PendingAction::SaveAndRestart,
            )
        }
        _ => {}
    }
}

fn handle_output_input(app: &mut App, key: KeyCode) {
    // Output log scrolling
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            app.output_scroll = app.output_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.output_scroll = app.output_scroll.saturating_add(1);
        }
        _ => {}
    }
}
