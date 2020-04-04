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
        if state.is_done() {
            break;
        }

        terminal.draw(|mut f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Min(3),
                        Constraint::Percentage(25),
                        Constraint::Percentage(75),
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

            let metadata_items = state.metadata()
                .map(|path| Text::raw(path));
            List::new(metadata_items)
                .block(
                    Block::default()
                        .title("Metadata")
                        .borders(Borders::ALL),
                )
                .render(&mut f, chunks[1]);

            let stats_items = vec![
                Text::Raw(format!("Remaining: {}", state.paths().count()).into()),
            ];
            List::new(stats_items.into_iter())
                .block(
                    Block::default()
                        .title("Stats")
                        .borders(Borders::ALL),
                )
                .render(&mut f, chunks[0]);
        })?;

        match events.next()? {
            events::Event::Input(key) => match key {
                Key::Esc => break,
                Key::Char('w') => state.write()?,
                Key::Char(' ') => state.toggle_pause(),
                Key::Char('1') => state.rate(state::Rating::R01)?,
                Key::Char('2') => state.rate(state::Rating::R02)?,
                Key::Char('4') => state.rate(state::Rating::R04)?,
                Key::Char('6') => state.rate(state::Rating::R06)?,
                Key::Char('8') => state.rate(state::Rating::R08)?,
                Key::Char('0') => state.rate(state::Rating::R10)?,
                Key::Ctrl('l') => terminal.clear()?,
                _ => (),
            },
            events::Event::Tick => (),
        }
    }

    terminal.show_cursor()?;
    state.write()?;

    Ok(())
}
