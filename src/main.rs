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
use std::path::PathBuf;
use structopt::StructOpt;
use termion::event::Event;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod components;
mod event;
mod indexer;
mod painting_utils;
mod quick_open;
mod terminal;

use indexer::index::Indexer;
use indexer::local_index::LocalIndexer;
use indexer::ssh_index::SshIndexer;

struct LocalConfig {
    directory_path: PathBuf,
}

struct SshConfig {
    ssh_args: Vec<String>,
}

enum LocationConfig {
    Local(LocalConfig),
    Remote(SshConfig),
}

struct Config {
    location_config: LocationConfig,
}

fn run(config: Config) {
    let stdin = stdin();
    let mut stdout = MouseTerminal::from(std::io::stdout().into_raw_mode().unwrap());
    write!(stdout, "{}", termion::screen::ToAlternateScreen).unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    let terminal_size = terminal::get_terminal_size();
    let terminal_width = u16::try_from(terminal_size.0).unwrap();
    let terminal_height = u16::try_from(terminal_size.1).unwrap();
    let root_rect = terminal::Rect {
        left: 1,
        top: 1,
        width: terminal_width - 1,
        height: terminal_height,
    };

    let indexer: Box<dyn Indexer> = match config.location_config {
        LocationConfig::Local(local_config) => {
            Box::new(LocalIndexer::new(local_config.directory_path))
        }
        LocationConfig::Remote(ssh_config) => Box::new(SshIndexer::new(ssh_config.ssh_args)),
    };
    let mut root_component = components::root::RootComponent::new(&*indexer);

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
    #[structopt(long = "ssh")]
    ssh: bool,

    #[structopt(
        parse(from_str),
        help = "Directory to open. Current directory if unspecified."
    )]
    directory_or_ssh_options: Vec<String>,
}

fn main() {
    let options = Options::from_args();
    let config = if options.ssh {
        Config {
            location_config: LocationConfig::Remote(SshConfig {
                ssh_args: options.directory_or_ssh_options,
            }),
        }
    } else {
        Config {
            location_config: LocationConfig::Local(LocalConfig {
                directory_path: if options.directory_or_ssh_options.len() == 0 {
                    PathBuf::from(".")
                } else {
                    PathBuf::from(options.directory_or_ssh_options.first().unwrap())
                },
            }),
        }
    };

    run(config);
}
