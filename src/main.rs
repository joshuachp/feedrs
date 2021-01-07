use crossterm::{
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use database::get_all;
use std::{
    collections::VecDeque,
    io,
    io::stdout,
    io::Write,
    sync::{Arc, Mutex},
};
use tui::{backend::CrosstermBackend, Terminal};

mod configuration;
mod content;
mod database;
mod ui;
mod update;

use crate::ui::App;
use crate::update::update_thread;

fn input_thread(inputs: &Arc<Mutex<VecDeque<KeyEvent>>>) {
    let inputs = Arc::clone(inputs);
    tokio::spawn(async move {
        loop {
            // Blocks until event/input
            // TODO: Catch error
            match read().unwrap() {
                Event::Key(event) => {
                    let mut inputs = inputs.lock().unwrap();
                    inputs.push_back(event);
                    if event.code == KeyCode::Char('q') {
                        return;
                    }
                }
                _ => {}
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
    let pool = database::create_database(&config.cache_path).await?;
    // Initialize UI
    enable_raw_mode()?;
    let mut std_out = io::stdout();
    // Open another screen to clean the output
    execute!(std_out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std_out);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    app.list_state.select(Some(0));
    // Request all the content
    get_all(&pool, &app.content).await?;
    // Draws the area every 50 milliseconds
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
    // FIFO of the inputs
    let inputs = Arc::new(Mutex::new(VecDeque::<KeyEvent>::new()));
    // Starts user input thread
    input_thread(&inputs);
    // Starts update thread
    update_thread(&config, &app.content);
    // Main loop
    loop {
        // Drawing tick
        interval.tick().await;
        // Get new inputs
        let mut inputs = inputs.lock().unwrap();
        for event in inputs.drain(..) {
            match event.code {
                KeyCode::Char('h') | KeyCode::Left => {}
                KeyCode::Char('j') | KeyCode::Down => app.list_next(),
                KeyCode::Char('k') | KeyCode::Up => app.list_previous(),
                KeyCode::Char('l') | KeyCode::Right => {}
                KeyCode::Enter => {}
                KeyCode::Char('q') => {
                    close_application()?;
                    return Ok(());
                }
                _ => {}
            }
        }

        app.main_view(&mut terminal)?;
    }
}
