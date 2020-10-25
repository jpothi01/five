use components::component::Component;
use std::convert::TryFrom;
use std::io::{stdin, stdout, Write};
use structopt::StructOpt;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod components;
mod divider;
mod file_view;
mod indexer;
mod quick_open;
mod terminal;

enum View {
    File,
    FilePane,
}

struct Config {
    cwd: String,
}

fn run(config: Config) {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();

    let terminal_size = terminal::get_terminal_size();
    let terminal_width = u16::try_from(terminal_size.0).unwrap();
    let terminal_height = u16::try_from(terminal_size.1).unwrap();
    let margin = 2u16;
    let root_rect = terminal::Rect {
        left: margin,
        top: margin,
        width: terminal_width - 2 * margin,
        height: terminal_height - 2 * margin,
    };

    let mut root_component = components::root::RootComponent::new(&config.cwd);

    root_component.paint(&mut stdout, root_rect).unwrap();

    for c in stdin.keys() {
        let key = c.unwrap();

        match key {
            Key::Ctrl(c) => {
                if c == 'c' {
                    break;
                }
            }
            _ => {}
        }

        root_component.dispatch_key(key);
        root_component.paint(&mut stdout, root_rect).unwrap();

        // if let Some(index) = file_pane_view_model.selected_item_index {
        //     match &file_pane_view_model.items[index] {
        //         file_pane::FilePaneItem::File(filename) => {
        //             let file_path = format!("/Users/john/code/writing/content/{}", filename);
        //             let file_view_model = file_view::ViewModel::new(&file_path).unwrap();
        //             file_view::paint(&mut stdout, file_view_rect, &file_view_model).unwrap();
        //         }
        //         _ => {}
        //     };
        // }
        // file_pane::paint(&mut stdout, file_pane_rect, &file_pane_view_model).unwrap();
    }

    write!(stdout, "{}", termion::screen::ToMainScreen).unwrap();
}

#[derive(Debug, StructOpt)]
#[structopt(name = "five", about = "A brutal text editor.")]
struct Options {
    #[structopt(
        parse(from_str),
        help = "Directory to open. Current directory if unspecified."
    )]
    directory: Option<String>,
}

fn main() {
    let options = Options::from_args();
    let config = Config {
        cwd: options.directory.unwrap_or(String::from(".")),
    };
    run(config);
}
