use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    io::{stdout, Write},
    sync::{Arc, Mutex},
};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod configuration;
mod content;
mod database;
mod update;

use crate::app::App;

fn input_thread(inputs: &Arc<Mutex<Vec<KeyEvent>>>) {
    let inputs = Arc::clone(inputs);
    tokio::spawn(async move {
        loop {
            // Blocks until event/input
            match crossterm::event::read() {
                Ok(event) => {
                    if let Event::Key(event) = event {
                        inputs.lock().unwrap().push(event);
                        if event.code == KeyCode::Char('q') {
                            return;
                        }
                    }
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    });
}

fn close_application() -> crossterm::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read configuration
    let config = configuration::config(std::env::args())?;
    // Create database pool
    let pool = Arc::new(database::get_database(&config.cache_path).await?);
    // Initialize UI
    enable_raw_mode()?;
    let mut std_out = io::stdout();
    // Open another screen to clean the output
    execute!(std_out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std_out);
    let terminal = Terminal::new(backend)?;
    let mut app = App::new(terminal);
    // Request all the content
    database::get_all(&pool, &app.content).await?;
    // Draws the area every 50 milliseconds
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    // Shared collection of events with input thread
    let inputs = Arc::new(Mutex::new(Vec::new()));
    // Sync list of events
    // Starts user input thread
    input_thread(&inputs);
    // Starts update thread
    update::update_thread(&config, &pool, &app.content);
    // Main loop
    loop {
        // Drawing tick
        interval.tick().await;

        let events: Vec<KeyEvent>;
        // Consume all the inputs in the shared collections
        {
            events = inputs.lock().unwrap().drain(0..).collect();
        }

        for event in events {
            match event.code {
                KeyCode::Char('h') | KeyCode::Left => {
                    app.set_view_article(false);
                }
                KeyCode::Char('j') | KeyCode::Down => app.down_key_event(),
                KeyCode::Char('k') | KeyCode::Up => app.up_key_event(),
                KeyCode::Char('l') | KeyCode::Right => {
                    app.set_view_article(true);
                }
                KeyCode::Enter => {
                    app.set_view_article(true);
                }
                KeyCode::Esc => {
                    app.set_view_article(false);
                }
                KeyCode::Char('q') => {
                    close_application()?;
                    return Ok(());
                }
                _ => {}
            }
        }

        app.draw()?;
    }
}
