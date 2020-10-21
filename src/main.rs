use std::convert::TryFrom;
use std::fs;
use std::io::{stdin, stdout, Write};
use structopt::StructOpt;
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct Config {}

fn get_terminal_size() -> (usize, usize) {
    term_size::dimensions().unwrap()
}
fn paint<Writer: Write>(stream: &mut Writer) -> std::io::Result<()> {
    // TODO: bounds checking
    let terminal_size = get_terminal_size();

    stream.flush()
}

fn run(config: &Config) -> String {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();

    let mut buffer = String::new();

    paint(&mut stdout).unwrap();

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

        paint(&mut stdout).unwrap();
    }

    write!(stdout, "{}", termion::screen::ToMainScreen).unwrap();

    return buffer;
}

#[derive(Debug, StructOpt)]
#[structopt(name = "edit", about = "A brutal text editor.")]
struct Options {}

fn main() {
    let options = Options::from_args();
    let config = Config {};
    run(&config);
}
