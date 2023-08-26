use std::{
    io::{self, Stdout},
    time::Duration,
};

use eyre::{Result, Context};

use ratatui::{Terminal, backend::CrosstermBackend};

use crossterm::{
    event::{self, KeyEvent},
    execute,
    terminal,
};

use app::{App, AppReturn};
use utils::event_handler::{EventsHandler, TerminalEvent};

use crate::app::ui;

pub mod app;
pub mod utils;

pub fn start_terminal_app(app: &mut App) -> Result<()> {
    let mut terminal = setup_terminal()?;

    let mut events_handler = EventsHandler::new(Duration::from_millis(100));

    let loop_error_report = run_loop(app, &mut terminal, &mut events_handler);

    restore_terminal(&mut terminal)?;
    events_handler.close();

    if let Err(e) = loop_error_report {
        println!("Error in the main loop:\n{}", e);
    }

    Ok(())
}

fn run_loop(app: &mut App, terminal: &mut Terminal<CrosstermBackend<Stdout>>, events_handler: &mut EventsHandler) -> Result<()> {
    loop {
        // Check if the terminal is big enough
        ui::check_size(&terminal.get_frame().size()).context("Unable to continue, the terminal is too small.")?;

        terminal.draw(|frame| ui::draw(frame, app))?;
        
        if let Ok(event) = events_handler.next() {
            match event {
                TerminalEvent::Event(event) => {
                    
                    if let event::Event::Key(key_event) = event {
                        // Check if we should exit
                        if app.key_event(key_event) == AppReturn::Exit {
                            return Ok(());
                        }
                    }
                },
                TerminalEvent::Tick => {
                    app.tick();
                }
            }
        }

        // wait for the next tick
        std::thread::sleep(Duration::from_millis(500));
    }
}


fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}