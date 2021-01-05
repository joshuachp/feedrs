use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tokio::time::{interval, Duration};

use crate::configuration::Config;
use crate::content::{parse_content, Article};

async fn request_content(url: &str) -> reqwest::Result<String> {
    // TODO: Check status and show errors
    reqwest::get(url).await?.text().await
}

async fn update_content(sources: &Vec<Arc<String>>, content: &Arc<RwLock<HashSet<Article>>>) {
    // Update only if there is a source to update from
    if sources.len() > 0 {
        for source in sources {
            let source = Arc::clone(source);
            let content = Arc::clone(content);
            tokio::spawn(async move {
                let articles =
                    parse_content(&source, request_content(&source).await.unwrap()).unwrap();
                let mut content = content.write().unwrap();
                for article in articles {
                    content.insert(article);
                }
            });
        }
    }
}

pub fn update_thread(config: &Config, content: &Arc<RwLock<HashSet<Article>>>) {
    let update_interval = config.update_interval;
    let sources: Vec<Arc<String>> = config.sources.iter().map(|x| Arc::clone(x)).collect();
    let content = Arc::clone(content);

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(update_interval));
        loop {
            interval.tick().await;
            update_content(&sources, &content).await;
        }
    });
}

#[cfg(test)]
mod test {

    use super::request_content;

    #[tokio::test]
    async fn test_request_content() {
        request_content("https://joshuachp.github.io/index.xml")
            .await
            .unwrap();
    }
}
