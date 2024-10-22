use std::{
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

use crate::content::{Article, ArticleMap};

pub struct App<B>
where
    B: Backend,
{
    // Set of the articles
    pub content: Arc<RwLock<ArticleMap>>,
    // List state
    pub list_state: ListState,
    // TUI terminal
    pub terminal: Terminal<B>,
    article: Option<Arc<Article>>,
    max_scroll: Option<u16>,
    scroll: u16,
    view_article: bool,
}

impl<B> App<B>
where
    B: Backend,
{
    pub fn new(terminal: Terminal<B>) -> App<B> {
        App::<B> {
            content: Arc::new(RwLock::new(ArticleMap::default())),
            list_state: ListState::default(),
            terminal,
            view_article: false,
            article: None,
            scroll: 0,
            max_scroll: None,
        }
    }

    pub fn draw(&mut self) -> io::Result<()> {
        if self.view_article {
            self.draw_article_view()
        } else {
            self.draw_main_view()
        }
    }

    fn draw_main_view(&mut self) -> io::Result<()> {
        let content = &self.content;
        let list_state = &mut self.list_state;
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let content = content.read().unwrap();

            let items: Vec<ListItem> = content.articles()
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
            let scroll = &mut self.scroll;
            let max_scroll = &mut self.max_scroll;
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

                // If max_scroll is not set calculate max_scroll or has changed
                let current_max_scroll = u16::try_from(text.height())
                    .unwrap_or(u16::MAX)
                    .saturating_sub(f.size().height - 4);
                if max_scroll.is_none() || max_scroll.unwrap() != current_max_scroll {
                    *max_scroll = Some(current_max_scroll);
                    if *scroll > current_max_scroll {
                        *scroll = current_max_scroll;
                    }
                }

                let offset = (*scroll, 0);
                let paragraph = Paragraph::new(text)
                    .block(Block::default().title("Article").borders(Borders::ALL))
                    .alignment(tui::layout::Alignment::Left)
                    .scroll(offset)
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
            // Get the article if is selected
            let index = self.list_state.selected().unwrap();
            {
                let content = self.content.read().unwrap();
                let mut articles = content.articles().iter();
                for _ in 0..index {
                    articles.next();
                }
                self.article = Some(Arc::clone(articles.next().unwrap()));
            }
        } else {
            self.article = None;
            self.max_scroll = None;
            self.scroll = 0;
            self.view_article = false;
        }
    }

    pub fn down_key_event(&mut self) {
        if self.view_article {
            self.scroll = self
                .scroll
                .saturating_add(1)
                .min(self.max_scroll.unwrap_or(0));
        } else {
            let content = self.content.read().unwrap();
            // Select an article if there is one to select
            if !content.articles().is_empty() {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i >= content.articles().len() - 1 {
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
            let content = self.content.read().unwrap();
            // Select an article if there is one to select
            if !content.articles().is_empty() {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            content.articles().len() - 1
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
