//! Read a file or command output into the buffer (io.c read_file and
//! read_stream).

#![allow(dead_code)]

use crate::buffer::Buffer;
use crate::commands::CommandError;
use std::io::Write;

/// Binary-file state of the buffer (io.c and buffer.c statics).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BinaryState {
    /// The buffer holds a NUL byte from a read (C isbinary_). Reset by e
    /// and E.
    pub isbinary: bool,
    /// The last line came from a binary read with no final newline and has
    /// not been changed since (C unterminated_last_line()). w then writes it
    /// without a newline.
    pub unterminated: bool,
}

/// What a read did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOutcome {
    /// Lines added to the buffer.
    pub lines: usize,
    /// True when the file could be opened for reading but not for writing
    /// (C read_onlyp).
    pub read_only: bool,
}

/// Ports io.c `read_file`, `read_stream`, and `read_stream_line`.
///
/// `name` is a path (open it with `io::bytes::to_path`), or a shell
/// command when it starts with `!` (use `shell::shell_read` on the text
/// after the `!`). Read all bytes and split them into lines at each
/// newline. A NUL byte sets `bin.isbinary`. With `strip_cr`, remove a CR
/// only when it sits right before a newline. Decode each line with
/// `io::bytes::decode`. A last line with no newline is kept and counts as
/// "newline added".
///
/// Append the lines to `buffer` after `addr` with `Buffer::append`. Then put
/// the dirty flag back to what it was before the read (a read does not dirty
/// the buffer; the r command sets it). The current line is the last line
/// read, or `addr` when nothing was read.
///
/// When not `scripted`, follow read_stream exactly: print "Newline
/// inserted" when appending at the end after an unterminated last line,
/// else "Newline appended" when a newline was added and (not appending at
/// the end, or the buffer is not binary). Then print the byte count and a
/// newline (C `%lu\n`), counting one more byte in the case read_stream
/// does. Set `bin.unterminated` when appending at the end of a binary
/// buffer and a newline was added (or nothing was read), as read_stream
/// does; appending after an unterminated last line clears it.
///
/// Errors: `Message("Cannot open input file")` when the file or command
/// cannot be opened or run, `Message("Cannot read input file")` on a read
/// error. A missing file must not change the buffer.
pub fn read_file(
    buffer: &mut Buffer,
    bin: &mut BinaryState,
    name: &str,
    addr: usize,
    strip_cr: bool,
    scripted: bool,
    out: &mut dyn Write,
) -> Result<ReadOutcome, CommandError> {
    let _ = (buffer, bin, name, addr, strip_cr, scripted, out);
    todo!("port io.c read_file")
}
