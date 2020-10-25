use crate::components::component::Component;
use crate::event::Event;
use crate::indexer::Index;
use crate::quick_open::{get_quick_open_results, QuickOpenResult};
use crate::terminal::{Rect, SPACES};
use std::cmp::min;
use std::fs;
use std::io::Write;
use termion::event::Key;

enum FilePaneItem {
    File(String),
    Folder(String),
}

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
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        write!(
            stream,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(rect.left, rect.top)
        )?;
        write!(
            stream,
            "{}{}",
            termion::color::Bg(termion::color::Yellow),
            termion::color::Fg(termion::color::Black)
        )?;
        write!(stream, "{}", self.search_query)?;

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

            write!(
                stream,
                "{}{}",
                termion::cursor::Goto(rect.left, row),
                result.path.file_name().unwrap().to_str().unwrap()
            )?;
            write!(
                stream,
                "{}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset)
            )?;
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

        Ok(())
    }

    fn dispatch_key(&mut self, key: Key) -> bool {
        self.events.clear();
        let handled = match key {
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
        };
        if let Some(selected_index) = self.selected_item_index {
            self.events.push(Event::FileItemSelected(
                self.results[selected_index].clone(),
            ));
        }
        handled
    }

    fn get_events(&self) -> Vec<Event> {
        return self.events.clone();
    }
    fn dispatch_events(&mut self, events: &[Event]) {}
}

struct DirectoryTreeComponent {
    selected_item_index: Option<usize>,
    items: Vec<FilePaneItem>,
}

impl Component for DirectoryTreeComponent {
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        let mut row = rect.top;
        for (index, item) in self.items.iter().enumerate() {
            write!(stream, "{}", termion::cursor::Goto(rect.left, row))?;
            if self.selected_item_index.is_some() && self.selected_item_index.unwrap() == index {
                write!(
                    stream,
                    "{}{}",
                    termion::color::Bg(termion::color::White),
                    termion::color::Fg(termion::color::Black)
                )?;
            } else {
                match item {
                    FilePaneItem::File(_) => {
                        write!(stream, "{}", termion::color::Fg(termion::color::White))?;
                    }
                    FilePaneItem::Folder(_) => {
                        write!(stream, "{}", termion::color::Fg(termion::color::Green))?;
                    }
                }
            }
            let line = match item {
                FilePaneItem::File(filename) => filename,
                FilePaneItem::Folder(foldername) => foldername,
            };
            // TODO: this is wrong, do what file_view does
            let line_length = line.chars().count();
            let truncated_length = min(line_length, rect.width as usize);
            if let Some(last_char) = line.char_indices().take(truncated_length).last() {
                write!(stream, "{}", &line[0..=last_char.0])?;
            }
            write!(
                stream,
                "{}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset)
            )?;
            row += 1;
        }
        while row < rect.top + rect.height {
            write!(
                stream,
                "{}{}",
                termion::cursor::Goto(rect.left, row),
                &SPACES[0..(rect.width as usize - 1)]
            )?;
            row += 1
        }
        stream.flush()
    }

    fn dispatch_key(&mut self, key: Key) -> bool {
        match key {
            Key::Down => {
                let next_item_index = match self.selected_item_index {
                    None => 0usize,
                    Some(index) => index + 1usize,
                };
                if next_item_index < self.items.len() {
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
            _ => false,
        }
    }

    fn get_events(&self) -> Vec<Event> {
        Vec::new()
    }
    fn dispatch_events(&mut self, events: &[Event]) {}
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
    pub fn new(cwd: &str) -> std::io::Result<FilePaneComponent> {
        let mut items: Vec<FilePaneItem> = Vec::new();
        for entry in fs::read_dir(cwd)? {
            if let Ok(entry) = entry {
                let file_type = entry.file_type()?;
                let file_name = entry.file_name().into_string().unwrap();
                if file_type.is_file() {
                    items.push(FilePaneItem::File(file_name));
                } else if file_type.is_dir() {
                    items.push(FilePaneItem::Folder(file_name));
                }
            }
        }
        return Ok(FilePaneComponent {
            directory_tree: DirectoryTreeComponent {
                selected_item_index: None,
                items: items,
            },
            quick_open: QuickOpenComponent::new(),
            mode: FilePaneMode::DirectoryTree,
        });
    }

    pub fn start_quick_open(&mut self, index: Index) {
        self.quick_open.index = Some(index);
        self.mode = FilePaneMode::QuickOpen;
    }
}

impl Component for FilePaneComponent {
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        match self.mode {
            FilePaneMode::DirectoryTree => self.directory_tree.paint(stream, rect),
            FilePaneMode::QuickOpen => self.quick_open.paint(stream, rect),
        }
    }
    fn dispatch_key(&mut self, key: Key) -> bool {
        match key {
            Key::Esc => match self.mode {
                FilePaneMode::QuickOpen => {
                    self.mode = FilePaneMode::DirectoryTree;
                    return true;
                }
                _ => {}
            },
            _ => {}
        }

        match self.mode {
            FilePaneMode::DirectoryTree => self.directory_tree.dispatch_key(key),
            FilePaneMode::QuickOpen => self.quick_open.dispatch_key(key),
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
