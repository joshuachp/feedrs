use sqlx::SqlitePool;
use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

use crate::configuration::Config;
use crate::content::{parse_content, Article};

async fn request_content(url: &str) -> reqwest::Result<String> {
    // TODO: Check status and show errors
    reqwest::get(url).await?.text().await
}

async fn update_content(sources: &[Arc<String>], content: &Arc<RwLock<BTreeSet<Article>>>) {
    // Spawns update threads
    let handles: Vec<JoinHandle<()>> = sources
        .iter()
        .map(|source| {
            let source = Arc::clone(source);
            let content = Arc::clone(content);
            tokio::spawn(async move {
                let articles =
                    parse_content(&source, request_content(&source).await.unwrap()).unwrap();
                let mut content = content.write().unwrap();
                for article in articles {
                    content.insert(article);
                }
            })
        })
        .collect();
    // Waits for each thread to finish
    for handle in handles {
        handle.await.unwrap();
    }
}

pub fn update_thread(
    config: &Config,
    pool: &Arc<SqlitePool>,
    content: &Arc<RwLock<BTreeSet<Article>>>,
) {
    if !config.sources.is_empty() {
        let update_interval = config.update_interval;
        let sources: Vec<Arc<String>> = config.sources.iter().map(|x| Arc::clone(x)).collect();
        let content = Arc::clone(content);
        let pool = Arc::clone(pool);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(update_interval));
            loop {
                interval.tick().await;
                update_content(&sources, &content).await;
                update_cache(&pool, &content)
            }
        });
    }
}

pub fn update_cache(pool: &Arc<SqlitePool>, content: &Arc<RwLock<BTreeSet<Article>>>) {
    let content = Arc::clone(content);
    let pool = Arc::clone(pool);
    tokio::spawn(async move {
        let articles: Vec<Article>;
        {
            articles = content.read().unwrap().iter().cloned().collect();
        }
        for article in articles {
            crate::database::insert_article(&pool, &article)
                .await
                .unwrap();
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
