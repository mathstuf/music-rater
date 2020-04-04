use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::iter;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use itertools::Itertools;

pub struct Playlist {
    paths: VecDeque<PathBuf>,
}

impl Playlist {
    pub fn from_path<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let paths = fs::read(path)?
            .split(|&ch| ch == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| {
                OsStr::from_bytes(line).to_os_string().into()
            })
            .collect();

        Ok(Playlist {
            paths,
        })
    }

    pub fn write<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let content = if self.paths.is_empty() {
            Vec::new()
        } else {
            self.paths
                .iter()
                .map(|path| path.as_os_str().as_bytes())
                .intersperse(&[b'\n'])
                .flatten()
                .cloned()
                .chain(iter::once(b'\n'))
                .collect()
        };

        fs::write(path, &content)
    }

    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.paths.iter()
    }
}
