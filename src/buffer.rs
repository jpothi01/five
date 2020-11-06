// Implementation of a gap buffer.
// A gap buffer is a dynamic array divided into three regions:
// - "Left string"
// - "Gap"
// - "Right string"

// As a diagram, where 'X' means "part of the Gap":
// [a|b|c|X|X|X|d|e|f]
//  0 1 2 3 4 5 6 7 8

// Here,
// [0, 2] is the Left string
// [3, 5] is the Gap
// [6, 8] is the Right string

// Hidden in this data structure is the concept of the "cursor". At all times, the
// position of the cursor is the start index of the gap. The Buffer exposes an API
// called insert(), which is used to write new characters into the buffer starting
// at the cursor position. Inserted characters become part of the Left string.

// Before:
// [a|b|c|X|X|X|d|e|f]
//  0 1 2 3 4 5 6 7 8

// After inserting the string 'hi':
// [a|b|c|h|i|X|d|e|f]
//  0 1 2 3 4 5 6 7 8

// Now the cursor is at index 5, which is the new start of the Gap.

// The Buffer also exposes an API to move the cursor, called move_cursor(). When the
// cursor is moved, we must also move the Gap. This is done by copying the necessary
// elements from the Left string to the Right string, or vice versa:

// Cursor at index 5:
// [a|b|c|h|i|X|d|e|f]
//  0 1 2 3 4 5 6 7 8

// Move Cursor to index 6, by copying 'd' to index 5:
// [a|b|c|h|i|d|X|e|f]
//  0 1 2 3 4 5 6 7 8

// Move Cursor to index 0, by copying 'abchi' to [1, 6]:
// [X|a|b|c|h|i|d|e|f]
//  0 1 2 3 4 5 6 7 8

// Note that the memory location for X may contain any character. We do not overwrite
// gap locations with a sentinel for performance reasons.

// There will be times that inserted characters do not fit in the gap. In this case, we must
// "grow" the gap by reallocating the buffer to enlarge the gap:

// Before:
// [a|X|b|c|h|i|d|e|f]
//  0 1 2 3 4 5 6 7 8

// Insert 'hello':
// Step 1, reallocate:
// [X|X|X|X|X|X|X|X|X|X|X|X|X|X|X|X|X|X]
//  0 1 2 3 4 5 6 7 8 9 a b c d e f . .
// Step 2, copy Left and Right strings to the sides:
// [a|X|X|X|X|X|X|X|X|X|X|b|c|h|i|d|e|f]
//  0 1 2 3 4 5 6 7 8 9 a b c d e f . .
// Step 3, insert the string:
// [a|h|e|l|l|o|X|X|X|X|X|b|c|h|i|d|e|f]
//  0 1 2 3 4 5 6 7 8 9 a b c d e f . .

// OK, this is great and all, but how do I render the string?
// The Buffer exposes a get() API that returns the Left and Right string slices. The client
// can either combine these into one string by concatenating them (requiring an allocation),
// or use them separately. Usually, the client will just be rendering Left and Right next to
// each other with the cursor in the middle, requiring no extra allocations.

use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
const INITIAL_CAPACITY: usize = 10 * 1024;

pub struct Buffer {
    buffer: Vec<u8>,
    left_string_range: Range<usize>,
    right_string_range: Range<usize>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buffer: vec![0; INITIAL_CAPACITY],
            left_string_range: 0..0,
            right_string_range: (INITIAL_CAPACITY - 1)..(INITIAL_CAPACITY - 1),
        }
    }
}

impl Buffer {
    pub fn get(&self) -> (&str, &str) {
        unsafe {
            (
                std::str::from_utf8_unchecked(&self.buffer[self.left_string_range.clone()]),
                std::str::from_utf8_unchecked(&self.buffer[self.right_string_range.clone()]),
            )
        }
    }

    pub fn insert_at_cursor(&mut self, characters: &str) {
        let as_bytes = characters.as_bytes();
        let num_bytes = as_bytes.len();
        let gap_size = self.right_string_range.start - self.left_string_range.end;
        if num_bytes > gap_size {
            panic!("Not implemented");
        }

        self.buffer[self.left_string_range.end..num_bytes].copy_from_slice(as_bytes);
        self.left_string_range.end += num_bytes;
    }

    pub fn move_cursor(&mut self, offset: isize) {
        if offset < 0 {
            self.move_cursor_left(-offset);
        } else if offset > 0 {
            self.move_cursor_right(offset);
        }
    }

    fn move_cursor_right(&mut self, offset: isize) {}

    fn move_cursor_left(&mut self, number_of_characters: isize) {
        debug_assert!(number_of_characters > 0);

        let (left, right) = self.get();
        let mut graphemes = left.grapheme_indices(true);
        let target_character_index = graphemes
            .nth_back((number_of_characters - 1) as usize)
            .unwrap()
            .0;
        let source_copy_range = target_character_index..self.left_string_range.end;
        let destination_copy_start_index = self.right_string_range.start - source_copy_range.len();
        self.buffer
            .copy_within(source_copy_range, destination_copy_start_index);

        self.left_string_range.end = target_character_index;
        self.right_string_range.start = destination_copy_start_index;
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::*;

    #[test]
    fn initialize() {
        let buffer = Buffer::new();
        assert_eq!(buffer.get(), ("", ""));
    }

    #[test]
    fn simple_insert() {
        let mut buffer = Buffer::new();
        buffer.insert_at_cursor("John is the best");
        assert_eq!(buffer.get(), ("John is the best", ""));
    }

    #[test]
    fn move_cursor_left_1() {
        let mut buffer = Buffer::new();
        buffer.insert_at_cursor("Let's move the cursor 😎");
        buffer.move_cursor(-1);
        assert_eq!(buffer.get(), ("Let's move the cursor ", "😎"));
    }

    #[test]
    fn move_cursor_left_2() {
        let mut buffer = Buffer::new();
        buffer.insert_at_cursor("Test out this weird character: a̐");
        buffer.move_cursor(-3);
        assert_eq!(buffer.get(), ("Test out this weird character", ": a̐"));
    }
}
