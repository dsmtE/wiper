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

use crate::app::ui;

pub mod app;
pub mod utils;

pub fn start_terminal_app(app: &mut App) -> Result<()> {
    let mut terminal = setup_terminal()?;

    let loop_error_report = run_loop(app, &mut terminal);

    restore_terminal(&mut terminal)?;

    if let Err(e) = loop_error_report {
        println!("Error in the main loop:\n{}", e);
    }

    Ok(())
}

fn run_loop(app: &mut App, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    loop {
        // Check if the terminal is big enough
        ui::check_size(&terminal.get_frame().size()).context("Unable to continue, the terminal is too small.")?;

        terminal.draw(|frame| ui::draw(frame, app))?;
        
        for key_event in get_key_input_events(Duration::from_millis(200)) {
            // Check if we should exit
            if app.key_event(key_event) == AppReturn::Exit {
                return Ok(());
            }
        }
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

pub fn get_key_input_events(tick_rate: Duration) -> Vec<KeyEvent> {
    let mut input_events: Vec<KeyEvent> = Vec::new();

    while event::poll(tick_rate).context("event poll failed").unwrap()  {
        if let event::Event::Key(key_event) = event::read().context("event read failed").unwrap() {
            input_events.push(key_event);
        }
    }

    input_events
}