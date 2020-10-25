use crate::indexer::FileIndexEntry;

#[derive(Clone)]
pub enum Event {
    FileItemSelected(FileIndexEntry),
}
