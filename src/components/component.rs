use crate::event::Event;
use crate::terminal::Rect;
use std::io::Write;

pub trait Component {
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()>;

    fn dispatch_key(&mut self, key: termion::event::Key) -> bool;

    fn get_events(&self) -> Vec<Event>;

    fn dispatch_events(&mut self, events: &[Event]);
}
