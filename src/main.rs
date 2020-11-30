mod configuration;
mod database;
mod update;

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::io;
use std::sync::{Arc, Mutex, RwLock};
use syndication::Feed;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, List, ListItem};
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

// TODO: Check for errors in unwraps and is just a test, maybe refactor
fn draw<B>(
    terminal: &mut Terminal<B>,
    content: &Arc<RwLock<HashMap<Arc<String>, Feed>>>,
) -> io::Result<()>
where
    B: Backend,
{
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        let content = content.read().unwrap();

        let items: Vec<ListItem> = content
            .values()
            .flat_map(|feed| match feed {
                Feed::Atom(feed) => feed
                    .entries()
                    .iter()
                    .map(|entry| {
                        let lines = vec![
                            Spans::from(entry.title()),
                            Spans::from(entry.summary().unwrap()),
                        ];
                        return ListItem::new(lines)
                            .style(Style::default().fg(Color::Black).bg(Color::White));
                    })
                    .collect::<Vec<ListItem>>(),

                Feed::RSS(feed) => feed
                    .items()
                    .iter()
                    .map(|entry| {
                        let lines = vec![
                            Spans::from(entry.title().unwrap()),
                            Spans::from(entry.description().unwrap()),
                        ];
                        return ListItem::new(lines)
                            .style(Style::default().fg(Color::Black).bg(Color::White));
                    })
                    .collect::<Vec<ListItem>>(),
            })
            .collect();

        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        f.render_widget(items, chunks[0]);
    })?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Read configuration
    let config = configuration::config(std::env::args())?;
    // Create database pool
    let _pool = database::create_database(&config.cache_path).await?;

    // Map of the source url and content of the feeds.
    let content: Arc<RwLock<HashMap<Arc<String>, Feed>>> = Arc::new(RwLock::new(HashMap::new()));

    // Initialize TUI
    let stdout = io::stdout();
    let screen = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(screen.into_raw_mode()?);
    let mut terminal = Terminal::new(backend)?;

    // Draws the area every 50 milliseconds
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));

    // FIFO of the inputs
    let inputs = Arc::new(Mutex::new(VecDeque::<Key>::new()));
    // Flag for closing all threads

    // Starts user input thread
    input_thread(&inputs);

    // Starts update thread
    update::update_thread(&config, &content);

    // Clear the terminal before drawing
    terminal.clear()?;

    loop {
        // Drawing tick
        interval.tick().await;

        draw(&mut terminal, &content)?;

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
