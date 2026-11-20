//! Line buffer, marks, modified state, and undo.
//! Skeleton: fields are unused until this seat is implemented.
#![allow(dead_code)]

#[derive(Debug, Default)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub current: usize,
    pub modified: bool,
}
