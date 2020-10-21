use crate::terminal::get_terminal_size;
use std::fs;
use std::io::Write;

enum FilePaneItem {
    File(String),
    Folder(String),
}

pub struct ViewModel {
    items: Vec<FilePaneItem>,
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
        return Ok(ViewModel { items: items });
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
