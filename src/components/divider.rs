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
