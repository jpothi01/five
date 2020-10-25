/*
    Copyright 2020, John Pothier
    This file is part of Five.

    Five is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Five is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Five.  If not, see <https://www.gnu.org/licenses/>.
*/

use components::component::Component;
use std::convert::TryFrom;
use std::io::{stdin, Write};
use structopt::StructOpt;
use termion::event::Event;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod components;
mod event;
mod indexer;
mod quick_open;
mod terminal;

struct Config {
    cwd: String,
}

fn run(config: Config) {
    let stdin = stdin();
    let mut stdout = MouseTerminal::from(std::io::stdout().into_raw_mode().unwrap());
    write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();

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

    for c in stdin.events() {
        let event = c.unwrap();

        match event {
            Event::Key(key) => match key {
                Key::Ctrl(c) => {
                    if c == 'c' {
                        break;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        root_component.dispatch_event(event);
        let events = root_component.get_events();
        root_component.dispatch_events(&events);
        root_component.paint(&mut stdout, root_rect).unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
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
