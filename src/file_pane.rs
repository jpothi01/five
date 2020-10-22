use crate::terminal::get_terminal_size;
use std::fs;
use std::io::Write;

pub enum FilePaneItem {
    File(String),
    Folder(String),
}

pub struct ViewModel {
    pub items: Vec<FilePaneItem>,
    pub selected_item_index: Option<usize>,
    pub is_focused: bool,
}

impl ViewModel {
    pub fn new(cwd: &str) -> std::io::Result<ViewModel> {
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
        return Ok(ViewModel {
            items: items,
            selected_item_index: None,
            is_focused: false,
        });
    }
}

pub fn paint<Writer: Write>(stream: &mut Writer, view_model: &ViewModel) -> std::io::Result<()> {
    let terminal_size = get_terminal_size();

    let mut row = 1u16;
    for item in &view_model.items {
        match item {
            FilePaneItem::File(filename) => {
                write!(
                    stream,
                    "{}{}{}",
                    termion::cursor::Goto(1, row),
                    termion::color::Fg(termion::color::White),
                    filename
                )?;
            }
            FilePaneItem::Folder(foldername) => {
                write!(
                    stream,
                    "{}{}{}",
                    termion::cursor::Goto(1, row),
                    termion::color::Fg(termion::color::Green),
                    foldername
                )?;
            }
        };

        row += 1;
    }

    stream.flush()
}

pub fn dispatch_key<Writer: Write>(
    stream: &mut Writer,
    key: termion::event::Key,
    view_model: &mut ViewModel,
) {
    match key {
        termion::event::Key::Down => {
            let next_item_index = match view_model.selected_item_index {
                None => 0usize,
                Some(index) => index + 1usize,
            };
            if next_item_index < view_model.items.len() {
                view_model.selected_item_index = Some(next_item_index)
            }
        }
        termion::event::Key::Up => {
            let maybe_next_item_index = match view_model.selected_item_index {
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
                view_model.selected_item_index = Some(next_item_index)
            }
        }
        _ => {}
    }
}
