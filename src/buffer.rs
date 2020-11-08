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
const DEFAULT_INITIAL_CAPACITY: usize = 10 * 1024;

pub struct Buffer {
    buffer: Vec<u8>,
    left_string_range: Range<usize>,
    right_string_range: Range<usize>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer::with_initial_capacity(DEFAULT_INITIAL_CAPACITY)
    }

    pub fn with_initial_capacity(capacity: usize) -> Buffer {
        Buffer {
            buffer: vec![0; capacity],
            left_string_range: 0..0,
            right_string_range: capacity..capacity,
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

        self.buffer[self.left_string_range.end..self.left_string_range.end + num_bytes]
            .copy_from_slice(as_bytes);
        self.left_string_range.end += num_bytes;
    }

    pub fn move_cursor(&mut self, offset: isize) {
        if offset < 0 {
            self.move_cursor_left(-offset as usize);
        } else if offset > 0 {
            self.move_cursor_right(offset as usize);
        }
    }

    pub fn move_cursor_right(&mut self, number_of_characters: usize) {
        let (_, right) = self.get();
        let mut graphemes = right.grapheme_indices(true);

        // We want to move the cursor to *after* the number_of_characters'th character in the right string
        let maybe_target_cursor_character_index = graphemes.nth(number_of_characters);

        let target_cursor_buffer_index = match maybe_target_cursor_character_index {
            Some((index, _)) => self.right_string_range.start + index,
            None => self.right_string_range.end,
        };

        self.move_cursor_right_to(target_cursor_buffer_index)
    }

    pub fn move_cursor_left(&mut self, number_of_characters: usize) {
        let (left, _) = self.get();
        let mut graphemes = left.grapheme_indices(true);
        let maybe_target_cursor_character_index = graphemes.nth_back(number_of_characters - 1);

        let target_cursor_buffer_index = match maybe_target_cursor_character_index {
            Some((index, _)) => index,
            None => 0,
        };
        self.move_cursor_left_to(target_cursor_buffer_index);
    }

    pub fn move_cursor_to_beginning(&mut self) {
        self.move_cursor_left_to(0);
    }

    pub fn move_cursor_to_end(&mut self) {
        self.move_cursor_right_to(self.right_string_range.end - 1);
    }

    fn move_cursor_right_to(&mut self, target_cursor_buffer_index: usize) {
        let source_copy_range = self.right_string_range.start..target_cursor_buffer_index;
        let destination_copy_start_index = self.left_string_range.end;
        self.buffer
            .copy_within(source_copy_range.clone(), destination_copy_start_index);

        let num_copied_bytes = source_copy_range.len();
        let num_original_right_bytes = self.get().1.as_bytes().len();
        let num_remaining_right_bytes = num_original_right_bytes - num_copied_bytes;
        self.left_string_range.end = destination_copy_start_index + num_copied_bytes;
        self.right_string_range.start = self.right_string_range.end - num_remaining_right_bytes;
    }

    fn move_cursor_left_to(&mut self, target_cursor_buffer_index: usize) {
        let source_copy_range = target_cursor_buffer_index..self.left_string_range.end;
        let destination_copy_start_index = self.right_string_range.start - source_copy_range.len();
        self.buffer
            .copy_within(source_copy_range, destination_copy_start_index);

        self.left_string_range.end = target_cursor_buffer_index;
        self.right_string_range.start = destination_copy_start_index;
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::*;

    const TEST_CAPACITY: usize = 64;

    #[test]
    fn initialize() {
        let buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        assert_eq!(buffer.get(), ("", ""));
    }

    #[test]
    fn simple_insert() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("John is the best");
        assert_eq!(buffer.get(), ("John is the best", ""));
    }

    #[test]
    fn move_cursor_left_1() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("Let's move the cursor 😎");
        buffer.move_cursor(-1);
        assert_eq!(buffer.get(), ("Let's move the cursor ", "😎"));
    }

    #[test]
    fn move_cursor_left_2() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("Test out this weird character: a̐");
        buffer.move_cursor(-3);
        assert_eq!(buffer.get(), ("Test out this weird character", ": a̐"));
    }

    #[test]
    fn move_cursor_left_saturating() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("Not enough room");
        buffer.move_cursor(-1000);
        assert_eq!(buffer.get(), ("", "Not enough room"));
    }

    #[test]
    fn move_cursor_left_multiple_times() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("OneTwoThree");
        buffer.move_cursor(-5);
        buffer.move_cursor(-3);
        assert_eq!(buffer.get(), ("One", "TwoThree"));
    }

    #[test]
    fn move_cursor_nowhere() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("Don't even move the dang cursor");
        buffer.move_cursor(0);
        assert_eq!(buffer.get(), ("Don't even move the dang cursor", ""));
    }

    #[test]
    fn move_cursor_right_1() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("Let's move the cursor 😎");
        buffer.move_cursor(-4);
        buffer.move_cursor(2);
        assert_eq!(buffer.get(), ("Let's move the cursor", " 😎"));
    }

    #[test]
    fn move_cursor_right_2() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("Test out this weird character: a̐");
        buffer.move_cursor(-10);
        buffer.move_cursor(2);
        assert_eq!(buffer.get(), ("Test out this weird char", "acter: a̐"));
    }

    #[test]
    fn move_cursor_right_saturating() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("Cursor is already at the end");
        buffer.move_cursor(1);
        assert_eq!(buffer.get(), ("Cursor is already at the end", ""));
    }

    #[test]
    fn move_cursor_right_multiple_times() {
        let mut buffer = Buffer::with_initial_capacity(TEST_CAPACITY);
        buffer.insert_at_cursor("OneTwoThree");
        buffer.move_cursor(-1000);
        buffer.move_cursor(3);
        buffer.move_cursor(3);
        assert_eq!(buffer.get(), ("OneTwo", "Three"));
    }

    #[test]
    fn complex_1() {
        let mut buffer = Buffer::with_initial_capacity(1000);
        buffer.insert_at_cursor("This ");
        buffer.insert_at_cursor("is a test");
        buffer.insert_at_cursor("\nThat tests öut a møre complex\t\tscenario");
        buffer.move_cursor_to_beginning();
        buffer.insert_at_cursor("Put this at the front");
        buffer.move_cursor_right(10);
        buffer.insert_at_cursor("Last thing");
        assert_eq!(
            buffer.get(),
            (
                "Put this at the frontThis is a Last thing",
                "test\nThat tests öut a møre complex\t\tscenario"
            )
        );
    }
}
