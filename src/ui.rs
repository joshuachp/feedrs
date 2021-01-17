use std::{
    collections::BTreeSet,
    io,
    sync::{Arc, RwLock},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};

use crate::content::Article;

pub struct App<B>
where
    B: Backend,
{
    // Set of the articles
    pub content: Arc<RwLock<BTreeSet<Article>>>,
    // List state
    pub list_state: ListState,
    // TUI terminal
    pub terminal: Terminal<B>,
    pub view_article: bool,
}

impl<B> App<B>
where
    B: Backend,
{
    pub fn new(terminal: Terminal<B>) -> App<B> {
        App::<B> {
            content: Arc::new(RwLock::new(BTreeSet::new())),
            list_state: ListState::default(),
            terminal,
            view_article: false,
        }
    }

    pub fn draw(&mut self) -> io::Result<()> {
        if self.view_article {
            self.draw_article_view()
        } else {
            self.draw_main_view()
        }
    }

    // TODO: Check for errors in unwraps and is just a test, maybe refactor
    fn draw_main_view(&mut self) -> io::Result<()> {
        let content = &self.content;
        let list_state = &mut self.list_state;
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let content = content.read().unwrap();

            let items: Vec<ListItem> = content
                .iter()
                .rev()
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
            f.render_stateful_widget(items, chunks[0], list_state);
        })
    }

    fn draw_article_view(&mut self) -> io::Result<()> {
        let index = self.list_state.selected();
        if let Some(index) = index {
            let article: Article;
            {
                let content = self.content.read().unwrap();
                let mut content = content.iter();
                for _ in 0..index {
                    content.next();
                }
                article = content.next().unwrap().clone();
            }
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());

                let text = vec![
                    Spans::from(article.title),
                    Spans::from(article.sub_title),
                    Spans::from(article.content),
                ];
                let paragraph = Paragraph::new(text)
                    .block(Block::default().title("Article").borders(Borders::ALL))
                    .wrap(Wrap { trim: false });
                f.render_widget(paragraph, chunks[0]);
            })
        } else {
            self.draw_main_view()
        }
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
