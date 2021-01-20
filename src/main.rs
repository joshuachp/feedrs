use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, io::stdout, io::Write};
use tokio::sync::{mpsc, mpsc::error::TryRecvError};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod configuration;
mod content;
mod database;
mod update;

use crate::app::App;

fn input_thread(mut sender: mpsc::Sender<KeyEvent>) {
    tokio::spawn(async move {
        loop {
            // Blocks until event/input
            match crossterm::event::read() {
                Ok(event) => {
                    if let Event::Key(event) = event {
                        if let Err(err) = sender.send(event).await {
                            panic!("{}", err);
                        }
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
    let pool = database::get_database(&config.cache_path).await?;
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
    // Input thread channel
    // NOTE: Buffer up to 10 keys
    let (sender, mut reciver) = mpsc::channel(10);
    // Starts user input thread
    input_thread(sender);
    // Starts update thread
    update::update_thread(&config, &app.content);
    // Main loop
    loop {
        // Drawing tick
        interval.tick().await;
        loop {
            match reciver.try_recv() {
                Ok(event) => match event.code {
                    KeyCode::Char('h') | KeyCode::Left => {
                        app.view_article = false;
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.list_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.list_previous(),
                    KeyCode::Char('l') | KeyCode::Right => {
                        app.view_article = true;
                    }
                    KeyCode::Enter => {
                        app.view_article = true;
                    }
                    KeyCode::Esc => {
                        app.view_article = false;
                    }
                    KeyCode::Char('q') => {
                        close_application()?;
                        return Ok(());
                    }
                    _ => {}
                },
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(err @ TryRecvError::Closed) => {
                    close_application()?;
                    panic!(err);
                }
            }
        }

        app.draw()?;
    }
}
