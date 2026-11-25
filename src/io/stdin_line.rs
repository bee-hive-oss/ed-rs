//! Read one script line from stdin (io.c get_stdin_line).

#![allow(dead_code)]

use crate::commands::CommandError;
use std::io::BufRead;

/// Ports io.c `get_stdin_line`.
///
/// Read up to and including the next newline. Return `Ok(Some(line))` with
/// the trailing newline kept, and add 1 to `*linenum`. At end of input
/// return `Ok(None)`; a final line with no newline is discarded (but still
/// counted in `*linenum` when it had any text), as the C code does. A read
/// error is `Err(Message("Cannot read stdin"))`. Read bytes with
/// `read_until(b'\n', ..)` and convert with `io::bytes::decode`.
pub fn get_stdin_line(
    input: &mut dyn BufRead,
    linenum: &mut usize,
) -> Result<Option<String>, CommandError> {
    let _ = (input, linenum);
    todo!("port io.c get_stdin_line")
}
