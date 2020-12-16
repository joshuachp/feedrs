use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
};
use syndication::Feed;
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

// TODO: Check for errors in unwraps and is just a test, maybe refactor
pub fn main_view<B>(
    terminal: &mut Terminal<B>,
    content: &RwLock<HashMap<Arc<String>, Feed>>,
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
