use super::component::Component;
use crate::event::Event;
use crate::terminal::Rect;
use std::io::Write;

pub struct DividerComponent {}

impl DividerComponent {
    pub fn new() -> DividerComponent {
        DividerComponent {}
    }
}

impl Component for DividerComponent {
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        for row in rect.top..=(rect.top + rect.height) {
            write!(
                stream,
                "{}{} ",
                termion::cursor::Goto(rect.left, row),
                termion::color::Bg(termion::color::Yellow)
            )?;
        }
        write!(stream, "{}", termion::color::Bg(termion::color::Reset))
    }

    fn dispatch_key(&mut self, key: termion::event::Key) -> bool {
        false
    }
    fn get_events(&self) -> Vec<Event> {
        Vec::new()
    }
    fn dispatch_events(&mut self, events: &[Event]) {}
}
