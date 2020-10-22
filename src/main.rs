use std::convert::TryFrom;
use std::io::{stdin, stdout, Write};
use structopt::StructOpt;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod divider;
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

    let terminal_size = terminal::get_terminal_size();
    let terminal_width = u16::try_from(terminal_size.0).unwrap();
    let terminal_height = u16::try_from(terminal_size.1).unwrap();
    let margin = 2u16;
    let file_pane_rect = terminal::Rect {
        left: margin,
        top: margin,
        width: 20,
        height: terminal_height - 2 * margin,
    };
    let divider_rect = terminal::Rect {
        left: file_pane_rect.width + 1,
        top: 0,
        width: 1,
        height: terminal_height,
    };
    let file_view_rect = terminal::Rect {
        left: margin + file_pane_rect.width + divider_rect.width,
        top: margin,
        width: terminal_width - file_pane_rect.width - margin,
        height: terminal_height - 2 * margin,
    };

    divider::paint(&mut stdout, divider_rect).unwrap();

    let mut file_pane_view_model =
        file_pane::ViewModel::new("/Users/john/code/writing/content").unwrap();
    file_pane::paint(&mut stdout, file_pane_rect, &file_pane_view_model).unwrap();

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
                    let file_path = format!("/Users/john/code/writing/content/{}", filename);
                    let file_view_model = file_view::ViewModel::new(&file_path).unwrap();
                    file_view::paint(&mut stdout, file_view_rect, &file_view_model).unwrap();
                }
                _ => {}
            };
        }
        file_pane::paint(&mut stdout, file_pane_rect, &file_pane_view_model).unwrap();
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
