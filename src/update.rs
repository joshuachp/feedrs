use sqlx::SqlitePool;
use std::{
    collections::{HashMap},
    sync::{Arc, RwLock},
};
use tokio::{
    sync::mpsc,
    time::{interval, Duration},
};

use crate::{
    configuration::Config,
    content::{parse_content, Article, ArticleMap},
};

async fn request_content(url: &str) -> reqwest::Result<String> {
    // TODO: Check status and show errors
    reqwest::get(url).await?.text().await
}

/// Asynchronously retrieves the content from the sources in the config
async fn get_content(sources: &[Arc<String>]) -> HashMap<(String, String), Article> {
    // Set of the new articles
    let mut result: HashMap<(String, String), Article> = HashMap::new();
    // Channel for retrieving the parsed articles
    let (sender, mut receiver) = mpsc::channel::<Article>(sources.len());
    // Spawns update threads
    sources.iter().for_each(|source| {
        let source = Arc::clone(source);
        let sender = sender.clone();
        tokio::spawn(async move {
            let articles = parse_content(&source, request_content(&source).await.unwrap()).unwrap();
            for article in articles {
                sender.send(article).await.unwrap();
            }
        });
    });
    drop(sender);
    // Waits to receive the result for each thread
    while let Some(res) = receiver.recv().await {
        result.insert((res.id.clone(), res.source.clone()), res);
    }
    result
}

pub fn update_thread(config: &Config, pool: &Arc<SqlitePool>, content: &Arc<RwLock<ArticleMap>>) {
    if !config.sources.is_empty() {
        let update_interval = config.update_interval;
        let sources: Vec<Arc<String>> = config.sources.iter().map(|x| Arc::clone(x)).collect();
        let content_c = Arc::clone(content);
        let pool = Arc::clone(pool);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(update_interval));
            loop {
                interval.tick().await;
                let content_update = get_content(&sources).await;
                let deleted_content;
                {
                    let mut content = content_c.write().unwrap();
                    deleted_content = content.update_content(&content_update);
                }
                update_cache(&pool, content_update.values().cloned().collect(), deleted_content.keys().cloned().collect());
            }
        });
    }
}

/// It will invalidate every element in the database and then insert the new content with the new
/// data. Then delete the content not found in the update.
// TODO: This can be improved by deleting only the content with a time stamp or inserted some time
// ago.
pub fn update_cache(pool: &Arc<SqlitePool>, content_update: Vec<Article>, deleted_content: Vec<(String, String)>) {
    let pool = Arc::clone(pool);
    tokio::spawn(async move {
        crate::database::insert_articles(&pool, &content_update).await.unwrap();
        crate::database::delete_articles(&pool, &deleted_content).await.unwrap();
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
