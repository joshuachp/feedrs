mod configuration;
mod database;
mod update;

use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;

fn input_thread(inputs: &Arc<Mutex<VecDeque<Key>>>) {
    let stdin = io::stdin();
    let inputs = Arc::clone(inputs);
    tokio::spawn(async move {
        loop {
            // Wait for some time to get input
            let stdin = stdin.lock();
            let keys = stdin.keys();
            // This loop never ends until q is pressed
            for key in keys {
                if let Ok(key) = key {
                    let mut inputs = inputs.lock().unwrap();
                    inputs.push_back(key);

                    if key == Key::Char('q') {
                        return;
                    }
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Read configuration
    let config = configuration::config(std::env::args())?;
    // Create database pool
    let _pool = database::create_database(&config.cache_uri);

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Draws the area every 50 milliseconds
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));

    // FIFO of the inputs
    let inputs = Arc::new(Mutex::new(VecDeque::<Key>::new()));
    // Flag for closing all threads

    // Starts user input thread
    input_thread(&inputs);

    // Clear the terminal before drawing
    terminal.clear()?;

    loop {
        // Drawing tick
        interval.tick().await;

        // TODO: Move to different function
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Feed").borders(Borders::ALL);
            f.render_widget(block, size);
        })?;

        let mut inputs = inputs.lock().unwrap();
        for key in inputs.drain(..) {
            match key {
                Key::Char('q') => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
