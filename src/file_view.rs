use crate::terminal::Rect;
use std::fs;
use std::io::Write;
use termion;

const SPACES: &str = "                                                                                                                                                                                                                                                                                                            ";

pub struct ViewModel {
    content: String,
}

impl ViewModel {
    pub fn new(file_path: &str) -> std::io::Result<ViewModel> {
        let content = fs::read_to_string(file_path)?;
        return Ok(ViewModel { content: content });
    }
}

pub fn paint<Writer: Write>(
    stream: &mut Writer,
    rect: Rect,
    view_model: &ViewModel,
) -> std::io::Result<()> {
    write!(stream, "{}", termion::color::Fg(termion::color::White))?;
    let lines = view_model.content.lines();
    let mut row = 1u16;
    for line in lines {
        let line_length = line.chars().count();
        write!(stream, "{}{}", termion::cursor::Goto(rect.left, row), line)?;
        write!(
            stream,
            "{}",
            &SPACES[0..(rect.width as usize - line_length)]
        )?;
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
