use std::io;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;

fn main() -> io::Result<()> {
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
