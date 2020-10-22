use crate::terminal::{Rect, SPACES};
use std::cmp::min;
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

pub fn paint<Writer: Write>(
    stream: &mut Writer,
    rect: Rect,
    view_model: &ViewModel,
) -> std::io::Result<()> {
    let mut row = rect.top;
    for (index, item) in view_model.items.iter().enumerate() {
        write!(stream, "{}", termion::cursor::Goto(rect.left, row))?;
        if view_model.selected_item_index.is_some()
            && view_model.selected_item_index.unwrap() == index
        {
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
