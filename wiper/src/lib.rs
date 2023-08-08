use std::{
    io::{self, Stdout},
    time::Duration,
};

use eyre::{Result, Context};

use log::{error, warn};
use ratatui::{Terminal, backend::CrosstermBackend};

use crossterm::{
    execute,
    terminal,
};

use app::{App, AppReturn};

use inputs::events::Events;
use inputs::InputEvent;

use crate::app::ui;

pub mod app;
pub mod inputs;
pub mod utils;

pub async fn start_terminal_app(app: &mut App) -> Result<()> {
    let mut terminal = setup_terminal()?;

    // User event handler
    let mut events = Events::new(Duration::from_millis(200));

    let loop_error_report = run_loop(app, &mut events, &mut terminal).await;

    restore_terminal(&mut terminal)?;

    if let Err(e) = loop_error_report {
        println!("Error in the main loop:\n{}", e);
    }

    Ok(())
}

async fn run_loop(app: &mut App, events: &mut Events, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    Ok(loop {
        // Check if the terminal is big enough
        ui::check_size(&terminal.get_frame().size()).context("Unable to continue, the terminal is too small.")?;

        terminal.draw(|frame| ui::draw(frame, app))?;
        
        // Handle inputs
        let result = match events.next().await {
            InputEvent::Pressed(key) => app.key_pressed(key).await,
            InputEvent::Released(key) => app.key_released(key).await,
            // TODO: Handle repeat if needed
            InputEvent::Repeat(_key) => AppReturn::Continue,
            InputEvent::Tick => app.update_on_tick().await,
        };

        // Check if we should exit
        if result == AppReturn::Exit {
            events.close();
            break;
        }
    })
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