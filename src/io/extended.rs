//! Lines continued with a backslash-newline (io.c get_extended_line).

#![allow(dead_code)]

use crate::commands::CommandError;

/// Ports io.c `get_extended_line` (with `trailing_escape`).
///
/// `first` is the current command line including its trailing newline. If
/// the characters before the newline end in an odd number of backslashes,
/// the newline is escaped: replace that final backslash (and, when
/// `strip_escaped_newlines`, also the newline) and append the next line from
/// `next_line` (which returns lines with their newline). Repeat while the
/// newly added line also ends in an escaped newline. A line shorter than two
/// characters never continues. If `next_line` returns None while more is
/// needed, return `Err(Message("Unexpected end-of-file"))`.
///
/// Return the joined text, still ending in a newline.
pub fn get_extended_line(
    first: &str,
    strip_escaped_newlines: bool,
    next_line: &mut dyn FnMut() -> Option<String>,
) -> Result<String, CommandError> {
    let _ = (first, strip_escaped_newlines, next_line);
    todo!("port io.c get_extended_line")
}
