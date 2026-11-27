//! Delete a range of lines (the d command).
//!
//! Ports GNU ed 1.22.6 buffer.c delete_lines, which exec_command in
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
pub fn delete_lines(buffer: &mut Buffer, from: usize, to: usize) -> Result<(), CommandError> {
    // Buffer::delete is GNU ed 1.22.6 buffer.c delete_lines after the
    // cut-buffer yank and the isglobal active-list update, both of which
    // belong to later tickets: it rejects address 0, a reversed range, and
    // a range past the last line, then sets current to min(from, last) and
    // marks the buffer dirty. It does not take an undo snapshot.
    buffer
        .delete(from, to)
        .map_err(|_| CommandError::InvalidAddress)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Buffer, BufferError};

    fn loaded(lines: &[&str]) -> Buffer {
        let mut buf = Buffer::new();
        buf.load(lines.iter().copied());
        buf
    }

    #[test]
    fn deletes_an_inclusive_middle_range() {
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_current(1).unwrap();
        buf.set_modified(false);

        delete_lines(&mut buf, 2, 4).unwrap();

        assert_eq!(buf.lines(), &["a", "e"].map(str::to_string));
        // The next surviving line slides into `from`.
        assert_eq!(buf.current(), 2);
        assert_eq!(buf.line(2).unwrap(), "e");
        assert!(buf.is_modified());
        assert!(!buf.has_snapshot());
    }

    #[test]
    fn deleting_the_tail_sets_current_to_the_new_last_line() {
        let mut buf = loaded(&["a", "b", "c", "d"]);
        buf.set_current(1).unwrap();

        delete_lines(&mut buf, 3, 4).unwrap();

        assert_eq!(buf.lines(), &["a", "b"].map(str::to_string));
        // min(from, last) == min(3, 2).
        assert_eq!(buf.current(), 2);
        assert!(buf.is_modified());
    }

    #[test]
    fn deleting_every_line_leaves_address_zero() {
        let mut buf = loaded(&["a", "b", "c"]);

        delete_lines(&mut buf, 1, 3).unwrap();

        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.current(), 0);
        assert!(buf.is_modified());
    }

    #[test]
    fn single_line_delete_moves_current_onto_the_following_line() {
        let mut buf = loaded(&["a", "b", "c"]);

        delete_lines(&mut buf, 1, 1).unwrap();

        assert_eq!(buf.lines(), &["b", "c"].map(str::to_string));
        assert_eq!(buf.current(), 1);
        assert!(buf.is_modified());
    }

    #[test]
    fn current_before_the_range_still_becomes_min_from_last() {
        // GNU ed 1.22.6 assigns current_addr_ = min(from, last_addr_)
        // even when `.` was before the deleted range.
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_current(1).unwrap();
        buf.set_modified(false);

        delete_lines(&mut buf, 4, 5).unwrap();

        assert_eq!(buf.lines(), &["a", "b", "c"].map(str::to_string));
        assert_eq!(buf.current(), 3);
        assert!(buf.is_modified());
    }

    #[test]
    fn bad_ranges_are_invalid_address_and_leave_the_buffer_unchanged() {
        let mut buf = loaded(&["a", "b", "c"]);
        buf.set_current(2).unwrap();
        buf.set_mark(2, 'a').unwrap();
        buf.set_modified(false);
        let before = buf.clone();

        for (from, to) in [
            (0, 0),
            (0, 1),
            (0, 3),
            (2, 1),
            (1, 0),
            (1, 4),
            (4, 4),
            (3, 2),
        ] {
            let err = delete_lines(&mut buf, from, to).unwrap_err();
            assert_eq!(err, CommandError::InvalidAddress);
            assert_eq!(err.message(), "Invalid address");
            assert_eq!(buf, before);
        }

        let mut empty = Buffer::new();
        empty.set_modified(false);
        assert_eq!(
            delete_lines(&mut empty, 0, 0),
            Err(CommandError::InvalidAddress)
        );
        assert_eq!(
            delete_lines(&mut empty, 1, 1),
            Err(CommandError::InvalidAddress)
        );
        assert_eq!(empty, Buffer::new());
    }

    #[test]
    fn marks_on_deleted_lines_clear_and_later_marks_move_up() {
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_mark(1, 'a').unwrap();
        buf.set_mark(3, 'b').unwrap();
        buf.set_mark(3, 'c').unwrap();
        buf.set_mark(5, 'd').unwrap();

        delete_lines(&mut buf, 2, 4).unwrap();

        assert_eq!(buf.get_mark('a').unwrap(), 1);
        assert_eq!(buf.get_mark('b'), Err(BufferError::InvalidAddress));
        assert_eq!(buf.get_mark('c'), Err(BufferError::InvalidAddress));
        assert_eq!(buf.get_mark('d').unwrap(), 2);
        assert_eq!(buf.current(), 2);
    }

    #[test]
    fn does_not_replace_an_existing_undo_snapshot() {
        let mut buf = loaded(&["a", "b", "c"]);
        buf.snapshot();

        delete_lines(&mut buf, 2, 2).unwrap();
        assert_eq!(buf.lines(), &["a", "c"].map(str::to_string));
        assert!(buf.has_snapshot());

        buf.restore().unwrap();
        assert_eq!(buf.lines(), &["a", "b", "c"].map(str::to_string));
        assert_eq!(buf.current(), 3);
        assert!(!buf.is_modified());
    }
}
