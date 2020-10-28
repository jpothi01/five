/*
    Copyright 2020, John Pothier
    This file is part of Five.

    Five is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Five is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Five.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::error::Error;
use std::fs::read_dir;
use std::fs::Metadata;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct FileIndexEntry {
    pub metadata: Metadata,
    pub path: PathBuf,
    pub file_name: String,
    pub normalized_filename: String,
}

#[derive(Clone)]
pub struct FileTreeFolder {
    pub children: Vec<FileTreeNode>,
    pub folder_name: String,
}

#[derive(Clone)]
pub enum FileTreeNode {
    File(FileIndexEntry),
    Folder(FileTreeFolder),
}

impl FileTreeNode {
    fn all_files(&self) -> Vec<FileIndexEntry> {
        match self {
            FileTreeNode::File(file_index_entry) => vec![file_index_entry.clone()],
            FileTreeNode::Folder(file_tree_folder) => file_tree_folder
                .children
                .iter()
                .flat_map(|child| child.all_files())
                .collect(),
        }
    }
}

#[derive(Clone)]
pub struct Index {
    pub tree: FileTreeNode,
    pub files: Vec<FileIndexEntry>,
}

impl Index {
    fn new(root_node: FileTreeNode) -> Index {
        let files = root_node.all_files();
        Index {
            tree: root_node,
            files: files,
        }
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
            metadata: metadata,
            path: path,
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

pub struct Indexer {
    thread: thread::JoinHandle<()>,
    index: Arc<Mutex<Option<Index>>>,
}

impl Indexer {
    pub fn new(cwd: &str) -> Indexer {
        let index = Arc::new(Mutex::new(None));
        let mut background_thread_state = BackgroundThreadState {
            cwd: String::from(cwd),
            index: Arc::clone(&index),
        };
        Indexer {
            thread: thread::spawn(move || background_thread_state.run()),
            index: index,
        }
    }

    pub fn get_index(&self) -> Option<Index> {
        match self.index.lock() {
            Err(_) => None,
            Ok(index) => index.deref().clone(),
        }
    }
}
