use std::io::{stdin, stdout, Write};
use structopt::StructOpt;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod file_pane;
mod file_view;
mod terminal;

fn run() -> String {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();

    let mut buffer = String::new();
    let file_pane_view_model = file_pane::ViewModel::new(".").unwrap();
    file_pane::paint(&mut stdout, &file_pane_view_model).unwrap();

    let rect = terminal::Rect {
        left: 20u16,
        top: 10u16,
        width: 100u16,
        height: 40u16,
    };

    let test_file = "Cargo.toml";
    let file_view_model = file_view::ViewModel::new(test_file).unwrap();
    file_view::paint(&mut stdout, rect, &file_view_model).unwrap();

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
    }

    write!(stdout, "{}", termion::screen::ToMainScreen).unwrap();

    return buffer;
}

#[derive(Debug, StructOpt)]
#[structopt(name = "edit", about = "A brutal text editor.")]
struct Options {}

fn main() {
    let options = Options::from_args();
    run();
}
