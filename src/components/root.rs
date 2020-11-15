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

use crate::buffer::Buffer;
use crate::components::component::Component;
use crate::components::divider::DividerComponent;
use crate::components::file_pane::FilePaneComponent;
use crate::components::file_view::{FileViewComponent, FileViewContent};
use crate::event::Event;
use crate::indexer::index::Indexer;
use crate::indexer::index::{FileIndexEntry, FileTreeFolder, FileTreeNode};
use crate::terminal::Rect;
use std::io::Write;
use std::path::Path;
use termion::event::Key;

enum FocusedComponent {
    FilePane,
    FileView,
}

pub struct RootComponent<'a> {
    indexer: &'a dyn Indexer,
    file_pane: FilePaneComponent,
    file_view: FileViewComponent,
    divider: DividerComponent,
    focused_component: FocusedComponent,
}

impl<'a> RootComponent<'a> {
    pub fn new(indexer: &dyn Indexer) -> RootComponent {
        RootComponent {
            indexer: indexer,
            file_pane: FilePaneComponent::new(),
            file_view: FileViewComponent::new(),
            divider: DividerComponent::new(),
            focused_component: FocusedComponent::FilePane,
        }
    }

    pub fn update_index(&mut self) {
        match self.indexer.get_index() {
            None => println!("Could not get index"),
            Some(index) => {
                // TODO: somehow make this event based
                self.file_pane.update_index(index)
            }
        }
    }

    fn start_quick_open(&mut self) {
        self.update_index();
        self.file_pane.start_quick_open();
    }

    fn show_file_preview(&mut self, index_entry: &FileIndexEntry) {
        let path = Path::new(&index_entry.path);
        match std::fs::read_to_string(&path) {
            Err(_) => {
                // TODO: smart error handling for non-utf-8 strings
                self.file_view
                    .set_content(FileViewContent::BinaryFile(String::from(
                        path.to_str().unwrap(),
                    )));
            }
            Ok(content) => {
                self.file_view.set_content(FileViewContent::TextFile(
                    String::from(path.to_str().unwrap()),
                    content,
                ));
            }
        }
    }

    fn show_folder_preview(&mut self, folder: &FileTreeFolder) {
        let children: Vec<String> = folder
            .children
            .iter()
            .map(|child| match &child {
                FileTreeNode::File(file_index_entry) => file_index_entry.file_name.clone(),
                FileTreeNode::Folder(file_tree_folder) => file_tree_folder.folder_name.clone(),
            })
            .collect();
        self.file_view
            .set_content(FileViewContent::Folder(folder.path.clone(), children))
    }

    fn open_file(&mut self, index_entry: &FileIndexEntry) {
        self.show_file_preview(index_entry);
        self.focused_component = FocusedComponent::FileView;
        self.file_view.set_has_focus(true);
    }

    fn save_file(&self, buffer: &Buffer, file_path: String) {
        let (left, right) = buffer.get();
        let maybe_handle = std::fs::OpenOptions::new().write(true).open(&file_path);
        if let Err(err) = &maybe_handle {
            println!("Error saving: {}", err);
        }

        let mut handle = maybe_handle.unwrap();
        if let Err(err) = handle.write(left.as_bytes()) {
            // TODO: handle saving errors
            println!("Error saving: {}", err);
            return;
        }

        if let Err(err) = handle.write(right.as_bytes()) {
            // TODO: handle saving errors
            println!("Error saving: {}", err);
            return;
        }
    }
}

impl<'a> Component for RootComponent<'a> {
    fn needs_paint(&self) -> bool {
        self.file_view.needs_paint() || self.file_pane.needs_paint() || self.divider.needs_paint()
    }
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        let margin = 1;
        let file_pane_rect = Rect {
            left: 1,
            top: 1,
            width: 32,
            height: rect.height - margin,
        };
        let divider_rect = Rect {
            left: file_pane_rect.width + 1 + margin,
            top: 1,
            width: 1,
            height: rect.height,
        };
        let file_view_rect = Rect {
            left: file_pane_rect.width + 1 + divider_rect.width + 2 * margin,
            top: 1,
            width: rect.width - file_pane_rect.width - 2 * margin,
            height: rect.height - margin,
        };
        if self.file_pane.needs_paint() {
            self.file_pane.paint(stream, file_pane_rect)?;
        }
        if self.file_view.needs_paint() {
            self.file_view.paint(stream, file_view_rect)?;
        }
        if self.divider.needs_paint() {
            self.divider.paint(stream, divider_rect)?;
        }

        stream.flush()
    }

    fn dispatch_event(&mut self, event: termion::event::Event) -> bool {
        // Swap dispatch priority depending on focus
        match self.focused_component {
            FocusedComponent::FilePane => {
                if self.file_pane.dispatch_event(event.clone()) {
                    return true;
                }
                if self.file_view.dispatch_event(event.clone()) {
                    return true;
                }
            }
            FocusedComponent::FileView => {
                if self.file_view.dispatch_event(event.clone()) {
                    return true;
                }
                if self.file_pane.dispatch_event(event.clone()) {
                    return true;
                }
            }
        }

        match event {
            termion::event::Event::Key(key) => match key {
                Key::Ctrl(c) => {
                    if c == 'p' {
                        self.start_quick_open();
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },

            _ => false,
        }
    }

    fn get_events(&self) -> Vec<Event> {
        let mut result: Vec<Event> = Vec::new();
        let mut temp = self.file_pane.get_events();
        result.append(&mut temp);
        temp = self.file_view.get_events();
        result.append(&mut temp);
        temp = self.divider.get_events();
        result.append(&mut temp);
        result
    }

    fn dispatch_events(&mut self, events: &[Event]) {
        for event in events {
            match event {
                Event::FileItemSelected(file_tree_node) => match file_tree_node {
                    FileTreeNode::File(file_index_entry) => {
                        self.show_file_preview(file_index_entry)
                    }
                    FileTreeNode::Folder(file_tree_folder) => {
                        self.show_folder_preview(file_tree_folder)
                    }
                },
                Event::FileItemOpened(index_entry) => self.open_file(index_entry),
                Event::FileViewLostFocus => {
                    self.file_view.set_has_focus(false);
                    self.focused_component = FocusedComponent::FilePane;
                }
                Event::FileSaved => {
                    let (buffer, file_path) = self.file_view.get_buffer();
                    self.save_file(buffer, file_path);
                }
            }
        }

        self.file_pane.dispatch_events(events);
        self.file_view.dispatch_events(events);
        self.divider.dispatch_events(events);
    }
}
