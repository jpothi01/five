use crate::components::component::Component;
use crate::components::divider::DividerComponent;
use crate::components::file_pane::FilePaneComponent;
use crate::components::file_view::FileViewComponent;
use crate::indexer::Indexer;
use crate::terminal::Rect;
use std::io::Write;
use termion::event::Key;

pub struct RootComponent {
    indexer: Indexer,
    file_pane: FilePaneComponent,
    file_view: FileViewComponent,
    divider: DividerComponent,
}

impl RootComponent {
    pub fn new(cwd: &str) -> RootComponent {
        RootComponent {
            indexer: Indexer::new(cwd),
            file_pane: FilePaneComponent::new(cwd).unwrap(),
            file_view: FileViewComponent::new(),
            divider: DividerComponent::new(),
        }
    }

    fn start_quick_open(&mut self) {
        match self.indexer.get_index() {
            Err(err) => println!("Could not get index: {}", err.message),
            Ok(index) => self.file_pane.start_quick_open(index),
        }
    }
}

impl Component for RootComponent {
    fn paint<Writer: Write>(&self, stream: &mut Writer, rect: Rect) -> std::io::Result<()> {
        let file_pane_rect = Rect {
            left: 1,
            top: 1,
            width: 20,
            height: rect.height,
        };
        let divider_rect = Rect {
            left: file_pane_rect.width + 1,
            top: 1,
            width: 1,
            height: rect.height,
        };
        let file_view_rect = Rect {
            left: file_pane_rect.width + divider_rect.width,
            top: 1,
            width: rect.width - file_pane_rect.width,
            height: rect.height,
        };
        self.file_pane.paint(stream, file_pane_rect)?;
        self.file_view.paint(stream, file_view_rect)?;
        self.divider.paint(stream, divider_rect)?;
        stream.flush()
    }

    fn dispatch_key(&mut self, key: Key) -> bool {
        if self.file_pane.dispatch_key(key) {
            return true;
        }
        match key {
            Key::Ctrl(c) => {
                if c == 'p' {
                    self.start_quick_open()
                }
            }
            _ => {}
        };
        true
    }
}
