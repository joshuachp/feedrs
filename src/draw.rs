use std::{collections::HashSet, io, sync::RwLock};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
};
use tui::{
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};

use crate::content::Article;

// TODO: Check for errors in unwraps and is just a test, maybe refactor
pub fn main_view<B>(
    terminal: &mut Terminal<B>,
    content: &RwLock<HashSet<Article>>,
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
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        f.render_widget(items, chunks[0]);
    })?;

    Ok(())
}
