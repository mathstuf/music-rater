use std::fs;
use std::io::{self, BufReader, Cursor};
use std::path::{Path, PathBuf};

use lewton::inside_ogg;
use lewton::VorbisError;
use ogg::reading::PacketReader;
use rodio::decoder::{Decoder, DecoderError};
use rodio::source::{Pausable, Repeat, Source};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use thiserror::Error;

use crate::playlist::Playlist;

const IMPORTANT_KEYS: &[&str] = &[
    "ALBUM",
    "ALBUMARTIST",
    "TITLE",
    "SUBTITLE",
    "ARTIST",
    "album",
    "albumartist",
    "title",
    "subtitle",
    "artist",
];

pub enum Rating {
    R01,
    R02,
    R04,
    R06,
    R08,
    R10,
}

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

    fn write(&self) -> io::Result<()> {
        self.rating_01.write("rating-01.m3u8")?;
        self.rating_02.write("rating-02.m3u8")?;
        self.rating_04.write("rating-04.m3u8")?;
        self.rating_06.write("rating-06.m3u8")?;
        self.rating_08.write("rating-08.m3u8")?;
        self.rating_10.write("rating-10.m3u8")?;
        self.todo.write("todo.m3u8")?;
        Ok(())
    }
}

type MetaData = Vec<String>;
type Player = Pausable<Repeat<Decoder<BufReader<Box<Cursor<Vec<u8>>>>>>>;

pub struct State {
    playlists: Playlists,
    metadata: Option<MetaData>,
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("no device found")]
    NoDevice {
        #[from]
        source: rodio::StreamError,
    },
    #[error("sink could not be created")]
    SinkCreation {
        #[from]
        source: rodio::PlayError,
    },
    #[error("i/o error")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("decoder error")]
    Decoder {
        #[from]
        source: DecoderError,
    },
    #[error("vorbis error")]
    Vorbis {
        #[from]
        source: VorbisError,
    },
}

type StateResult<T> = Result<T, StateError>;

impl State {
    pub fn new() -> StateResult<Self> {
        let playlists = Playlists::new()?;
        let source = playlists
            .todo
            .next()
            .map(Self::prepare_path)
            .transpose()?;
        let (_stream, handle) = rodio::OutputStream::try_default()?;
        let sink = Sink::try_new(&handle)?;

        let metadata = source.map(|(player, metadata)| {
            sink.append(player);
            metadata
        });

        Ok(Self {
            playlists,
            metadata,
            _stream,
            handle,
            sink,
        })
    }

    pub fn is_done(&self) -> bool {
        self.metadata.is_none()
    }

    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.playlists.todo.paths()
    }

    pub fn metadata(&self) -> impl Iterator<Item = &String> {
        self.metadata
            .iter()
            .flatten()
    }

    fn prepare_path(path: &Path) -> StateResult<(Player, MetaData)> {
        let data = fs::read(path)?;

        // Read the metadata.
        let reader = Cursor::new(&data);
        let reader = BufReader::new(reader);
        let mut reader = PacketReader::new(reader);
        let (headers, _) = inside_ogg::read_headers(&mut reader)?;
        let (_, comments, _) = headers;
        let metadata = comments
            .comment_list
            .into_iter()
            .filter(|(key, _)| IMPORTANT_KEYS.iter().any(|&ikey| ikey == key))
            .map(|(key, value)| format!("{:<30}: {}", key, value))
            .collect();

        // Set up the decoder.
        let reader = Box::new(Cursor::new(data));
        let reader = BufReader::new(reader);
        let decoder = Decoder::new(reader)?;

        let player = decoder.repeat_infinite().pausable(false);

        Ok((player, metadata))
    }

    pub fn toggle_pause(&self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    fn update_metadata(&mut self) -> StateResult<()> {
        self.sink = Sink::try_new(&self.handle)?;
        let source = self
            .playlists
            .todo
            .next()
            .map(Self::prepare_path)
            .transpose()?;
        self.metadata = source.map(|(player, metadata)| {
            self.sink.append(player);
            metadata
        });

        Ok(())
    }

    pub fn rate(&mut self, rating: Rating) -> StateResult<()> {
        let item = self.playlists.todo.pop();
        self.update_metadata()?;

        if let Some(item) = item {
            let output = match rating {
                Rating::R01 => &mut self.playlists.rating_01,
                Rating::R02 => &mut self.playlists.rating_02,
                Rating::R04 => &mut self.playlists.rating_04,
                Rating::R06 => &mut self.playlists.rating_06,
                Rating::R08 => &mut self.playlists.rating_08,
                Rating::R10 => &mut self.playlists.rating_10,
            };

            output.push(item);
        }

        Ok(())
    }

    pub fn write(&self) -> StateResult<()> {
        Ok(self.playlists.write()?)
    }
}
