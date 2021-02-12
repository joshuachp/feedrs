use chrono::{DateTime, FixedOffset};
use sqlx::FromRow;
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

// Partial order articles from newer to older, so we reverse the order of the date compare
impl PartialOrd for Article {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let Some(order) = self.date.partial_cmp(&other.date) {
            Some(order.reverse())
        } else {
            None
        }
    }
}

// Partial order articles from newer to older, so we reverse the order of the date compare
impl Ord for Article {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date).reverse()
    }
}

pub fn parse_content(source: &str, content: String) -> anyhow::Result<Vec<Article>> {
    let feed = content.parse::<Feed>().unwrap();
    Ok(match feed {
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

fn parse_html(html: &str) -> String {
    html2text::from_read(html.as_bytes(), 80).trim().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

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
