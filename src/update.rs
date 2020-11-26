use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use crate::configuration::Config;

async fn request_content(url: &str) -> reqwest::Result<String> {
    // TODO: Check status and show errors
    reqwest::get(url).await?.text().await
}

async fn update_content(sources: &Vec<Arc<String>>) -> Option<Vec<String>> {
    // Update only if there is a source to update from
    if sources.len() > 0 {
        let (sender, mut receiver) = mpsc::channel(sources.len());
        let mut content_vec: Vec<String> = Vec::with_capacity(sources.len());

        for source in sources {
            let mut sender = sender.clone();
            let source = Arc::clone(source);
            tokio::spawn(async move {
                let content = request_content(&source).await.unwrap();
                sender.send(content).await.unwrap();
            });
        }
        drop(sender);

        while let Some(content) = receiver.recv().await {
            content_vec.push(content)
        }
        return Some(content_vec);
    }
    None
}

fn update_thread(config: &Config) {
    let update_interval = config.update_interval;
    let sources: Vec<Arc<String>> = config
        .sources
        .iter()
        .cloned()
        .map(|x| Arc::new(x))
        .collect();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(update_interval));

        loop {
            interval.tick().await;

            update_content(&sources).await;
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
