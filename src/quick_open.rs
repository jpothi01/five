use crate::indexer::FileIndexEntry;
use crate::indexer::Index;

pub type QuickOpenResult = FileIndexEntry;

pub fn get_quick_open_results(index: &Index, query: &str) -> Vec<QuickOpenResult> {
    let normalized_query = query.to_lowercase();
    let mut result: Vec<QuickOpenResult> = Vec::new();
    if normalized_query.is_empty() {
        return result;
    }

    for index_entry in &index.files {
        if index_entry
            .normalized_filename
            .starts_with(&normalized_query)
        {
            result.push(index_entry.clone());
        }
    }

    result
}
