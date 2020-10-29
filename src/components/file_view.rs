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
use crate::terminal::Rect;
use std::cell::Cell;
use std::convert::TryFrom;
use std::io::Write;
use std::path::Path;
use termion;

pub struct FileViewComponent {
    content: String,
    num_content_lines: usize,
    file_name: String,
    file_path: String,
    start_line: usize,
    has_focus: bool,
    file_cursor_position: (usize, usize),
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
            has_focus: false,
            file_cursor_position: (0, 0),
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

    pub fn set_has_focus(&mut self, focused: bool) {
        self.has_focus = focused;
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
            paint_truncated_text(stream, line, rect.width)?;

            if row >= rect.height {
                break;
            }

            row += 1;
        }

        paint_empty_lines(
            stream,
            Rect {
                top: row,
                left: rect.left,
                width: rect.width,
                height: rect.height - row + 1,
            },
        )?;

        if self.has_focus {
            let header_height = 1u16;
            let visible_cursor_position = (
                rect.left + u16::try_from(self.file_cursor_position.0).unwrap(),
                rect.top
                    + u16::try_from(self.file_cursor_position.1 - self.start_line).unwrap()
                    + header_height,
            );
            write!(
                stream,
                "{}{} {}",
                termion::cursor::Goto(visible_cursor_position.0, visible_cursor_position.1),
                termion::color::Bg(termion::color::White),
                termion::color::Bg(termion::color::Reset),
            )?;
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
            termion::event::Event::Key(key) => match key {
                termion::event::Key::Down => {
                    self.file_cursor_position.1 += 1;
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Left => {
                    self.file_cursor_position.0 -= 1;
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Right => {
                    self.file_cursor_position.0 += 1;
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Up => {
                    self.file_cursor_position.1 -= 1;
                    self.needs_paint.set(true);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn get_events(&self) -> Vec<Event> {
        Vec::new()
    }

    fn dispatch_events(&mut self, _: &[Event]) {}
}
