use std::{
    collections::HashSet,
    io,
    sync::{Arc, RwLock},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

use crate::content::Article;

pub struct App {
    // Set of the articles
    pub content: Arc<RwLock<HashSet<Article>>>,
    // List state
    pub list_state: ListState,
}

impl App {
    pub fn new() -> App {
        App {
            content: Arc::new(RwLock::new(HashSet::new())),
            list_state: ListState::default(),
        }
    }

    // TODO: Check for errors in unwraps and is just a test, maybe refactor
    pub fn main_view<B>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()>
    where
        B: Backend,
    {
        let content = Arc::clone(&self.content);
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let content = content.read().unwrap();

            let items: Vec<ListItem> = content
                .iter()
                .map(|article| {
                    let lines = vec![
                        Spans::from(article.title.clone()),
                        Spans::from(article.sub_title.clone()),
                    ];
                    ListItem::new(lines)
                })
                .collect();

            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("List"))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(items, chunks[0], &mut self.list_state);
        })?;

        Ok(())
    }

    pub fn list_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.content.read().unwrap().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn list_previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.content.read().unwrap().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
