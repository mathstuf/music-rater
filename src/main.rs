use std::error::Error;
use std::io;

use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem};

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

        terminal.draw(|f| {
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
                .map(|path| ListItem::new(Text::raw(path.to_string_lossy())))
                .collect::<Vec<_>>();
            let playlist = List::new(playlist_items)
                .block(
                    Block::default()
                        .title("Playlist")
                        .borders(Borders::ALL),
                );
            f.render_widget(playlist, chunks[2]);

            let metadata_items = state.metadata()
                .map(|line| ListItem::new(Text::raw(line)))
                .collect::<Vec<_>>();
            let metadata = List::new(metadata_items)
                .block(
                    Block::default()
                        .title("Metadata")
                        .borders(Borders::ALL),
                );
            f.render_widget(metadata, chunks[1]);

            let stats_items = vec![
                ListItem::new(Text::raw(format!("Remaining: {}", state.paths().count()))),
            ];
            let stats = List::new(stats_items)
                .block(
                    Block::default()
                        .title("Stats")
                        .borders(Borders::ALL),
                );
            f.render_widget(stats, chunks[0]);
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
