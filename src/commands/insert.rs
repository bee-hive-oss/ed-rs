//! Insert lines before an address (the i command).
//!
//! Ports GNU ed 1.22.4 buffer.c append_lines with insert true, which
//! exec_command in main_loop.c calls for the i case. Append lives in its
//! own file.
//!
//! Fill this body with the Buffer methods from issue 2 (PR 4), in
//! particular Buffer::insert, which already applies the empty and
//! non-empty current-line and dirty-flag rules for i, including the
//! first-line placement before addr (C: `if( insert ) { insert = false;
//! if( current_addr_ > 0 ) --current_addr_; }` then add after that).
//! Do not use the skeleton Buffer fields from main; those go away when
//! issue 2 merges. Mutating methods on Buffer do not snapshot
//! themselves. The later command loop takes the snapshot before calling
//! this function.
//!
//! The C isglobal path only changes where input is read from. Passing
//! any line iterator covers both stdin and a global command list.

#![allow(dead_code)]

use super::CommandError;
use crate::buffer::Buffer;

/// Read input lines and insert them before `addr`.
///
/// `addr` is 0-indexed in the sense GNU ed uses: 0 means before the
/// first line, which is the same place as appending after 0. A lone
/// input line that is "." ends the insertion and is not stored, matching
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
pub fn insert_lines<I, S>(
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
    todo!("port GNU ed 1.22.4 buffer.c append_lines for i")
}
