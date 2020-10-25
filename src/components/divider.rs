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
use crate::terminal::Rect;
use std::cell::Cell;
use std::io::Write;

pub struct DividerComponent {
    needs_paint: Cell<bool>,
}

impl DividerComponent {
    pub fn new() -> DividerComponent {
        DividerComponent {
            needs_paint: Cell::new(true),
        }
    }
}

impl Component for DividerComponent {
    fn needs_paint(&self) -> bool {
        self.needs_paint.take()
    }
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        for row in rect.top..=(rect.top + rect.height) {
            write!(
                stream,
                "{}{} ",
                termion::cursor::Goto(rect.left, row),
                termion::color::Bg(termion::color::Yellow)
            )?;
        }
        write!(stream, "{}", termion::color::Bg(termion::color::Reset))?;
        self.needs_paint.set(false);
        Ok(())
    }

    fn dispatch_event(&mut self, key: termion::event::Event) -> bool {
        false
    }
    fn get_events(&self) -> Vec<Event> {
        Vec::new()
    }
    fn dispatch_events(&mut self, events: &[Event]) {}
}
