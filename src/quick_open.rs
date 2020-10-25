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
