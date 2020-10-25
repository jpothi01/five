use super::component::Component;
use crate::event::Event;
use crate::terminal::{Rect, SPACES};
use std::cmp::min;
use std::io::Write;
use std::path::Path;
use termion;

pub struct FileViewComponent {
    content: String,
    file_name: String,
    file_path: String,
}

impl FileViewComponent {
    pub fn new() -> FileViewComponent {
        FileViewComponent {
            content: String::new(),
            file_name: String::new(),
            file_path: String::new(),
        }
    }

    pub fn set_text_file_content(&mut self, file_path: &Path, content: String) {
        self.content = content;
        self.file_path = String::from(file_path.to_str().unwrap_or(""));
    }

    pub fn set_binary_file_content(&mut self, file_path: &Path) {
        self.content = String::from("<binary file>");
        self.file_path = String::from(file_path.to_str().unwrap_or(""));
    }
}

impl Component for FileViewComponent {
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        write!(stream, "{}", termion::color::Fg(termion::color::Yellow))?;
        write!(
            stream,
            "{}{}",
            termion::cursor::Goto(rect.left, rect.top),
            self.file_path
        )?;
        write!(stream, "{}", termion::color::Fg(termion::color::White))?;
        let lines = self.content.lines();

        let mut row = rect.top + 1;
        for line in lines {
            write!(stream, "{}", termion::cursor::Goto(rect.left, row))?;
            // TODO: clean this up and centralize the logic
            let line_length = line.chars().count();
            if line_length < rect.width as usize {
                write!(stream, "{}", line)?;
                write!(
                    stream,
                    "{}",
                    &SPACES[0..(rect.width as usize - line_length)]
                )?;
            } else {
                let truncated_length = min(line_length, rect.width as usize);
                if let Some(last_char) = line.char_indices().take(truncated_length).last() {
                    write!(stream, "{}", &line[0..last_char.0])?;
                }
                write!(
                    stream,
                    "{}",
                    &SPACES[0..(rect.width as usize - truncated_length)]
                )?;
            }
            row += 1;
            if row > rect.height {
                break;
            }
        }

        while row < rect.top + rect.height {
            write!(
                stream,
                "{}{}",
                termion::cursor::Goto(rect.left, row),
                &SPACES[0..(rect.width as usize)]
            )?;
            row += 1
        }

        stream.flush()
    }

    fn dispatch_key(&mut self, key: termion::event::Key) -> bool {
        false
    }

    fn get_events(&self) -> Vec<Event> {
        Vec::new()
    }

    fn dispatch_events(&mut self, events: &[Event]) {}
}
