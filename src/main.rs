use reqwest;
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio::sync::{mpsc, Notify};
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

fn input_thread(inputs: &Arc<Mutex<VecDeque<Key>>>, notify: &Arc<Notify>) {
    let inputs = inputs.clone();
    let notify = notify.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        let stdin = io::stdin();

        loop {
            println!("Test");

            // Wait for some time to get input
            interval.tick().await;

            let stdin = stdin.lock();
            let keys: Vec<Key> = stdin.keys().map(|key| key.unwrap()).collect();

            if !keys.is_empty() {
                let mut inputs = inputs.lock().unwrap();
                inputs.extend(keys);

                // Notify main thread
                notify.notify();
            }
        }
    });
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut run = true;

    // FIFO of the inputs
    let inputs = Arc::new(Mutex::new(VecDeque::<Key>::new()));
    // Notifies the main thread when to redraw
    let notify = Arc::new(Notify::new());

    // Starts input thread
    input_thread(&inputs, &notify);

    while run {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, size);
        })?;

        notify.notified().await;

        let mut inputs = inputs.lock().unwrap();
        for key in inputs.drain(..) {
            match key {
                Key::Char('q') => {
                    run = false;
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
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
