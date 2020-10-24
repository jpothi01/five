use std::error::Error;
use std::fs::read_dir;
use std::fs::Metadata;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct FileIndexEntry {
    metadata: Metadata,
    path: PathBuf,
}

#[derive(Clone)]
pub struct Index {
    pub files: Vec<FileIndexEntry>,
}

impl Index {
    fn new() -> Index {
        Index { files: vec![] }
    }
}

#[derive(Debug)]
pub struct IndexError {
    pub message: String,
}

impl IndexError {
    pub fn new(message: &str) -> IndexError {
        IndexError {
            message: String::from(message),
        }
    }
}

pub type IndexResult<T> = Result<T, IndexError>;

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error indexing: {}", self.message)
    }
}

impl From<std::io::Error> for IndexError {
    fn from(error: std::io::Error) -> Self {
        IndexError::new(error.description())
    }
}

struct BackgroundThreadState {
    cwd: String,
    index: Arc<Mutex<Index>>,
}

impl BackgroundThreadState {
    fn append_all_files_in_dir(&mut self, dir: &Path) -> Result<(), IndexError> {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let path = entry.path();
            if metadata.is_dir() {
                self.append_all_files_in_dir(path.as_path())?;
                continue;
            }

            match self.index.lock() {
                Err(_) => return Err(IndexError::new("Could not acquire lock")),
                Ok(mut index) => {
                    index.deref_mut().files.push(FileIndexEntry {
                        metadata: metadata,
                        path: path,
                    });
                }
            };
        }
        Ok(())
    }

    fn run(&mut self) {
        let cwd = self.cwd.clone();
        let initial_dir = Path::new(&cwd);
        self.append_all_files_in_dir(initial_dir)
            .expect("Could not index!");
    }
}

pub struct Indexer {
    thread: thread::JoinHandle<()>,
    index: Arc<Mutex<Index>>,
}

impl Indexer {
    pub fn new(cwd: &str) -> Indexer {
        let index = Arc::new(Mutex::new(Index::new()));
        let mut background_thread_state = BackgroundThreadState {
            cwd: String::from(cwd),
            index: Arc::clone(&index),
        };
        Indexer {
            thread: thread::spawn(move || background_thread_state.run()),
            index: index,
        }
    }

    pub fn get_index(&self) -> IndexResult<Index> {
        match self.index.lock() {
            Err(_) => Err(IndexError::new("Could not acquire lock")),
            Ok(index) => Ok(index.deref().clone()),
        }
    }
}
