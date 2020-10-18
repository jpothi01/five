use std::convert::TryFrom;
use std::fs;
use std::io::{stdin, stdout, Write};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct Config {
    text_width: usize,
    text_height: usize,
    word_goal: usize,
}

impl Config {
    fn new() -> Config {
        Config {
            text_width: 80,
            text_height: 5,
            word_goal: 10,
        }
    }
}

fn get_terminal_size() -> (usize, usize) {
    term_size::dimensions().unwrap()
}

// Returns tuple of (lines, num_words)
fn get_lines(buffer: &String, text_width: usize) -> (Vec<String>, usize) {
    let mut current_line = String::new();
    let mut lines: Vec<String> = Vec::new();

    let mut current_line_length = 0;
    let mut num_words = 0;
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

        num_words += 1;
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

    return (lines, num_words);
}

fn goal_achieved(config: &Config, num_words: usize) -> bool {
    num_words >= config.word_goal
}

fn paint<Writer: Write>(
    stream: &mut Writer,
    buffer: &String,
    config: &Config,
) -> std::io::Result<()> {
    // TODO: bounds checking
    let terminal_size = get_terminal_size();
    let padding_x = u16::try_from((terminal_size.0 - config.text_width) / 2).unwrap();
    let padding_y = u16::try_from((terminal_size.1 - config.text_height) / 2).unwrap();
    let last_line = u16::try_from(terminal_size.1).unwrap();
    let last_column = u16::try_from(terminal_size.0).unwrap();

    // Careful not to clear the "goal achieved" bar so it doesn't flicker.
    write!(
        stream,
        "{}{}",
        termion::cursor::Goto(1, last_line),
        termion::clear::BeforeCursor
    )?;
    write!(stream, "{}", termion::cursor::Goto(1, 1))?;

    let mut current_y = padding_y;
    let (lines, num_words) = get_lines(&buffer, config.text_width);
    let num_lines_to_skip = if lines.len() > config.text_height {
        lines.len() - config.text_height
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

    if goal_achieved(config, num_words) {
        let stored_cursor_pos = stream.cursor_pos()?;
        let color_goal_achieved = termion::color::Green;
        write!(
            stream,
            "{}{}",
            termion::cursor::Goto(1, last_line),
            termion::color::Bg(color_goal_achieved),
        )?;
        write!(stream, "{}", " ".repeat(last_column as usize))?;
        write!(stream, "{}", termion::cursor::Goto(last_column, last_line))?;
        write!(
            stream,
            "{}{}",
            termion::color::Bg(termion::color::Reset),
            termion::cursor::Goto(stored_cursor_pos.0, stored_cursor_pos.1)
        )?;
    }

    stream.flush()
}

fn save(buffer: &String) -> std::io::Result<String> {
    let filename = format!("draft_{}.txt", chrono::Local::now().format("%F_%H_%M_%S"));
    fs::write(&filename, buffer.as_bytes())?;
    Ok(filename)
}

fn run(config: Config) -> String {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();

    let mut buffer = String::new();

    paint(&mut stdout, &buffer, &config).unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl(c) => {
                if c == 'c' {
                    break;
                }
            }
            Key::Char(c) => {
                buffer.push(c);
            }
            Key::Backspace => {
                buffer.pop();
            }
            _ => {}
        }

        paint(&mut stdout, &buffer, &config).unwrap();
    }

    write!(stdout, "{}", termion::screen::ToMainScreen).unwrap();
    return buffer;
}

fn main() {
    let buffer = run(Config::new());
    if buffer.is_empty() {
        return;
    }

    match save(&buffer) {
        Ok(filename) => println!("Writing saved to '{}'", filename),
        Err(err) => println!("Could not save file: {}", err),
    };
}
