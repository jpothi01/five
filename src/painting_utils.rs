use crate::terminal::Rect;
use crate::terminal::SPACES;
use std::io::Write;

pub fn paint_truncated_text<Writer: Write>(
    stream: &mut Writer,
    text: &str,
    target_width: u16,
) -> std::io::Result<()> {
    let text_slice = match text.char_indices().nth(target_width as usize) {
        None => text,
        Some((index, _)) => &text[0..index],
    };
    write!(stream, "{}", text_slice)?;
    let num_spaces = (target_width as usize) - text_slice.chars().count();
    write!(stream, "{}", &SPACES[0..num_spaces])
}

pub fn paint_empty_lines<Writer: Write>(stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
    for row in rect.top..=rect.top + rect.height {
        write!(
            stream,
            "{}{}",
            termion::cursor::Goto(rect.left, row),
            &SPACES[0..(rect.width as usize)]
        )?;
    }
    Ok(())
}
