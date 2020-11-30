use crossterm::{
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex, RwLock};
use std::{io, io::stdout, io::Write};
use syndication::Feed;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Terminal;

mod configuration;
mod database;
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

fn close_application() -> crossterm::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
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
    update::update_thread(&config, &content);

    loop {
        // Drawing tick
        interval.tick().await;

        draw(&mut terminal, &content)?;

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
