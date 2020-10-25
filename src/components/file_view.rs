use super::component::Component;
use crate::terminal::{Rect, SPACES};
use std::cmp::min;
use std::io::Write;
use termion;

pub struct FileViewComponent {
    content: String,
}

impl FileViewComponent {
    pub fn new() -> FileViewComponent {
        FileViewComponent {
            content: String::new(),
        }
    }
}

impl Component for FileViewComponent {
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        write!(stream, "{}", termion::color::Fg(termion::color::White))?;
        let lines = self.content.lines();
        let mut row = 1u16;
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
}
