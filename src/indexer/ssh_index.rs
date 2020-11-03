use crate::indexer::index::*;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct SshConfig {
    ssh_args: Vec<String>,
    directory_path: PathBuf,
}

impl SshConfig {
    pub fn new(ssh_args: Vec<String>) -> SshConfig {
        // Very stupid for now
        let user_and_host_and_path = &ssh_args[0];
        let mut split = user_and_host_and_path.split(':');
        let user_and_host = split.nth(0);
        if user_and_host.is_none() {
            panic!("Developer should do proper error handling here");
        }
        let path = split.nth(0);
        let mut munged_ssh_args = ssh_args.clone();
        munged_ssh_args[0] = String::from(user_and_host.unwrap());
        let directory_path = match path {
            None => PathBuf::from("/"),
            Some(path) => PathBuf::from(path),
        };
        SshConfig {
            ssh_args: munged_ssh_args,
            directory_path,
        }
    }
}

#[derive(Debug)]
enum FindOutput {
    File(PathBuf),
    Folder(PathBuf),
}

fn parse_find_line(line: &str) -> Option<FindOutput> {
    // Example:
    //    658744      4 -rw-r--r--   1 root     root          148 Aug 17  2015 ./.profile
    const file_attributes_column_index: usize = 2;

    let mut columns = line.split_ascii_whitespace();

    let file_attributes_column = columns.nth(file_attributes_column_index);
    if file_attributes_column.is_none() {
        return None;
    }

    let path_column = columns.last();
    if path_column.is_none() {
        return None;
    }

    let path_string = path_column.unwrap();
    let path = PathBuf::from(path_string);

    match file_attributes_column.unwrap().chars().nth(0) {
        None => return None,
        Some(c) => match c {
            'd' => Some(FindOutput::Folder(path)),
            '-' => Some(FindOutput::File(path)),
            _ => None,
        },
    }
}

// Assumption: When we encounter a directory in the FindOutput list, all following entries are the
// directory's contents until all its contents have been listed.
fn get_file_tree_node(
    find_output: &[FindOutput],
) -> Result<(&[FindOutput], FileTreeNode), IndexError> {
    let current_item = find_output.get(0);
    if current_item.is_none() {
        return Err(IndexError::new("No directory found"));
    }

    match current_item.unwrap() {
        FindOutput::Folder(path) => {
            let folder_path = if let Some(folder_path) = path.to_str() {
                folder_path
            } else {
                return Err(IndexError::new("Could not get folder path"));
            };

            let folder_name = if let Some(folder_name) = path.file_name() {
                if let Some(folder_name_string) = folder_name.to_str() {
                    String::from(folder_name_string)
                } else {
                    return Err(IndexError::new("Could not convert folder name"));
                }
            } else {
                return Err(IndexError::new("Could not get folder name"));
            };

            let mut children: Vec<FileTreeNode> = Vec::new();
            let mut next_slice = &find_output[1..];

            let should_continue = |slice: &[FindOutput]| match slice.first() {
                None => false,
                Some(find_output) => match find_output {
                    FindOutput::File(_) => true,
                    FindOutput::Folder(path) => path.starts_with(folder_path),
                },
            };

            while should_continue(next_slice) {
                let (next_slice_, child_node) = get_file_tree_node(next_slice)?;
                next_slice = next_slice_;
                children.push(child_node);
            }

            let node = FileTreeNode::Folder(FileTreeFolder {
                children,
                folder_name,
                path: String::from(folder_path),
            });
            Ok((next_slice, node))
        }
        FindOutput::File(path) => match FileIndexEntry::new(path) {
            Some(file_index_entry) => Ok((&find_output[1..], FileTreeNode::File(file_index_entry))),
            None => Err(IndexError::new("Could not create FileIndexEntry")),
        },
    }
}

fn retrieve_index(config: &SshConfig) -> Result<Index, IndexError> {
    let mut args = config.ssh_args.clone();
    args.push(format!(
        "find {} -ls",
        config.directory_path.to_str().unwrap()
    ));
    let output = Command::new("ssh").args(&args).output();
    let output_string = String::from_utf8(output.unwrap().stdout).unwrap();
    let find_output: Vec<FindOutput> = output_string.lines().filter_map(parse_find_line).collect();

    Ok(Index::new(get_file_tree_node(&find_output)?.1))
}

struct BackgroundThreadState {
    config: SshConfig,
    index: Arc<Mutex<Option<Index>>>,
}

impl BackgroundThreadState {
    fn run(&mut self) {
        let retrieved_index = retrieve_index(&self.config);
        if retrieved_index.is_err() {
            println!("Could not retrieve index: {}", retrieved_index.unwrap_err());
            return;
        }

        match self.index.lock() {
            Err(_) => {}
            Ok(mut index) => {
                mem::replace(index.deref_mut(), Some(retrieved_index.unwrap()));
            }
        }
    }
}

pub struct SshIndexer {
    thread: thread::JoinHandle<()>,
    index: Arc<Mutex<Option<Index>>>,
}

impl SshIndexer {
    pub fn new(config: SshConfig) -> SshIndexer {
        let index = Arc::new(Mutex::new(None));
        let mut background_thread_state = BackgroundThreadState {
            config: config,
            index: Arc::clone(&index),
        };
        SshIndexer {
            thread: thread::spawn(move || background_thread_state.run()),
            index: index,
        }
    }
}

impl Indexer for SshIndexer {
    fn get_index(&self) -> Option<Index> {
        match self.index.lock() {
            Err(_) => None,
            Ok(index) => index.deref().clone(),
        }
    }
}
