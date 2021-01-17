use chrono::DateTime;
use chrono::FixedOffset;
use html2text::from_read;
use sqlx::FromRow;
use syndication::Feed;

#[derive(FromRow, PartialEq, Eq, Clone)]
pub struct Article {
    pub id: String,
    pub source: String,
    pub title: String,
    pub sub_title: String,
    pub content: String,
    pub date: Option<DateTime<FixedOffset>>,
}

//impl Hash for Article {
//    fn hash<H: Hasher>(&self, state: &mut H) {
//        self.id.hash(state);
//        self.source.hash(state);
//    }
//}

impl PartialOrd for Article {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl Ord for Article {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date)
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
                    id: String::from(""),
                    source: String::from(""),
                    title: String::from(item.title().unwrap_or("")),
                    sub_title: parse_html(item.description().unwrap_or("")),
                    content,
                    date: update,
                };
            })
            .collect(),
    })
}

/*
 * Parse the html of a feed item content into simple text
 */
fn parse_html(content: &str) -> String {
    // TODO: Set proper line length, default to 80
    String::from(from_read(content.as_bytes(), 80).trim())
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
