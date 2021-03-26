use chrono::{DateTime, FixedOffset};
use sqlx::FromRow;
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    sync::Arc,
};
use syndication::Feed;

#[derive(FromRow, PartialEq, Eq, Clone, Debug)]
pub struct Article {
    pub id: String,
    pub source: String,
    pub title: String,
    pub sub_title: String,
    pub content: String,
    pub date: Option<DateTime<FixedOffset>>,
}

/// Partial order articles from newer to older, so we reverse the order of the date compare
impl PartialOrd for Article {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let date_ordering = self.date.partial_cmp(&other.date);
        if date_ordering == Some(Ordering::Equal) {
            let source_ordering = self.source.partial_cmp(&other.source);
            if source_ordering == Some(Ordering::Equal) {
                return self.id.partial_cmp(&other.id);
            }
            return source_ordering;
        }
        date_ordering
    }
}

/// Order articles from newer to older, so we reverse the order of the date compare
impl Ord for Article {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let date_ordering = self.date.cmp(&other.date);
        if date_ordering == Ordering::Equal {
            let source_order = self.source.cmp(&other.source);
            if source_order == Ordering::Equal {
                return self.id.cmp(&other.id);
            }
            return source_order;
        }
        date_ordering
    }
}

/// Structure that maintains integrity by having unique id and source for each article and sorts
/// the articles. This is because every Article should be identified by the id and source, but we
/// want to maintain the Eq and Hash function to check the whole struct and order it by the update
/// date.
#[derive(Default)]
pub struct ArticleMap {
    // Those need to be Arc because they are shared references
    ids: HashMap<(String, String), Arc<Article>>,
    articles: BTreeSet<Arc<Article>>,
}

/// Those functions grant access to the values inside the ArticleMap restricting mutability to
/// maintain the articles vector sorted
impl ArticleMap {
    /// Insert an article in the map setting the (id, source) into the HashMap
    pub fn insert(&mut self, article: Article) {
        let article = Arc::new(article);
        let key = (article.id.clone(), article.source.clone());
        // If the there is an article in the BTreeSet with the same id and source but other fields
        // different, we need to remove it before inserting the new one
        if let Some(old_article) = self.ids.insert(key, Arc::clone(&article)) {
            if article.ne(&old_article) {
                self.articles.remove(&old_article);
            }
        }
        self.articles.insert(Arc::clone(&article));
    }

    /// Get a reference to the article map's articles.
    pub fn articles(&self) -> &BTreeSet<Arc<Article>> {
        &self.articles
    }

    pub fn remove(&mut self, key: &(String, String)) -> Option<Arc<Article>> {
        let value = self.ids.remove(key);
        if let Some(value) = value {
            self.articles.remove(&value);
            return Some(value);
        }
        None
    }

    /// Inserts a list of new elements and returns the Vec that where removed from the Vec
    pub fn update_content(
        &mut self,
        content_update: &HashMap<(String, String), Article>,
    ) -> HashMap<(String, String), Arc<Article>> {
        let mut ret: HashMap<(String, String), Arc<Article>> = HashMap::new();
        // Remove the values not found in the update
        let keys: Vec<(String, String)> = self.ids.keys().cloned().collect();
        for key in keys {
            if !content_update.contains_key(&key) {
                let value = self.remove(&key);
                ret.insert(key, value.unwrap());
            }
        }
        content_update.values().for_each(|x| self.insert(x.clone()));
        ret
    }
}

/// Parses an RSS or Atom iterm/feed into a collection of Articles
pub fn parse_content(source: &str, content: String) -> anyhow::Result<Vec<Article>> {
    let feed = content.parse::<Feed>().unwrap();
    Ok(match feed {
        // Atom feed
        Feed::Atom(feed) => feed
            .entries()
            .iter()
            .map(|entry| {
                let content = if let Some(content) = entry.content() {
                    parse_html(content.value().unwrap_or(""))
                } else {
                    String::from("")
                };
                let update = if let Ok(date) = DateTime::parse_from_rfc3339(entry.updated()) {
                    Some(date)
                } else {
                    None
                };
                return Article {
                    id: String::from(entry.id()),
                    source: String::from(source),
                    title: String::from(entry.title()),
                    sub_title: parse_html(entry.summary().unwrap_or("")),
                    content,
                    date: update,
                };
            })
            .collect(),
        // RSS feed
        Feed::RSS(channel) => channel
            .items()
            .iter()
            .map(|item| {
                let id = if let Some(guid) = item.guid() {
                    String::from(guid.value())
                } else {
                    String::from("")
                };
                let content = if let Some(content) = item.content() {
                    parse_html(content)
                } else {
                    String::from("")
                };
                let update = if let Some(date) = item.pub_date() {
                    DateTime::parse_from_rfc2822(date).ok()
                } else {
                    None
                };
                return Article {
                    id,
                    source: String::from(source),
                    title: String::from(item.title().unwrap_or("")),
                    sub_title: parse_html(item.description().unwrap_or("")),
                    content,
                    date: update,
                };
            })
            .collect(),
    })
}

/// Parses an html formated text into a pretty representation easy to view in the terminal
fn parse_html(html: &str) -> String {
    // Set max line-length to 160 (80 * 2) this could be better
    html2text::from_read(html.as_bytes(), 160)
        .trim()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_article_map_ordering_date_none() {
        let article_1 = Article {
            id: "1".to_owned(),
            source: "source".to_owned(),
            title: "title".to_owned(),
            sub_title: "sub_title".to_owned(),
            content: "content".to_owned(),
            date: None,
        };
        let mut article_2 = article_1.clone();
        article_2.id = "2".to_owned();
        let mut article_3 = article_1.clone();
        article_3.id = "3".to_owned();
        let mut article_1_copy = article_1.clone();
        article_1_copy.id = "1".to_owned();
        // Test
        let mut article_map: ArticleMap = Default::default();
        article_map.insert(article_3.clone());
        article_map.insert(article_1.clone());
        article_map.insert(article_2.clone());
        article_map.insert(article_1_copy.clone());
        let result: Vec<Article> = article_map.articles.iter().map(|x| (**x).clone()).collect();
        let expected = vec![article_1, article_2, article_3];
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_html_base() {
        let expected = String::from("Hello World!");
        assert_eq!(parse_html("<span>Hello World!</span>"), expected);
    }

    #[test]
    fn test_parse_html_multi() {
        let expected = String::from("Hello World!\n\nGood Bye World!");
        assert_eq!(
            parse_html("<p>Hello World!</p><p>Good Bye World!</p>"),
            expected
        );
    }

    #[test]
    fn test_parse_html_link() {
        let expected = String::from("Here is a [link][1]\n\n[1] https://example.com");
        assert_eq!(
            parse_html("<span>Here is a <a href=\"https://example.com\">link</a></span>"),
            expected
        );
    }
}
