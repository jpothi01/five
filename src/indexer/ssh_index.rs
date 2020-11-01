use crate::indexer::index::*;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

struct BackgroundThreadState {
    ssh_command: Vec<String>,
    index: Arc<Mutex<Option<Index>>>,
}

fn retrieve_index(ssh_command: &Vec<String>) -> Index {
    let mut args = ssh_command.clone();
    args.push(String::from("find /"));
    let output = Command::new("ssh").args(&args).output();
    let output_string = String::from_utf8(output.unwrap().stdout).unwrap();
    let file_paths: Vec<String> = output_string.lines().map(|s| String::from(s)).collect();
    println!("{:?}", file_paths);

    Index::new(FileTreeNode::File(FileIndexEntry {
        path: String::from("/"),
        file_name: file_paths.first().unwrap().clone(),
        normalized_filename: file_paths.first().unwrap().to_lowercase(),
    }))
}

impl BackgroundThreadState {
    fn run(&mut self) {
        let retrieved_index = retrieve_index(&self.ssh_command);
        match self.index.lock() {
            Err(_) => {}
            Ok(mut index) => {
                mem::replace(index.deref_mut(), Some(retrieved_index));
            }
        }
    }
}

pub struct SshIndexer {
    thread: thread::JoinHandle<()>,
    index: Arc<Mutex<Option<Index>>>,
}

impl SshIndexer {
    pub fn new(ssh_command: Vec<String>) -> SshIndexer {
        let index = Arc::new(Mutex::new(None));
        let mut background_thread_state = BackgroundThreadState {
            ssh_command: ssh_command,
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
