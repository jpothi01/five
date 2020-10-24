use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::Thread;

#[derive(Clone)]
pub struct Index {
    pub files: Vec<PathBuf>,
}

impl Index {
    fn new() -> Index {
        Index { files: vec![] }
    }
}

struct BackgroundThreadState {
    mutex: Arc<Mutex<u32>>,
    index: Index,
}

impl BackgroundThreadState {
    fn run(&mut self) {}
}

pub struct Indexer {
    thread: thread::JoinHandle<()>,
    mutex: Arc<Mutex<u32>>,
    index: Index,
}

impl Indexer {
    pub fn new() -> Indexer {
        let mutex = Arc::new(Mutex::new(0));
        let mut background_thread_state = BackgroundThreadState {
            mutex: Arc::clone(&mutex),
            index: Index::new(),
        };
        Indexer {
            thread: thread::spawn(move || background_thread_state.run()),
            mutex: mutex,
            index: Index::new(),
        }
    }

    pub fn get_index(&self) -> Index {
        Index::new()
    }
}
