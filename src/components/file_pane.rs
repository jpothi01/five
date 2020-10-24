use crate::components::component::Component;
use crate::indexer::Index;
use crate::terminal::{Rect, SPACES};
use std::cmp::min;
use std::fs;
use std::io::Write;
use termion::event::Key;

enum FilePaneItem {
    File(String),
    Folder(String),
}

enum FilePaneMode {
    DirectoryTree,
    QuickOpen,
}

pub struct FilePaneComponent {
    items: Vec<FilePaneItem>,
    selected_item_index: Option<usize>,
    index: Option<Index>,
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
            items: items,
            selected_item_index: None,
            index: None,
            mode: FilePaneMode::DirectoryTree,
        });
    }

    pub fn start_quick_open(&mut self, index: Index) {
        self.index = Some(index);
        self.mode = FilePaneMode::QuickOpen;
    }

    fn paint_directory_tree<Writer: Write>(
        &self,
        stream: &mut Writer,
        rect: Rect,
    ) -> std::io::Result<()> {
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

    fn paint_quick_open<Writer: Write>(
        &self,
        stream: &mut Writer,
        rect: Rect,
    ) -> std::io::Result<()> {
        write!(
            stream,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )?;
        write!(
            stream,
            "Number of files: {}",
            self.index.as_ref().unwrap().files.len()
        )
    }
}

impl Component for FilePaneComponent {
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        match self.mode {
            FilePaneMode::DirectoryTree => self.paint_directory_tree(stream, rect),
            FilePaneMode::QuickOpen => self.paint_quick_open(stream, rect),
        }
    }
    fn dispatch_key(&mut self, key: Key) -> bool {
        match key {
            Key::Esc => match self.mode {
                FilePaneMode::QuickOpen => {
                    self.mode = FilePaneMode::DirectoryTree;
                    true
                }
                _ => false,
            },
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
}
