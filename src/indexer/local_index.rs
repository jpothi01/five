use crate::indexer::index::*;
use std::fs::read_dir;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

struct BackgroundThreadState {
    cwd: PathBuf,
    index: Arc<Mutex<Option<Index>>>,
}

fn get_node_for_dir(dir: &Path) -> Result<FileTreeNode, IndexError> {
    let mut children: Vec<FileTreeNode> = Vec::new();
    for entry in read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let path = entry.path();
        if metadata.is_dir() {
            children.push(get_node_for_dir(path.as_path())?);
            continue;
        }

        let file_name = match path.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => Some(String::from(file_name)),
                None => None,
            },
            None => None,
        };
        if file_name.is_none() {
            return Err(IndexError::new("Could not get file name"));
        }
        let normalized_filename = file_name.as_ref().unwrap().to_lowercase();
        let file_index_entry = FileIndexEntry {
            path: String::from(path.to_str().unwrap()),
            file_name: file_name.unwrap(),
            normalized_filename: normalized_filename,
        };
        children.push(FileTreeNode::File(file_index_entry));
    }
    Ok(FileTreeNode::Folder(FileTreeFolder {
        children: children,
        folder_name: String::from(dir.file_name().unwrap().to_str().unwrap()),
    }))
}

impl BackgroundThreadState {
    fn run(&mut self) {
        let cwd = self.cwd.clone();
        let initial_dir = Path::new(&cwd);
        let root_node = get_node_for_dir(initial_dir).expect("Could not index!");
        match self.index.lock() {
            Err(_) => {}
            Ok(mut index) => {
                mem::replace(index.deref_mut(), Some(Index::new(root_node)));
            }
        }
    }
}

pub struct LocalIndexer {
    thread: thread::JoinHandle<()>,
    index: Arc<Mutex<Option<Index>>>,
}

impl LocalIndexer {
    pub fn new(cwd: PathBuf) -> LocalIndexer {
        let index = Arc::new(Mutex::new(None));
        let mut background_thread_state = BackgroundThreadState {
            cwd: cwd,
            index: Arc::clone(&index),
        };
        LocalIndexer {
            thread: thread::spawn(move || background_thread_state.run()),
            index: index,
        }
    }
}

impl Indexer for LocalIndexer {
    fn get_index(&self) -> Option<Index> {
        match self.index.lock() {
            Err(_) => None,
            Ok(index) => index.deref().clone(),
        }
    }
}
