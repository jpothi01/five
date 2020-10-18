use std::convert::TryFrom;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct Config {
    text_width: u16,
    text_height: u16,
}

impl Config {
    fn new() -> Config {
        Config {
            text_width: 80u16,
            text_height: 5u16,
        }
    }
}

fn get_terminal_size() -> (u16, u16) {
    let (width, height) = term_size::dimensions().unwrap();
    (
        u16::try_from(width).unwrap(),
        u16::try_from(height).unwrap(),
    )
}

fn get_lines(buffer: &String, text_width: usize) -> Vec<String> {
    let mut current_line = String::new();
    let mut lines: Vec<String> = Vec::new();

    let mut current_line_length = 0;

    let mut remaining_buffer = &buffer[0..];
    while !remaining_buffer.is_empty() {
        // First step: copy whitespace
        let maybe_next_non_whitespace_index = remaining_buffer
            .chars()
            .position(|c| !c.is_whitespace() || c == '\n');
        let (mut whitespace_chunk, _remaining_buffer) = match maybe_next_non_whitespace_index {
            None => {
                // It's all whitespace
                (remaining_buffer, "")
            }
            Some(next_non_whitespace_index) => (
                &remaining_buffer[0..next_non_whitespace_index],
                &remaining_buffer[next_non_whitespace_index..],
            ),
        };
        remaining_buffer = _remaining_buffer;

        loop {
            let current_line_remaining = text_width - current_line_length;
            let whitespace_chunk_length = whitespace_chunk.chars().count();
            if whitespace_chunk_length < current_line_remaining {
                current_line.extend(whitespace_chunk.chars());
                current_line_length += whitespace_chunk_length;
                break;
            }

            current_line.extend(whitespace_chunk[0..current_line_remaining].chars());
            lines.push(current_line.clone());
            current_line.clear();
            current_line_length = 0;
            whitespace_chunk = &whitespace_chunk[current_line_remaining..];
        }

        if !remaining_buffer.is_empty() && remaining_buffer.chars().nth(0).unwrap() == '\n' {
            remaining_buffer = &remaining_buffer[1..];
            lines.push(current_line.clone());
            current_line.clear();
            current_line_length = 0;
            continue;
        }

        // Next step: place the next word
        let maybe_next_word_end_index = remaining_buffer.find(char::is_whitespace);
        let (__remaining_buffer, word_chunk) = match maybe_next_word_end_index {
            None => {
                // Buffer ends with a word
                ("", remaining_buffer)
            }
            Some(next_word_end_index) => (
                &remaining_buffer[next_word_end_index..],
                &remaining_buffer[0..next_word_end_index],
            ),
        };
        remaining_buffer = __remaining_buffer;

        let word_chunk_length = word_chunk.chars().count();
        // TODO: If the word is longer than the entire text width, just treat the excess as a brand new word
        assert!(word_chunk_length < text_width);
        if word_chunk_length == 0 {
            continue;
        }

        if current_line_length + word_chunk_length < text_width {
            current_line.extend(word_chunk.chars());
            current_line_length += word_chunk_length;
        } else {
            lines.push(current_line.clone());
            current_line = String::from(word_chunk);
            current_line_length = word_chunk_length;
        }
    }

    lines.push(current_line.clone());

    return lines;
}

fn paint<Writer: Write>(
    stream: &mut Writer,
    buffer: &String,
    config: &Config,
    terminal_size: (u16, u16),
) -> std::io::Result<()> {
    // TODO: bounds checking
    let padding_x = (terminal_size.0 - config.text_width) / 2;
    let padding_y = (terminal_size.1 - config.text_height) / 2;

    write!(stream, "{}", termion::clear::All)?;
    write!(stream, "{}", termion::cursor::Goto(1, 1))?;

    let mut current_y = padding_y;
    let text_width = config.text_width as usize;
    let text_height = config.text_height as usize;
    let lines = get_lines(&buffer, text_width);
    let num_lines_to_skip = if lines.len() > text_height {
        lines.len() - text_height
    } else {
        0
    };

    for line in lines.iter().skip(num_lines_to_skip) {
        write!(
            stream,
            "{}{}",
            termion::cursor::Goto(padding_x, current_y),
            line
        )?;
        current_y += 1;
    }
    stream.flush()
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut buffer = String::new();
    let config = Config::new();

    paint(&mut stdout, &buffer, &config, get_terminal_size()).unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl(c) => {
                if c == 'c' {
                    break;
                }
            }
            Key::Char(c) => {
                buffer.push(c);
                paint(&mut stdout, &buffer, &config, get_terminal_size()).unwrap();
            }
            Key::Backspace => {
                buffer.pop();
                paint(&mut stdout, &buffer, &config, get_terminal_size()).unwrap();
            }
            _ => {}
        }
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
