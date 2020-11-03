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

use crate::components::component::Component;
use crate::event::Event;
use crate::indexer::index::{FileIndexEntry, FileTreeFolder, FileTreeNode, Index};
use crate::painting_utils::{paint_empty_lines, paint_truncated_text};
use crate::quick_open::{get_quick_open_results, QuickOpenResult};
use crate::terminal::Rect;
use std::cell::Cell;
use std::io::Write;
use termion::event::Key;

struct QuickOpenComponent {
    search_query: String,
    index: Option<Index>,
    results: Vec<QuickOpenResult>,
    selected_item_index: Option<usize>,
    events: Vec<Event>,
}

impl QuickOpenComponent {
    fn new() -> QuickOpenComponent {
        QuickOpenComponent {
            search_query: String::new(),
            index: None,
            results: vec![],
            selected_item_index: None,
            events: vec![],
        }
    }
    fn update_quick_open_results(&mut self) {
        match &self.index {
            Some(index) => {
                self.results = get_quick_open_results(&index, &self.search_query);
                if self.results.len() > 0 {
                    self.selected_item_index = Some(0)
                } else {
                    self.selected_item_index = None
                }
            }
            None => {}
        };
    }
}

impl Component for QuickOpenComponent {
    fn needs_paint(&self) -> bool {
        true
    }

    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        write!(stream, "{}", termion::cursor::Goto(rect.left, rect.top))?;
        write!(
            stream,
            "{}{}",
            termion::color::Bg(termion::color::Yellow),
            termion::color::Fg(termion::color::Black)
        )?;
        paint_truncated_text(stream, &self.search_query, rect.width)?;

        let mut row = rect.top + 1;

        for (index, result) in self.results.iter().enumerate() {
            if self.selected_item_index.is_some() && self.selected_item_index.unwrap() == index {
                write!(
                    stream,
                    "{}{}",
                    termion::color::Bg(termion::color::White),
                    termion::color::Fg(termion::color::Black)
                )?;
            } else {
                write!(
                    stream,
                    "{}{}",
                    termion::color::Fg(termion::color::White),
                    termion::color::Bg(termion::color::Reset)
                )?;
            }

            write!(stream, "{}", termion::cursor::Goto(rect.left, row))?;
            paint_truncated_text(stream, &result.file_name, rect.width)?;
            if row >= rect.height {
                break;
            }

            row += 1;
        }

