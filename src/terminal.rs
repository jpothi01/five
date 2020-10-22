use term_size;

pub fn get_terminal_size() -> (usize, usize) {
    term_size::dimensions().unwrap()
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub top: u16,
    pub left: u16,
    pub width: u16,
    pub height: u16,
}
