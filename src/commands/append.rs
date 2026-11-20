//! Append lines after an address (the a command).
//!
//! Ports GNU ed 1.22.4 buffer.c append_lines with insert false, which
//! exec_command in main_loop.c calls for the a case. Insert lives in its
//! own file. The c command also calls append_lines after a delete; that
//! wiring stays in the dispatcher.
//!
//! Fill this body with the Buffer methods from issue 2 (PR 4), in
//! particular Buffer::append, which already applies the empty and
//! non-empty current-line and dirty-flag rules for a. Do not use the
//! skeleton Buffer fields from main; those go away when issue 2 merges.
//! Mutating methods on Buffer do not snapshot themselves. The later
//! command loop takes the snapshot before calling this function.
//!
//! The C isglobal path only changes where input is read from. Passing
//! any line iterator covers both stdin and a global command list.

#![allow(dead_code)]

use super::CommandError;
use crate::buffer::Buffer;

/// Read input lines and append them after `addr`.
///
/// `addr` is 0-indexed in the sense GNU ed uses: 0 means after the
/// beginning (before the first line). A lone input line that is "."
/// ends the insertion and is not stored, matching
/// `if( size == 2 && **ibufpp == '.' )`. End of input also ends the
/// insertion successfully (GNU returns true on EOF).
///
/// Input items should be a single line without a trailing newline. An
/// empty item is a blank line, not the terminator; only a line that is
/// exactly "." terminates.
///
/// Empty insertion: GNU ed sets the current line to `addr` and does not
/// dirty the buffer. After a non-empty insertion the current line is the
/// last inserted line and the buffer is dirty. Map a Buffer error to
/// CommandError in this file (for example InvalidAddress).
pub fn append_lines<I, S>(
    buffer: &mut Buffer,
    input: I,
    addr: usize,
) -> Result<(), CommandError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let _ = (buffer, addr);
    let _ = input;
    todo!("port GNU ed 1.22.4 buffer.c append_lines for a")
}
