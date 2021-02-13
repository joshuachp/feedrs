use std::{
    collections::BTreeSet,
    convert::TryFrom,
    io,
    sync::{Arc, RwLock},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
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
    view_article: bool,
    article: Option<Article>,
    scroll: u16,
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
            article: None,
            scroll: 0,
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
                .map(|article| {
                    let lines = vec![Spans::from(article.title.clone())];
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
                .highlight_symbol("> ");
            f.render_stateful_widget(items, chunks[0], list_state);
        })
    }

    fn draw_article_view(&mut self) -> io::Result<()> {
        if self.article.is_some() {
            // Get borrow from self since is not possible inside of closure
            let article = self.article.as_ref().unwrap();
            let scroll = self.scroll;
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());

                // Multi-line text for the content of an article
                let mut text = Text::from(Span::styled(
                    &article.title,
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                text.extend(Text::raw(&article.sub_title));
                text.extend(Text::raw(&article.content));

                let scroll = (
                    scroll.min(
                        u16::try_from(text.height())
                            .unwrap_or(u16::MAX)
                            .saturating_sub(f.size().height),
                    ),
                    0,
                );
                let paragraph = Paragraph::new(text)
                    .block(Block::default().title("Article").borders(Borders::ALL))
                    .alignment(tui::layout::Alignment::Left)
                    .scroll(scroll)
                    .wrap(Wrap { trim: false });
                f.render_widget(paragraph, chunks[0]);
            })
        } else {
            self.draw_main_view()
        }
    }

    pub fn set_view_article(&mut self, view: bool) {
        if (view) && (view != self.view_article) && (self.list_state.selected().is_some()) {
            self.view_article = true;
            self.scroll = 0;
            // Get the article if is selected
            let index = self.list_state.selected().unwrap();
            {
                let content = self.content.read().unwrap();
                let mut content = content.iter();
                for _ in 0..index {
                    content.next();
                }
                self.article = Some(content.next().unwrap().clone());
            }
        } else {
            self.view_article = false;
            self.article = None;
        }
    }

    pub fn down_key_event(&mut self) {
        if self.view_article {
            self.scroll = self.scroll.saturating_add(1);
        } else {
            // Select an article if there is one to select
            if !self.content.read().unwrap().is_empty() {
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
        }
    }

    pub fn up_key_event(&mut self) {
        if self.view_article {
            self.scroll = self.scroll.saturating_sub(1);
        } else {
            // Select an article if there is one to select
            if !self.content.read().unwrap().is_empty() {
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
    }
}
