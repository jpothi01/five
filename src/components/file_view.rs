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

use super::component::{Component, DispatchEventResult};
use crate::buffer::Buffer;
use crate::event::Event;
use crate::painting_utils::{paint_empty_lines, paint_truncated_text};
use crate::terminal::Rect;
use std::cell::Cell;
use std::convert::TryFrom;
use std::io::Write;
use termion;
use unicode_segmentation::UnicodeSegmentation;

pub struct FileViewComponent {
    content: String,
    num_content_lines: i64,
    file_path: String,
    start_line: i64,
    has_focus: bool,
    needs_paint: Cell<bool>,
    buffer: Buffer,
}

pub enum FileViewContent {
    TextFile(String, String),
    BinaryFile(String),
    Folder(String, Vec<String>),
}

impl FileViewComponent {
    pub fn new() -> FileViewComponent {
        FileViewComponent {
            content: String::new(),
            num_content_lines: 0,
            file_path: String::new(),
            start_line: 0,
            has_focus: false,
            needs_paint: Cell::new(true),
            buffer: Buffer::new(),
        }
    }

    pub fn set_content(&mut self, content: FileViewContent) {
        self.buffer.delete_all();
        match content {
            FileViewContent::TextFile(path, content) => {
                self.content = content;
                self.num_content_lines = i64::try_from(self.content.lines().count()).unwrap();
                self.file_path = path;
            }
            FileViewContent::BinaryFile(path) => {
                self.content = String::from("<binary file>");
                self.num_content_lines = 1;
                self.file_path = path;
            }
            FileViewContent::Folder(path, mut children) => {
                self.num_content_lines = i64::try_from(children.len()).unwrap();
                self.content = children
                    .iter_mut()
                    .map(|child| String::from("./") + child)
                    .collect::<Vec<String>>()
                    .join("\n");
                self.file_path = path;
            }
        };

        // TODO: this is pretty inefficient. There should be an optimized method to initialize a buffer with
        // content and put the cursor at the beginning instead of having to move cursor after initial insertion.
        self.buffer.insert_at_cursor(&self.content);
        self.buffer.move_cursor_to_beginning();
        self.start_line = 0;
        self.needs_paint.set(true);
    }

    pub fn set_has_focus(&mut self, focused: bool) {
        self.has_focus = focused;
        self.needs_paint.set(true);
    }

    pub fn get_buffer(&self) -> (&Buffer, String) {
        return (&self.buffer, self.file_path.clone());
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

        let total_num_lines = rect.height as usize - 1;
        let (before_cursor, after_cursor) = self.buffer.get();

        // This code is ugly because I'm trying to iterate over our lines iterators only once.
        let num_lines_to_skip = self.start_line;
        let mut current_line_index = 0i64;
        let mut num_painted_lines = 0usize;
        let mut last_before_cursor_line_length = 0usize;

        let row_offset = |current_line_index: i64, num_lines_to_skip: i64| {
            u16::try_from(current_line_index - num_lines_to_skip).unwrap()
        };

        let before_cursor_lines = before_cursor.lines();
        let mut first = true;
        for line in before_cursor_lines {
            if !first {
                current_line_index += 1;
            }

            first = false;
            if current_line_index < num_lines_to_skip {
                continue;
            }

            let row_offset = row_offset(current_line_index, num_lines_to_skip);
            write!(
                stream,
                "{}",
                termion::cursor::Goto(rect.left, rect.top + 1 + row_offset)
            )?;
            paint_truncated_text(stream, line, rect.width)?;

            num_painted_lines += 1;
            last_before_cursor_line_length = line.graphemes(true).count();
        }

        // Draw the cursor
        let draw_cursor = self.has_focus
            && current_line_index >= num_lines_to_skip
            && last_before_cursor_line_length < rect.width as usize;
        if draw_cursor {
            let column_offset = u16::try_from(last_before_cursor_line_length).unwrap();
            write!(
                stream,
                "{}{} {}",
                termion::cursor::Goto(
                    rect.left + column_offset,
                    rect.top + 1 + row_offset(current_line_index, num_lines_to_skip)
                ),
                termion::color::Bg(termion::color::White),
                termion::color::Bg(termion::color::Reset)
            )?;
            stream.flush().unwrap();
        }

        let after_cursor_lines = after_cursor.lines();
        let first_column_offset = u16::try_from(last_before_cursor_line_length).unwrap();
        let mut first = true;
        for line in after_cursor_lines {
            if !first {
                current_line_index += 1;
            }

            if current_line_index < num_lines_to_skip {
                continue;
            }

            let (column_offset, line_offset) = if first {
                if draw_cursor {
                    (first_column_offset + 1, 1usize)
                } else {
                    (first_column_offset, 0usize)
                }
            } else {
                (0u16, 0usize)
            };

            write!(
                stream,
                "{}",
                termion::cursor::Goto(
                    rect.left + column_offset,
                    rect.top + 1 + row_offset(current_line_index, num_lines_to_skip)
                )
            )?;
            stream.flush().unwrap();

            let first_char_to_paint = line.grapheme_indices(true).nth(line_offset);
            match first_char_to_paint {
                Some((i, _)) => {
                    paint_truncated_text(stream, &line[i..], rect.width - column_offset)?;
                }
                None => {
                    paint_truncated_text(stream, "", rect.width - column_offset)?;
                }
            }
            stream.flush().unwrap();

            num_painted_lines += 1;
            first = false;
        }

        if num_painted_lines < total_num_lines {
            let row_offset = row_offset(current_line_index, num_lines_to_skip);
            paint_empty_lines(
                stream,
                Rect {
                    top: rect.top + 1 + row_offset,
                    left: rect.left,
                    width: rect.width,
                    height: rect.height - row_offset + 1,
                },
            )?;
        }

        self.needs_paint.set(false);
        Ok(())
    }

    fn dispatch_event(&mut self, event: termion::event::Event) -> DispatchEventResult {
        let mut events = Vec::<Event>::new();
        let handled = match event {
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
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Left => {
                    self.buffer.move_cursor_left(1);
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Right => {
                    self.buffer.move_cursor_right(1);
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Up => {
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Char(c) => {
                    self.buffer.insert_at_cursor(&c.to_string());
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Backspace => {
                    self.buffer.delete_at_cursor(1);
                    self.needs_paint.set(true);
                    true
                }
                termion::event::Key::Esc => {
                    events.push(Event::FileViewLostFocus);
                    true
                }
                termion::event::Key::Ctrl(c) => {
                    if c == 's' {
                        events.push(Event::FileSaved);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            _ => false,
        };
        DispatchEventResult {
            handled: handled,
            events: events,
        }
    }

    fn dispatch_events(&mut self, _: &[Event]) {}
}
