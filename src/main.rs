use std::io::{stdin, stdout, Write};
use structopt::StructOpt;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod file_pane;
mod file_view;
mod terminal;

enum View {
    File,
    FilePane,
}

fn run() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();

    let mut file_pane_view_model = file_pane::ViewModel::new(".").unwrap();
    file_pane::paint(&mut stdout, &file_pane_view_model).unwrap();

    let rect = terminal::Rect {
        left: 20u16,
        top: 10u16,
        width: 100u16,
        height: 40u16,
    };

    let focused_view = View::FilePane;

    for c in stdin.keys() {
        let key = c.unwrap();
        match key {
            Key::Ctrl(c) => {
                if c == 'c' {
                    break;
                }
            }
            _ => match focused_view {
                View::FilePane => {
                    file_pane::dispatch_key(&mut stdout, key, &mut file_pane_view_model)
                }
                _ => {}
            },
        }

        if let Some(index) = file_pane_view_model.selected_item_index {
            match &file_pane_view_model.items[index] {
                file_pane::FilePaneItem::File(filename) => {
                    let file_view_model = file_view::ViewModel::new(filename).unwrap();
                    file_view::paint(&mut stdout, rect, &file_view_model).unwrap();
                }
                _ => {}
            };
        }
        file_pane::paint(&mut stdout, &file_pane_view_model).unwrap();
    }

    write!(stdout, "{}", termion::screen::ToMainScreen).unwrap();
}

#[derive(Debug, StructOpt)]
#[structopt(name = "edit", about = "A brutal text editor.")]
struct Options {}

fn main() {
    let options = Options::from_args();
    run();
}
