use std::{
    io::{self, Stdout},
    time::Duration,
};

use eyre::{Result, Context};

use ratatui::{Terminal, backend::CrosstermBackend};

use crossterm::{
    event,
    execute,
    terminal,
};

use app::{App, AppReturn};

use inputs::{InputEvent, key::Key};

use crate::app::ui;

pub mod app;
pub mod inputs;
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
        
        for input_event in get_input_events(Duration::from_millis(200)) {
            let result = match input_event {
                InputEvent::Pressed(key) => app.key_pressed(key),
                InputEvent::Released(key) => app.key_released(key),
                // TODO: Handle repeat if needed
                InputEvent::Repeat(_key) => AppReturn::Continue,
            };

            // Check if we should exit
            if result == AppReturn::Exit {
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

pub fn get_input_events(tick_rate: Duration) -> Vec<InputEvent> {
    let mut input_events: Vec<InputEvent> = Vec::new();

    while event::poll(tick_rate).context("event poll failed").unwrap()  {
        
        if let event::Event::Key(key_event) = event::read().context("event read failed").unwrap() {
            let key = Key::from(key_event);
            let input_event = match key_event.kind {
                event::KeyEventKind::Press => InputEvent::Pressed(key),
                event::KeyEventKind::Release => InputEvent::Released(key),
                event::KeyEventKind::Repeat => InputEvent::Repeat(key),
            };
            input_events.push(input_event);
        }
    }

    input_events
}