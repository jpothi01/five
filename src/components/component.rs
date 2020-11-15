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
use crate::event::Event;
use crate::terminal::Rect;
use std::io::Write;

pub struct DispatchEventResult {
    pub handled: bool,
    pub events: Vec<Event>,
}

impl DispatchEventResult {
    pub fn empty() -> DispatchEventResult {
        DispatchEventResult {
            handled: false,
            events: vec![],
        }
    }
}

pub trait Component {
    fn needs_paint(&self) -> bool;
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()>;

    fn dispatch_event(&mut self, event: termion::event::Event) -> DispatchEventResult;

    fn dispatch_events(&mut self, events: &[Event]);
}
