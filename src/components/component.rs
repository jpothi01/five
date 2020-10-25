use crate::event::Event;
use crate::terminal::Rect;
use std::io::Write;

pub trait Component {
    fn needs_paint(&self) -> bool;
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()>;

    fn dispatch_event(&mut self, event: termion::event::Event) -> bool;

    fn get_events(&self) -> Vec<Event>;

    fn dispatch_events(&mut self, events: &[Event]);
}
