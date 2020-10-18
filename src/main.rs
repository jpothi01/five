use std::convert::TryFrom;
use std::fs;
use std::io::{stdin, stdout, Write};
use structopt::StructOpt;
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

const DEFAULT_WIDTH: usize = 80;
const DEFAULT_HEIGHT: usize = 5;

struct Config {
    text_width: usize,
    text_height: usize,
    word_goal: Option<usize>,
}

fn get_terminal_size() -> (usize, usize) {
    term_size::dimensions().unwrap()
}

fn show_goal_indicator(config: &Config, num_words: usize) -> bool {
    match config.word_goal {
        None => false,
        Some(word_goal) => num_words >= word_goal,
    }
}

fn goal_achieved(config: &Config, num_words: usize) -> bool {
    match config.word_goal {
        None => true,
        Some(word_goal) => num_words >= word_goal,
    }
}

struct BufferDecomposition {
    lines: Vec<String>,
    num_words: usize,
}

impl BufferDecomposition {
    fn new() -> BufferDecomposition {
        BufferDecomposition {
            lines: vec![],
            num_words: 0,
        }
    }

    fn from_buffer(buffer: &String, text_width: usize) -> BufferDecomposition {
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
        return BufferDecomposition {
            lines: lines,
            num_words: num_words,
        };
    }
}

fn paint<Writer: Write>(
    stream: &mut Writer,
    decomposition: &BufferDecomposition,
    config: &Config,
) -> std::io::Result<()> {
    // TODO: bounds checking
    let terminal_size = get_terminal_size();
    let padding_x = u16::try_from((terminal_size.0 - config.text_width) / 2).unwrap();
    let padding_y = u16::try_from((terminal_size.1 - config.text_height) / 2).unwrap();
    let last_line = u16::try_from(terminal_size.1).unwrap();
    let last_column = u16::try_from(terminal_size.0).unwrap();
    let lines = &decomposition.lines;
    let num_words = decomposition.num_words;

    let mut current_y = padding_y;
    let num_lines_to_skip = if lines.len() > config.text_height {
        lines.len() - config.text_height
    } else {
        0
    };

    // Careful not to clear the "goal achieved" bar so it doesn't flicker.
    write!(
        stream,
        "{}{}",
        termion::cursor::Goto(1, last_line),
        termion::clear::BeforeCursor
    )?;
    write!(stream, "{}", termion::cursor::Goto(padding_x, current_y))?;

    for line in lines.iter().skip(num_lines_to_skip) {
        write!(
            stream,
            "{}{}",
            termion::cursor::Goto(padding_x, current_y),
            line
        )?;
        current_y += 1;
    }

    if show_goal_indicator(config, num_words) {
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

fn run(config: &Config) -> String {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();

    let mut buffer = String::new();

    paint(&mut stdout, &BufferDecomposition::new(), &config).unwrap();

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

        let decomposition = BufferDecomposition::from_buffer(&buffer, config.text_width);
        paint(&mut stdout, &decomposition, &config).unwrap();
    }

    write!(stdout, "{}", termion::screen::ToMainScreen).unwrap();

    return buffer;
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "draft",
    about = "A brutal drafting tool for writers who need a kick in the teeth."
)]
struct Options {
    #[structopt(
        short = "w",
        long = "words",
        help = "Word goal. If set, if you write fewer than this number of words, your writing is not saved"
    )]
    words: Option<usize>,
}

fn main() {
    let options = Options::from_args();
    let config = Config {
        text_height: DEFAULT_HEIGHT,
        text_width: DEFAULT_WIDTH,
        word_goal: options.words,
    };
    let buffer = run(&config);
    if buffer.is_empty() {
        return;
    }

    let decomposition = BufferDecomposition::from_buffer(&buffer, config.text_width);
    if !goal_achieved(&config, decomposition.num_words) {
        println!("Writing not saved. Goal not achieved");
        return;
    }

    match save(&buffer) {
        Ok(filename) => println!("Writing saved to '{}'", filename),
        Err(err) => println!("Could not save file: {}", err),
    };
}
