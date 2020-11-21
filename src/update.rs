use tokio::sync::mpsc;

async fn request_content(url: &str) -> reqwest::Result<String> {
    // TODO: Check status and show errors
    reqwest::get(url).await?.text().await
}

async fn update_content(sources: Vec<String>) -> Option<Vec<String>> {
    // Update only if there is a source to update from
    if sources.len() > 0 {
        let (sender, mut receiver) = mpsc::channel(sources.len());
        let mut content_vec: Vec<String> = Vec::with_capacity(sources.len());

        for source in sources {
            let mut sender = sender.clone();
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