        write!(
            stream,
            "{}{}",
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset)
        )?;
        paint_empty_lines(
            stream,
            Rect {
                top: row,
                left: rect.left,
                width: rect.width,
                height: rect.height - row + 1,
            },
        )?;

        Ok(())
    }

    fn dispatch_event(&mut self, event: termion::event::Event) -> bool {
        self.events.clear();
        let handled = match event {
            termion::event::Event::Key(key) => match key {
                Key::Char(c) => {
                    self.search_query.push(c);
                    self.update_quick_open_results();
                    true
                }
                Key::Backspace => {
                    self.search_query.pop();
                    self.update_quick_open_results();
                    true
                }
                Key::Down => {
                    let next_item_index = match self.selected_item_index {
                        None => 0usize,
                        Some(index) => index + 1usize,
                    };
                    if next_item_index < self.results.len() {
                        self.selected_item_index = Some(next_item_index);
                    };
                    true
                }
                Key::Up => {
                    let maybe_next_item_index = match self.selected_item_index {
                        None => None,
                        Some(index) => {
                            if index > 0 {
                                Some(index - 1usize)
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(next_item_index) = maybe_next_item_index {
                        self.selected_item_index = Some(next_item_index);
                    };
                    true
                }
                _ => false,
            },
            _ => false,
        };
        if handled {
            if let Some(selected_index) = self.selected_item_index {
                self.events.push(Event::FileItemSelected(
                    self.results[selected_index].clone(),
                ));
            }
        }
        handled
    }

    fn get_events(&self) -> Vec<Event> {
        return self.events.clone();
    }
    fn dispatch_events(&mut self, _: &[Event]) {}
}

struct FileTreeCache {
    root_node: FileTreeNode,
    node_stack: Vec<FileTreeNode>,
}

impl FileTreeCache {
    pub fn new(file_tree: &FileTreeNode) -> FileTreeCache {
        FileTreeCache {
            root_node: file_tree.clone(),
            node_stack: vec![file_tree.clone()],
        }
    }
}

struct DirectoryTreeComponent {
    selected_item_index: Option<usize>,
    needs_paint: Cell<bool>,
    file_tree_cache: Option<FileTreeCache>,
    events: Vec<Event>,
}

impl DirectoryTreeComponent {
    fn update_index(&mut self, index: Index) {
        self.needs_paint.set(true);

        match self.file_tree_cache {
            None => self.file_tree_cache = Some(FileTreeCache::new(&index.tree)),
            _ => {}
        }
    }

    fn num_current_items(&self) -> usize {
        match &self.file_tree_cache {
            None => 0,
            Some(file_tree_cache) => match file_tree_cache.node_stack.last().unwrap() {
                FileTreeNode::File(_) => 1,
                FileTreeNode::Folder(file_tree_folder) => file_tree_folder.children.len(),
            },
        }
    }

    fn file_tree_node_at_index(&self, index: usize) -> Option<&FileTreeNode> {
        match &self.file_tree_cache {
            None => None,
            Some(file_tree_cache) => match file_tree_cache.node_stack.last().unwrap() {
                FileTreeNode::File(_) => None,
                FileTreeNode::Folder(file_tree_folder) => file_tree_folder.children.get(index),
            },
        }
    }

    fn file_index_entry_at_index(&self, index: usize) -> Option<&FileIndexEntry> {
        match self.file_tree_node_at_index(index) {
            None => None,
            Some(file_tree_node) => match file_tree_node {
                FileTreeNode::Folder(_) => None,
                FileTreeNode::File(file_index_entry) => Some(file_index_entry),
            },
        }
    }

    fn open_selected_item(&mut self) {
        let next_current_node = match self.selected_item_index {
            None => None,
            Some(selected_index) => match self.file_tree_node_at_index(selected_index) {
                None => None,
                Some(file_tree_node) => match file_tree_node {
                    FileTreeNode::File(file_index_entry) => {
                        Some(FileTreeNode::File(file_index_entry.clone()))
                    }
                    FileTreeNode::Folder(file_tree_folder) => {
                        Some(FileTreeNode::Folder(file_tree_folder.clone()))
                    }
                },
            },
        };

        match next_current_node {
            None => {}
            Some(next_current_node) => {
                if let FileTreeNode::File(file_index_entry) = next_current_node {
                    self.open_file(file_index_entry);
                } else {
                    self.push_node_stack(next_current_node);
                }
            }
        }
    }

    fn open_file(&mut self, file_index_entry: FileIndexEntry) {
        self.events.push(Event::FileItemOpened(file_index_entry))
    }

    fn push_node_stack(&mut self, next_current_node: FileTreeNode) {
        if let Some(file_tree_cache) = &mut self.file_tree_cache {
            file_tree_cache.node_stack.push(next_current_node);
            self.selected_item_index = None
        }
    }

    fn pop_node_stack(&mut self) -> bool {
        if let Some(file_tree_cache) = &mut self.file_tree_cache {
            if file_tree_cache.node_stack.len() > 1 {
                file_tree_cache.node_stack.pop();
                self.selected_item_index = None;
            }

            true
        } else {
            false
        }
    }

    fn paint_directory<Writer: Write>(
        &self,
        stream: &mut Writer,
        directory: &FileTreeFolder,
        rect: Rect,
    ) -> std::io::Result<()> {
        let mut row = rect.top;
        for (index, node) in directory.children.iter().enumerate() {
            write!(stream, "{}", termion::cursor::Goto(rect.left, row))?;
            if self.selected_item_index.is_some() && self.selected_item_index.unwrap() == index {
                write!(
                    stream,
                    "{}{}",
                    termion::color::Bg(termion::color::White),
                    termion::color::Fg(termion::color::Black)
                )?;
            } else {
                match node {
                    FileTreeNode::File(_) => {
                        write!(stream, "{}", termion::color::Fg(termion::color::White))?;
                    }
                    FileTreeNode::Folder(_) => {
                        write!(stream, "{}", termion::color::Fg(termion::color::Green))?;
                    }
                }
            }
            let line = match node {
                FileTreeNode::File(file_index_entry) => &file_index_entry.file_name,
                FileTreeNode::Folder(file_tree_folder) => &file_tree_folder.folder_name,
            };
            paint_truncated_text(stream, line, rect.width)?;
            write!(
                stream,
                "{}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset)
            )?;
            row += 1;
        }
        paint_empty_lines(
            stream,
            Rect {
                top: row,
                left: rect.left,
                width: rect.width,
                height: rect.height + 1 - row,
            },
        )?;
        self.needs_paint.set(false);
        Ok(())
    }
}

impl Component for DirectoryTreeComponent {
    fn needs_paint(&self) -> bool {
        self.needs_paint.take()
    }
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        if let Some(file_tree_cache) = &self.file_tree_cache {
            assert!(file_tree_cache.node_stack.len() > 0);
            if let FileTreeNode::Folder(file_tree_folder) =
                file_tree_cache.node_stack.last().unwrap()
            {
                self.paint_directory(stream, &file_tree_folder, rect)
            } else {
                // TODO: single file support
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn dispatch_event(&mut self, event: termion::event::Event) -> bool {
        self.events.clear();
        let handled = match event {
            termion::event::Event::Key(key) => match key {
                Key::Down => {
                    let next_item_index = match self.selected_item_index {
                        None => 0usize,
                        Some(index) => index + 1usize,
                    };
                    if next_item_index < self.num_current_items() {
                        self.selected_item_index = Some(next_item_index)
                    };
                    true
                }
                Key::Up => {
                    let maybe_next_item_index = match self.selected_item_index {
                        None => None,
                        Some(index) => {
                            if index > 0 {
                                Some(index - 1usize)
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(next_item_index) = maybe_next_item_index {
                        self.selected_item_index = Some(next_item_index)
                    };
                    true
                }
                Key::Backspace => self.pop_node_stack(),
                Key::Char(c) => match c {
                    '\n' => {
                        self.open_selected_item();
                        true
                    }
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        };
        if handled {
            self.needs_paint.set(true);

            let event = if let Some(selected_index) = self.selected_item_index {
                if let Some(file_index_entry) = self.file_index_entry_at_index(selected_index) {
                    Some(Event::FileItemSelected(file_index_entry.clone()))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(event) = event {
                self.events.push(event)
            }
        };
        handled
    }

    fn get_events(&self) -> Vec<Event> {
        self.events.clone()
    }
    fn dispatch_events(&mut self, _: &[Event]) {}
}

enum FilePaneMode {
    DirectoryTree,
    QuickOpen,
}

pub struct FilePaneComponent {
    directory_tree: DirectoryTreeComponent,
    quick_open: QuickOpenComponent,
    mode: FilePaneMode,
}

impl FilePaneComponent {
    pub fn new() -> FilePaneComponent {
        FilePaneComponent {
            directory_tree: DirectoryTreeComponent {
                selected_item_index: None,
                needs_paint: Cell::new(true),
                file_tree_cache: None,
                events: Vec::new(),
            },
            quick_open: QuickOpenComponent::new(),
            mode: FilePaneMode::DirectoryTree,
        }
    }

    pub fn start_quick_open(&mut self, index: Index) {
        self.quick_open.index = Some(index);
        self.mode = FilePaneMode::QuickOpen;
    }

    pub fn update_index(&mut self, index: Index) {
        self.directory_tree.update_index(index);
    }
}

impl Component for FilePaneComponent {
    fn needs_paint(&self) -> bool {
        self.directory_tree.needs_paint() || self.quick_open.needs_paint()
    }
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        match self.mode {
            FilePaneMode::DirectoryTree => self.directory_tree.paint(stream, rect),
            FilePaneMode::QuickOpen => self.quick_open.paint(stream, rect),
        }
    }
    fn dispatch_event(&mut self, event: termion::event::Event) -> bool {
        match event {
            termion::event::Event::Key(key) => match key {
                Key::Esc => match self.mode {
                    FilePaneMode::QuickOpen => {
                        self.mode = FilePaneMode::DirectoryTree;
                        return true;
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        match self.mode {
            FilePaneMode::DirectoryTree => self.directory_tree.dispatch_event(event),
            FilePaneMode::QuickOpen => self.quick_open.dispatch_event(event),
        }
    }

    fn get_events(&self) -> Vec<Event> {
        match self.mode {
            FilePaneMode::DirectoryTree => self.directory_tree.get_events(),
            FilePaneMode::QuickOpen => self.quick_open.get_events(),
        }
    }
    fn dispatch_events(&mut self, events: &[Event]) {
        match self.mode {
            FilePaneMode::DirectoryTree => self.directory_tree.dispatch_events(events),
            FilePaneMode::QuickOpen => self.quick_open.dispatch_events(events),
        }
    }
}
