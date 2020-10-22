use crate::terminal::Rect;
use std::io;
use std::io::Write;

pub fn paint<Writer: Write>(stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
    for row in rect.top..=(rect.top + rect.height) {
        write!(
            stream,
            "{}{} ",
            termion::cursor::Goto(rect.left, row),
            termion::color::Bg(termion::color::Yellow)
        )?;
    }

    write!(stream, "{}", termion::color::Bg(termion::color::Reset))?;
    stream.flush()?;

    Ok(())
}
