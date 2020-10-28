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

use super::component::Component;
use crate::event::Event;
use crate::painting_utils::{paint_empty_lines, paint_truncated_text};
use crate::terminal::{Rect, SPACES};
use std::cell::Cell;
use std::cmp::min;
use std::io::Write;
use std::path::Path;
use termion;

pub struct FileViewComponent {
    content: String,
    num_content_lines: usize,
    file_name: String,
    file_path: String,
    start_line: usize,
    needs_paint: Cell<bool>,
}

impl FileViewComponent {
    pub fn new() -> FileViewComponent {
        FileViewComponent {
            content: String::new(),
            num_content_lines: 0,
            file_name: String::new(),
            file_path: String::new(),
            start_line: 0usize,
            needs_paint: Cell::new(true),
        }
    }

    pub fn set_text_file_content(&mut self, file_path: &Path, content: String) {
        self.content = content;
        self.num_content_lines = self.content.lines().count();
        self.file_path = String::from(file_path.to_str().unwrap_or(""));
        self.start_line = 0;
        self.needs_paint.set(true);
    }

    pub fn set_binary_file_content(&mut self, file_path: &Path) {
        self.content = String::from("<binary file>");
        self.num_content_lines = 1;
        self.file_path = String::from(file_path.to_str().unwrap_or(""));
        self.start_line = 0;
        self.needs_paint.set(true);
    }

    fn scroll_down(&mut self) {
        if self.start_line < self.num_content_lines {
            self.start_line = self.start_line + 1;
            self.needs_paint.set(true);
        }
    }

    fn scroll_up(&mut self) {
        if self.start_line > 0 {
            self.start_line = self.start_line - 1;
            self.needs_paint.set(true);
        }
    }
}

impl Component for FileViewComponent {
    fn needs_paint(&self) -> bool {
        self.needs_paint.take()
    }
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        write!(stream, "{}", termion::color::Fg(termion::color::Yellow))?;
        write!(stream, "{}", termion::cursor::Goto(rect.left, rect.top),)?;
        paint_truncated_text(stream, &self.file_path, rect.width)?;

        write!(stream, "{}", termion::color::Fg(termion::color::White))?;
        let lines = self.content.lines().skip(self.start_line);

        let mut row = rect.top + 1;
        for line in lines {
            write!(stream, "{}", termion::cursor::Goto(rect.left, row))?;
            // TODO: clean this up and centralize the logic
            let line_length = line.chars().count();
            if line_length < rect.width as usize {
                write!(stream, "{}", line)?;
                write!(
                    stream,
                    "{}",
                    &SPACES[0..(rect.width as usize - line_length)]
                )?;
            } else {
                let truncated_length = min(line_length, rect.width as usize);
                if let Some(last_char) = line.char_indices().take(truncated_length).last() {
                    write!(stream, "{}", &line[0..last_char.0])?;
                }
                write!(
                    stream,
                    "{}",
                    &SPACES[0..(rect.width as usize - truncated_length)]
                )?;
            }
            row += 1;
            if row > rect.height {
                break;
            }
        }

        while row < rect.top + rect.height {
            write!(
                stream,
                "{}{}",
                termion::cursor::Goto(rect.left, row),
                &SPACES[0..(rect.width as usize)]
            )?;
            row += 1
        }

        self.needs_paint.set(false);
        Ok(())
    }

    fn dispatch_event(&mut self, event: termion::event::Event) -> bool {
        match event {
            termion::event::Event::Mouse(mouse_event) => match mouse_event {
                termion::event::MouseEvent::Press(button, _, _) => match button {
                    termion::event::MouseButton::WheelDown => {
                        self.scroll_down();
                        true
                    }
                    termion::event::MouseButton::WheelUp => {
                        self.scroll_up();
                        true
                    }
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        }
    }

    fn get_events(&self) -> Vec<Event> {
        Vec::new()
    }

    fn dispatch_events(&mut self, events: &[Event]) {}
}
