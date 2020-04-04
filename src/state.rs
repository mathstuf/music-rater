use std::io;
use std::path::PathBuf;

use crate::playlist::Playlist;

struct Playlists {
    todo: Playlist,
    rating_01: Playlist,
    rating_02: Playlist,
    rating_04: Playlist,
    rating_06: Playlist,
    rating_08: Playlist,
    rating_10: Playlist,
}

impl Playlists {
    fn new() -> io::Result<Self> {
        Ok(Self {
            todo: Playlist::from_path("todo.m3u8")?,
            rating_01: Playlist::from_path("rating-01.m3u8")?,
            rating_02: Playlist::from_path("rating-02.m3u8")?,
            rating_04: Playlist::from_path("rating-04.m3u8")?,
            rating_06: Playlist::from_path("rating-06.m3u8")?,
            rating_08: Playlist::from_path("rating-08.m3u8")?,
            rating_10: Playlist::from_path("rating-10.m3u8")?,
        })
    }
}

pub struct State {
    playlists: Playlists,
}

impl State {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            playlists: Playlists::new()?,
        })
    }

    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.playlists.todo.paths()
    }
}
