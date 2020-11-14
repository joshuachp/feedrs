use reqwest;
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio::sync::mpsc;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;

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

fn input_thread(inputs: &Arc<Mutex<VecDeque<Key>>>) {
    let stdin = io::stdin();
    let inputs = Arc::clone(inputs);
    tokio::spawn(async move {
        loop {
            // Wait for some time to get input
            let stdin = stdin.lock();
            let keys = stdin.keys();
            for key in keys {
                if let Ok(key) = key {
                    let mut inputs = inputs.lock().unwrap();
                    inputs.push_back(key);

                    if key == Key::Char('q') {
                        return;
                    }
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Draws the area every 50 milliseconds
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));

    // FIFO of the inputs
    let inputs = Arc::new(Mutex::new(VecDeque::<Key>::new()));
    // Flag for closing all threads

    // Starts user input thread
    input_thread(&inputs);

    terminal.clear()?;
    loop {
        // Drawing tick
        interval.tick().await;

        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Feed").borders(Borders::ALL);
            f.render_widget(block, size);
        })?;

        let mut inputs = inputs.lock().unwrap();
        for key in inputs.drain(..) {
            match key {
                Key::Char('q') => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::request_content;

    #[tokio::test]
    async fn test_request_content() {
        request_content("https://joshuacho.github.io/index.xml")
            .await
            .unwrap();
    }
}
