//! Delete a range of lines (the d command).
//!
//! Ports GNU ed 1.22.4 buffer.c delete_lines, which exec_command in
//! main_loop.c calls for the 'd' case after set_addr_range2 and the
//! command suffix. Default range and suffix checks stay in the
//! dispatcher.
//!
//! Fill this body with the Buffer methods from issue 2 (PR 4), in
//! particular Buffer::delete, which already sets the current line to
//! min(from, last) and marks the buffer dirty. Do not use the old
//! public fields on the skeleton Buffer from main; those go away when
//! issue 2 merges. Mutating methods on Buffer do not snapshot
//! themselves. The later command loop takes the snapshot before calling
//! this function.
//!
//! The C function also yanks the range into the cut buffer and updates
//! the global-command active list. Yank and the isglobal active-list
//! update are out of scope here; later x, y, and g tickets can add them.

#![allow(dead_code)]

use super::CommandError;
use crate::buffer::Buffer;

/// Delete the inclusive 1-indexed range `from..=to`.
///
/// After a successful delete, GNU ed sets the current line to
/// min(from, last) and marks the buffer dirty. A reversed range, address
/// 0, or a range past the last line is CommandError::InvalidAddress.
pub fn delete_lines(
    buffer: &mut Buffer,
    from: usize,
    to: usize,
) -> Result<(), CommandError> {
    let _ = (buffer, from, to);
    todo!("port GNU ed 1.22.4 buffer.c delete_lines")
}
