use std::error::Error;
use std::io;

use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, Text, Widget};

mod events;
mod playlist;
mod state;

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = events::Events::new();
    let mut state = state::State::new()?;

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Max(3),
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ]
                    .as_ref()
                )
                .split(size);

            let playlist_items = state.paths()
                .map(|path| Text::raw(path.to_string_lossy()));
            List::new(playlist_items)
                .block(
                    Block::default()
                        .title("Playlist")
                        .borders(Borders::ALL),
                )
                .render(&mut f, chunks[2]);
        })?;

        match events.next()? {
            events::Event::Input(key) => match key {
                Key::Esc => break,
                _ => (),
            },
            events::Event::Tick => (),
        }
    }

    terminal.show_cursor()?;

    Ok(())
}
