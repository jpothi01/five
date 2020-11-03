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

fn get_file_tree_node(find_output: &FindOutput) -> Result<FileTreeNode, IndexError> {
    Err(IndexError::new("todo"))
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
    println!("{:?}", find_output);

    if let Some(root_find_output) = find_output.first() {
        Ok(Index::new(get_file_tree_node(root_find_output)?))
    } else {
        Err(IndexError::new("No results found for directory"))
    }
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
