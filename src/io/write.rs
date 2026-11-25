//! Write buffer lines to a file or a command (io.c write_file and
//! write_stream).

#![allow(dead_code)]

use crate::commands::CommandError;
use std::io::Write;

/// Ports io.c `write_file` and `write_stream`.
///
/// Write lines `range.0..=range.1` (from, to) of `lines` (1-indexed, each encoded with
/// `io::bytes::encode` and followed by a newline) to `name`. When
/// `unterminated_last` is true (the caller passes `ed.bin.isbinary &&
/// ed.bin.unterminated`), the buffer's last line gets no newline, as
/// write_stream does. A `!` first means a shell command: send the bytes to
/// `shell::shell_write` with the text after the `!`. Otherwise create or
/// truncate the file (open it with `io::bytes::to_path`), or append when `append` is true (the W command).
/// `from == 0` writes nothing (an empty buffer writes an empty file).
///
/// When not `scripted` print the byte count and a newline to `out` (C
/// `%lu\n`). Return the number of lines written: `to - from + 1` when
/// `from > 0 && from <= to`, else 0.
///
/// Errors: `Message("Cannot open output file")`,
/// `Message("Cannot write file")`, `Message("Cannot close output file")`.
pub fn write_file(
    lines: &[String],
    unterminated_last: bool,
    name: &str,
    append: bool,
    range: (usize, usize),
    scripted: bool,
    out: &mut dyn Write,
) -> Result<usize, CommandError> {
    let _ = (lines, unterminated_last, name, append, range, scripted, out);
    todo!("port io.c write_file")
}
