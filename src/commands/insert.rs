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
    // GNU ed reads until a lone "." or EOF. The "." is not stored. EOF is
    // success. Buffer::insert places the first line before `addr` (address
    // 0 is the start of the buffer) and applies the empty and non-empty
    // current-line and dirty-flag rules. It does not take an undo snapshot.
    let mut lines = Vec::new();
    for item in input {
        let line = item.as_ref();
        if line == "." {
            break;
        }
        lines.push(line.to_string());
    }
    buffer
        .insert(addr, lines)
        .map_err(|_| CommandError::InvalidAddress)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded(lines: &[&str]) -> Buffer {
        let mut buf = Buffer::new();
        buf.load(lines.iter().copied());
        buf
    }

    #[test]
    fn inserts_before_addr_until_a_lone_dot() {
        let mut buf = loaded(&["a", "b", "c"]);
        buf.set_current(1).unwrap();
        buf.set_modified(false);

        insert_lines(&mut buf, ["X", "Y", ".", "Z"], 2).unwrap();

        assert_eq!(buf.lines(), &["a", "X", "Y", "b", "c"].map(str::to_string));
        // The first inserted line sits before addr 2. Current is the last
        // inserted line.
        assert_eq!(buf.line(2).unwrap(), "X");
        assert_eq!(buf.current(), 3);
        assert_eq!(buf.line(3).unwrap(), "Y");
        assert!(buf.is_modified());
        assert!(!buf.has_snapshot());
    }

    #[test]
    fn insert_before_the_first_line_and_at_address_zero() {
        let mut buf = loaded(&["a", "b"]);

        insert_lines(&mut buf, ["Z", "."], 1).unwrap();
        assert_eq!(buf.lines(), &["Z", "a", "b"].map(str::to_string));
        assert_eq!(buf.current(), 1);

        insert_lines(&mut buf, ["S"], 0).unwrap();
        assert_eq!(buf.lines(), &["S", "Z", "a", "b"].map(str::to_string));
        assert_eq!(buf.current(), 1);
        assert!(buf.is_modified());
    }

    #[test]
    fn end_of_input_without_a_dot_still_succeeds() {
        let mut buf = loaded(&["a", "b"]);
        buf.set_modified(false);

        insert_lines(&mut buf, ["X", "Y"], 2).unwrap();

        assert_eq!(buf.lines(), &["a", "X", "Y", "b"].map(str::to_string));
        assert_eq!(buf.current(), 3);
        assert!(buf.is_modified());
    }

    #[test]
    fn a_blank_line_is_stored_and_only_an_exact_dot_terminates() {
        let mut buf = loaded(&["a"]);

        insert_lines(&mut buf, ["", ".", ".."], 1).unwrap();

        assert_eq!(buf.lines(), &["", "a"].map(str::to_string));
        assert_eq!(buf.current(), 1);

        insert_lines(&mut buf, ["..", " .", "."], 2).unwrap();
        assert_eq!(buf.lines(), &["", "..", " .", "a"].map(str::to_string));
        assert_eq!(buf.current(), 3);
    }

    #[test]
    fn empty_insertion_sets_current_and_does_not_dirty() {
        let mut buf = loaded(&["a", "b", "c"]);
        buf.set_current(3).unwrap();
        buf.set_modified(false);

        insert_lines(&mut buf, [".", "nope"], 2).unwrap();
        assert_eq!(buf.lines(), &["a", "b", "c"].map(str::to_string));
        assert_eq!(buf.current(), 2);
        assert!(!buf.is_modified());

        buf.set_current(3).unwrap();
        insert_lines(&mut buf, Vec::<&str>::new(), 0).unwrap();
        assert_eq!(buf.current(), 0);
        assert!(!buf.is_modified());
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn empty_insertion_leaves_an_already_dirty_buffer_dirty() {
        let mut buf = loaded(&["a", "b"]);
        buf.set_modified(true);

        insert_lines(&mut buf, ["."], 1).unwrap();

        assert!(buf.is_modified());
        assert_eq!(buf.current(), 1);
        assert_eq!(buf.lines(), &["a", "b"].map(str::to_string));
    }

    #[test]
    fn a_bad_address_is_invalid_and_leaves_the_buffer_unchanged() {
        let mut buf = loaded(&["a", "b"]);
        buf.set_current(2).unwrap();
        buf.set_modified(false);
        let before = buf.clone();

        let err = insert_lines(&mut buf, ["X", "."], 3).unwrap_err();
        assert_eq!(err, CommandError::InvalidAddress);
        assert_eq!(err.message(), "Invalid address");
        assert_eq!(buf, before);

        let mut empty = Buffer::new();
        assert_eq!(
            insert_lines(&mut empty, ["X"], 1),
            Err(CommandError::InvalidAddress)
        );
        assert_eq!(empty, Buffer::new());
    }

    #[test]
    fn marks_at_and_after_addr_shift_and_undo_is_left_alone() {
        let mut buf = loaded(&["a", "b", "c"]);
        buf.set_mark(1, 'a').unwrap();
        buf.set_mark(2, 'b').unwrap();
        buf.set_mark(3, 'c').unwrap();
        buf.snapshot();

        insert_lines(&mut buf, ["X", "Y", "."], 2).unwrap();

        assert_eq!(buf.lines(), &["a", "X", "Y", "b", "c"].map(str::to_string));
        assert_eq!(buf.get_mark('a').unwrap(), 1);
        assert_eq!(buf.get_mark('b').unwrap(), 4);
        assert_eq!(buf.get_mark('c').unwrap(), 5);
        assert_eq!(buf.current(), 3);
        assert!(buf.has_snapshot());

        buf.restore().unwrap();
        assert_eq!(buf.lines(), &["a", "b", "c"].map(str::to_string));
        assert_eq!(buf.current(), 3);
        assert!(!buf.is_modified());
    }
}
