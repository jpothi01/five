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

#[derive(Debug, Clone)]
pub struct FileIndexEntry {
    pub path: String,
    pub file_name: String,
    pub normalized_filename: String,
}

#[derive(Debug, Clone)]
pub struct FileTreeFolder {
    pub children: Vec<FileTreeNode>,
    pub folder_name: String,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Index {
    pub tree: FileTreeNode,
    pub files: Vec<FileIndexEntry>,
}

impl Index {
    pub fn new(root_node: FileTreeNode) -> Index {
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

pub trait Indexer {
    fn get_index(&self) -> Option<Index>;
}
