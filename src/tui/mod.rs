mod app;
mod handlers;
mod theme;
mod ui;

use std::io;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

use app::App;
use handlers::handle_input;
use ui::draw_ui;

pub fn run() -> io::Result<()> {
    let mut app = App::new()?;
    app.refresh_service_status(true);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = loop {
        app.refresh_service_status(false);

        if let Err(err) = terminal.draw(|f| draw_ui(f, &app)) {
            break Err(err);
        }

        if !event::poll(std::time::Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if handle_input(&mut app, key.code)? {
            break Ok(());
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    run_result
}
