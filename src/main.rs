use crossterm::{
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use database::get_all;
use std::{
    collections::{HashMap, VecDeque},
    io,
    io::stdout,
    io::Write,
    sync::{Arc, Mutex, RwLock},
};
use syndication::Feed;
use tui::{backend::CrosstermBackend, Terminal};
use update::update_thread;

mod configuration;
mod database;
mod draw;
mod update;

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

    // Map of the source url and content of the feeds.
    let content: Arc<RwLock<HashMap<Arc<String>, Feed>>> = Arc::new(RwLock::new(HashMap::new()));
    get_all(&pool, &content).await?;

    // Initialize TUI
    enable_raw_mode()?;
    let mut std_out = io::stdout();
    execute!(std_out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std_out);
    let mut terminal = Terminal::new(backend)?;

    // Draws the area every 50 milliseconds
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));

    // FIFO of the inputs
    let inputs = Arc::new(Mutex::new(VecDeque::<KeyEvent>::new()));
    // Flag for closing all threads

    // Starts user input thread
    input_thread(&inputs);

    // Starts update thread
    update_thread(&config, &content);

    loop {
        // Drawing tick
        interval.tick().await;

        draw::main_view(&mut terminal, &content)?;

        let mut inputs = inputs.lock().unwrap();
        for event in inputs.drain(..) {
            match event.code {
                KeyCode::Char('q') => {
                    close_application()?;
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
