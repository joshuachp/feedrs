use reqwest;
use std::io;
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

const INPUT: u8 = 10;

async fn update_content(sources: Vec<String>) {
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
}

fn input_thread() {}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut run = true;
    while run {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, size);
        })?;

        let handle = stdin.lock();
        for key in handle.keys() {
            let c = key.unwrap();
            match c {
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
