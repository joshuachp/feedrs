use anyhow;
use sqlx::FromRow;
use std::hash::{Hash, Hasher};
use syndication::Feed;

#[derive(FromRow, PartialEq, Eq)]
pub struct Article {
    pub id: String,
    pub source: String,
    pub title: String,
    pub sub_title: String,
    pub content: String,
}

impl Hash for Article {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.source.hash(state);
    }
}

pub fn parse_content(source: &str, content: String) -> anyhow::Result<Vec<Article>> {
    let feed = content.parse::<Feed>().unwrap();
    Ok(match feed {
        Feed::Atom(feed) => feed
            .entries()
            .iter()
            .map(|entry| Article {
                id: String::from(entry.id()),
                source: String::from(source),
                title: String::from(entry.title()),
                sub_title: String::from(entry.summary().unwrap_or("")),
                // TODO: Set content
                content: String::from(""),
            })
            .collect(),
        Feed::RSS(channel) => channel
            .items()
            .iter()
            .map(|item| Article {
                id: String::from(""),
                source: String::from(""),
                title: String::from(item.title().unwrap_or("")),
                sub_title: String::from(item.description().unwrap_or("")),
                // TODO: Set content
                content: String::from(""),
            })
            .collect(),
    })
}
