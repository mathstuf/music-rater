use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

const TICK_RATE: Duration = Duration::from_millis(250);

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    _input_handle: thread::JoinHandle<()>,
    _tick_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let _input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for key in stdin.keys().flatten() {
                    if tx.send(Event::Input(key)).is_err() {
                        return;
                    }

                    if key == Key::Esc {
                        return;
                    }
                }
            })
        };

        let _tick_handle = {
            thread::spawn(move || {
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(TICK_RATE);
                }
            })
        };

        Self {
            rx,
            _input_handle,
            _tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
