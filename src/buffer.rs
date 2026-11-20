//! In-memory line buffer for ed.
//!
//! Lines are 1-indexed. Address 0 is the valid empty-buffer and
//! before-the-first-line address; it has no text. Marks `a`–`z` are line
//! numbers that follow their lines when text is inserted, deleted, or moved,
//! and are dropped when that line is deleted. A line may carry more than one
//! mark. Copying a line does not copy its marks.
//!
//! Undo is one snapshot. [`Buffer::snapshot`] replaces any previous snapshot.
//! [`Buffer::restore`] swaps the live buffer with that snapshot, so a second
//! restore puts the modified buffer back. Restoring with no snapshot is an
//! error. Mutating methods do not snapshot by themselves: a later command
//! takes the snapshot once, then applies the whole edit (a global command is
//! one undo).
//!
//! This module is not called from `main` yet. In a binary crate that still
//! counts as dead code, so the allow stays until the command seat wires it in.

#![allow(dead_code)]

use std::fmt;

/// Error from a buffer operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferError {
    /// Address is outside `0..=last`, the range is reversed, or a mark is unset.
    InvalidAddress,
    /// Mark name is not `a`–`z`.
    InvalidMark,
    /// Move destination lies strictly inside the source range.
    InvalidDestination,
    /// [`Buffer::restore`] was called with no snapshot.
    NothingToUndo,
}

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidAddress => "Invalid address",
            Self::InvalidMark => "Invalid mark character",
            Self::InvalidDestination => "Invalid destination",
            Self::NothingToUndo => "Nothing to undo",
        })
    }
}

impl std::error::Error for BufferError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    lines: Vec<String>,
    current: usize,
    modified: bool,
    marks: [Option<usize>; 26],
}

/// 1-indexed line buffer with marks, a dirty flag, and one undo snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    lines: Vec<String>,
    current: usize,
    modified: bool,
    marks: [Option<usize>; 26],
    undo: Option<Snapshot>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    /// Empty buffer. Current line and last line are both 0. Not modified.
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            current: 0,
            modified: false,
            marks: [None; 26],
            undo: None,
        }
    }

    /// Last line number (`$`). Zero when the buffer is empty.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// True when there are no lines. Address 0 is still valid.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Current line (`.`). May be 0.
    pub fn current(&self) -> usize {
        self.current
    }

    /// Last line (`$`). Same as [`Buffer::len`].
    pub fn last(&self) -> usize {
        self.lines.len()
    }

    /// Set `.` to `addr`. `addr` must be in `0..=last`. Does not dirty the buffer.
    pub fn set_current(&mut self, addr: usize) -> Result<(), BufferError> {
        if addr > self.lines.len() {
            return Err(BufferError::InvalidAddress);
        }
        self.current = addr;
        Ok(())
    }

    /// Dirty flag. File load clears it; edits set it; a later `w` of the whole
    /// buffer clears it through [`Buffer::set_modified`].
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Set the dirty flag without touching lines, marks, current line, or undo.
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    /// Text of line `addr` (`1..=last`). Address 0 has no text.
    pub fn line(&self, addr: usize) -> Result<&str, BufferError> {
        self.lines
            .get(addr.checked_sub(1).ok_or(BufferError::InvalidAddress)?)
            .map(String::as_str)
            .ok_or(BufferError::InvalidAddress)
    }

    /// Borrow every line in order. Index 0 of the slice is line 1.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Borrow an inclusive line range. Both ends must be real lines and `start <= end`.
    pub fn line_range(&self, start: usize, end: usize) -> Result<&[String], BufferError> {
        self.check_line_range(start, end)?;
        Ok(&self.lines[start - 1..end])
    }

    /// Replace the buffer with `lines`, as the `e` command does after a successful read.
    ///
    /// Each item is one line without its terminating newline. Current line becomes
    /// the last line (0 if empty). Marks, the dirty flag, and the undo snapshot are
    /// cleared.
    pub fn load<I, S>(&mut self, lines: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.lines = lines.into_iter().map(Into::into).collect();
        self.current = self.lines.len();
        self.modified = false;
        self.marks = [None; 26];
        self.undo = None;
    }

    /// Append `new_lines` after `addr` (`0..=last`), like the `a` command.
    ///
    /// Current line becomes `addr` when `new_lines` is empty (the buffer is not
    /// dirtied). Otherwise current line becomes the last appended line and the
    /// buffer is dirtied. Marks after `addr` shift down.
    pub fn append<S: AsRef<str>>(
        &mut self,
        addr: usize,
        new_lines: impl IntoIterator<Item = S>,
    ) -> Result<(), BufferError> {
        self.insert_after(addr, ingest(new_lines))
    }

    /// Insert `new_lines` before `addr`, like the `i` command.
    ///
    /// `addr` must be in `0..=last`. Address 0 inserts at the beginning (the same
    /// place as appending after 0). An empty insertion only sets the current line
    /// to `addr` and does not dirty the buffer. A non-empty insertion leaves the
    /// current line on the last inserted line.
    pub fn insert<S: AsRef<str>>(
        &mut self,
        addr: usize,
        new_lines: impl IntoIterator<Item = S>,
    ) -> Result<(), BufferError> {
        if addr > self.lines.len() {
            return Err(BufferError::InvalidAddress);
        }
        let lines = ingest(new_lines);
        self.current = addr;
        if lines.is_empty() {
            return Ok(());
        }
        let at = if addr > 0 { addr - 1 } else { 0 };
        self.insert_after(at, lines)
    }

    /// Delete inclusive range `start..=end`. Both ends must be real lines and
    /// `start <= end`.
    ///
    /// Current line becomes `start` (the next surviving line slides into that
    /// number), or the new last line when the tail was deleted, or 0 when the
    /// buffer is empty. Marks on the deleted lines are cleared; later marks
    /// move up. The buffer is dirtied.
    pub fn delete(&mut self, start: usize, end: usize) -> Result<(), BufferError> {
        self.check_line_range(start, end)?;
        self.lines.drain(start - 1..end);
        self.drop_marks_in(start, end);
        self.current = start.min(self.lines.len());
        self.modified = true;
        Ok(())
    }

    /// Replace inclusive range `start..=end` with `new_lines`, like the `c` command.
    ///
    /// An empty replacement deletes the range. Marks on the replaced lines are
    /// cleared. The current line is the last replacement line, or the same line
    /// `delete` would leave when the replacement is empty.
    pub fn replace<S: AsRef<str>>(
        &mut self,
        start: usize,
        end: usize,
        new_lines: impl IntoIterator<Item = S>,
    ) -> Result<(), BufferError> {
        self.check_line_range(start, end)?;
        let lines = ingest(new_lines);
        self.delete(start, end)?;
        let addr = self.current;
        if addr >= start {
            self.insert(addr, lines)
        } else {
            self.append(addr, lines)
        }
    }

    /// Join inclusive range `start..=end` into one line with no added separator.
    ///
    /// A one-line range is a no-op: current line, marks, and the dirty flag stay
    /// as they are. A wider join clears marks on every joined line (including the
    /// first), leaves the current line on the joined line, and dirties the buffer.
    pub fn join(&mut self, start: usize, end: usize) -> Result<(), BufferError> {
        self.check_line_range(start, end)?;
        if start == end {
            return Ok(());
        }
        let mut joined = String::new();
        for line in &self.lines[start - 1..end] {
            joined.push_str(line);
        }
        self.replace(start, end, [joined])
    }

    /// Copy inclusive range `start..=end` to after `dest` (`0..=last`), like `t`.
    ///
    /// Marks are not copied. Marks after `dest` shift to make room. Current line
    /// becomes the last copied line. The buffer is dirtied. `dest` may sit inside
    /// the source range; the original text is what gets copied.
    pub fn copy(&mut self, start: usize, end: usize, dest: usize) -> Result<(), BufferError> {
        self.check_line_range(start, end)?;
        if dest > self.lines.len() {
            return Err(BufferError::InvalidAddress);
        }
        let texts = self.lines[start - 1..end].to_vec();
        self.insert_after(dest, texts)
    }

    /// Move inclusive range `start..=end` to after `dest` (`0..=last`), like `m`.
    ///
    /// Marks on the moved lines follow them. A destination strictly inside the
    /// range (`start..end`) is [`BufferError::InvalidDestination`]. A destination
    /// of `start - 1` or `end` leaves the lines where they are, but still sets
    /// the current line to `end` and dirties the buffer (GNU ed does the same).
    /// Otherwise the current line becomes the last moved line in its new place.
    pub fn move_lines(&mut self, start: usize, end: usize, dest: usize) -> Result<(), BufferError> {
        self.check_line_range(start, end)?;
        if dest > self.lines.len() {
            return Err(BufferError::InvalidAddress);
        }
        if dest == start - 1 || dest == end {
            self.current = end;
            self.modified = true;
            return Ok(());
        }
        if (start..end).contains(&dest) {
            return Err(BufferError::InvalidDestination);
        }

        let moved_marks: Vec<(usize, usize)> = self
            .marks
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.and_then(|line| {
                    (line >= start && line <= end).then_some((index, line - start))
                })
            })
            .collect();
        let texts = self.lines[start - 1..end].to_vec();
        let count = texts.len();
        self.lines.drain(start - 1..end);
        self.drop_marks_in(start, end);
        let dest = if dest > end { dest - count } else { dest };
        self.lines.splice(dest..dest, texts);
        self.bump_marks_above(dest, count);
        for (index, offset) in moved_marks {
            self.marks[index] = Some(dest + 1 + offset);
        }
        self.current = dest + count;
        self.modified = true;
        Ok(())
    }

    /// Mark line `addr` (`1..=last`) with `a`–`z`.
    ///
    /// The letter moves if it already marked another line. Other letters on this
    /// line are left alone, so one line can hold several marks. Does not change
    /// the current line or the dirty flag.
    pub fn set_mark(&mut self, addr: usize, mark: char) -> Result<(), BufferError> {
        let index = mark_index(mark)?;
        if addr == 0 || addr > self.lines.len() {
            return Err(BufferError::InvalidAddress);
        }
        self.marks[index] = Some(addr);
        Ok(())
    }

    /// Line number currently marked `mark`, or an error if the mark is unset
    /// or the name is not `a`–`z`.
    pub fn get_mark(&self, mark: char) -> Result<usize, BufferError> {
        let index = mark_index(mark)?;
        self.marks[index].ok_or(BufferError::InvalidAddress)
    }

    /// True when [`Buffer::restore`] can swap back to a saved snapshot.
    pub fn has_snapshot(&self) -> bool {
        self.undo.is_some()
    }

    /// Drop the undo snapshot. [`Buffer::load`] does this on its own.
    pub fn clear_snapshot(&mut self) {
        self.undo = None;
    }

    /// Save lines, current line, marks, and the dirty flag.
    ///
    /// A second call replaces the previous snapshot. The live buffer is not
    /// changed. Mutating methods do not call this.
    pub fn snapshot(&mut self) {
        self.undo = Some(self.capture());
    }

    /// Swap the live buffer with the snapshot.
    ///
    /// Restores lines, current line, marks, and the dirty flag. The displaced
    /// state becomes the new snapshot, so a second restore redoes the edit.
    /// [`BufferError::NothingToUndo`] if no snapshot is stored.
    pub fn restore(&mut self) -> Result<(), BufferError> {
        let previous = self.undo.take().ok_or(BufferError::NothingToUndo)?;
        let now = self.capture();
        self.apply(previous);
        self.undo = Some(now);
        Ok(())
    }

    fn capture(&self) -> Snapshot {
        Snapshot {
            lines: self.lines.clone(),
            current: self.current,
            modified: self.modified,
            marks: self.marks,
        }
    }

    fn apply(&mut self, snap: Snapshot) {
        self.lines = snap.lines;
        self.current = snap.current;
        self.modified = snap.modified;
        self.marks = snap.marks;
    }

    fn check_line_range(&self, start: usize, end: usize) -> Result<(), BufferError> {
        if start == 0 || end == 0 || start > end || end > self.lines.len() {
            return Err(BufferError::InvalidAddress);
        }
        Ok(())
    }

    fn insert_after(&mut self, addr: usize, lines: Vec<String>) -> Result<(), BufferError> {
        if addr > self.lines.len() {
            return Err(BufferError::InvalidAddress);
        }
        self.current = addr;
        if lines.is_empty() {
            return Ok(());
        }
        let count = lines.len();
        self.lines.splice(addr..addr, lines);
        self.bump_marks_above(addr, count);
        self.current = addr + count;
        self.modified = true;
        Ok(())
    }

    /// Marks on lines `> after` move down by `count` (an insertion after `after`).
    fn bump_marks_above(&mut self, after: usize, count: usize) {
        for line in self.marks.iter_mut().flatten() {
            if *line > after {
                *line += count;
            }
        }
    }

    /// Clear marks in `start..=end` and pull later marks up by the width of that range.
    fn drop_marks_in(&mut self, start: usize, end: usize) {
        let width = end - start + 1;
        for slot in &mut self.marks {
            if let Some(line) = *slot {
                if line >= start && line <= end {
                    *slot = None;
                } else if line > end {
                    *slot = Some(line - width);
                }
            }
        }
    }
}

fn mark_index(mark: char) -> Result<usize, BufferError> {
    if mark.is_ascii_lowercase() {
        Ok((mark as u8 - b'a') as usize)
    } else {
        Err(BufferError::InvalidMark)
    }
}

fn ingest<S: AsRef<str>>(lines: impl IntoIterator<Item = S>) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| line.as_ref().to_string())
        .collect()
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
    fn empty_buffer_allows_address_zero() {
        let mut buf = Buffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.last(), 0);
        assert_eq!(buf.current(), 0);
        assert!(!buf.is_modified());
        assert!(buf.line(0).is_err());
        assert!(buf.line(1).is_err());
        assert!(buf.set_current(0).is_ok());
        assert!(buf.set_current(1).is_err());
    }

    #[test]
    fn load_sets_last_line_and_clears_edit_state() {
        let mut buf = loaded(&["a", "b", "c"]);
        assert_eq!(
            buf.lines(),
            &["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(buf.current(), 3);
        assert_eq!(buf.line(1).unwrap(), "a");
        assert_eq!(buf.line(3).unwrap(), "c");
        assert!(!buf.is_modified());
        buf.set_mark(2, 'a').unwrap();
        buf.snapshot();
        buf.append(3, ["z"]).unwrap();
        assert!(buf.is_modified());
        assert!(buf.has_snapshot());

        buf.load(["only"]);
        assert_eq!(buf.lines(), &["only".to_string()]);
        assert_eq!(buf.current(), 1);
        assert!(!buf.is_modified());
        assert!(buf.get_mark('a').is_err());
        assert!(!buf.has_snapshot());

        buf.load(Vec::<String>::new());
        assert!(buf.is_empty());
        assert_eq!(buf.current(), 0);
    }

    #[test]
    fn append_and_insert_place_lines_and_set_current() {
        let mut buf = loaded(&["a", "b", "c"]);
        buf.set_current(1).unwrap();

        buf.append(0, ["Z"]).unwrap();
        assert_eq!(buf.lines(), &["Z", "a", "b", "c"].map(str::to_string));
        assert_eq!(buf.current(), 1);
        assert!(buf.is_modified());

        buf.set_modified(false);
        buf.append(2, ["X", "Y"]).unwrap();
        // Z a X Y b c
        assert_eq!(
            buf.lines(),
            &["Z", "a", "X", "Y", "b", "c"].map(str::to_string)
        );
        assert_eq!(buf.current(), 4);
        assert!(buf.is_modified());

        // Empty append moves `.` and does not dirty the buffer.
        buf.set_modified(false);
        buf.append(1, Vec::<&str>::new()).unwrap();
        assert_eq!(buf.current(), 1);
        assert!(!buf.is_modified());
        assert_eq!(buf.len(), 6);

        buf.insert(3, ["I"]).unwrap();
        // Z a I X Y b c
        assert_eq!(buf.line(3).unwrap(), "I");
        assert_eq!(buf.line(4).unwrap(), "X");
        assert_eq!(buf.current(), 3);
        assert!(buf.is_modified());

        buf.set_modified(false);
        buf.insert(2, Vec::<&str>::new()).unwrap();
        assert_eq!(buf.current(), 2);
        assert!(!buf.is_modified());

        // Insert at 0 is the start of the buffer.
        buf.insert(0, ["S"]).unwrap();
        assert_eq!(buf.line(1).unwrap(), "S");
        assert_eq!(buf.current(), 1);

        assert!(buf.append(buf.len() + 1, ["no"]).is_err());
        assert!(buf.insert(buf.len() + 1, ["no"]).is_err());
    }

    #[test]
    fn delete_updates_current_and_rejects_bad_ranges() {
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.delete(2, 4).unwrap();
        assert_eq!(buf.lines(), &["a", "e"].map(str::to_string));
        assert_eq!(buf.current(), 2);
        assert!(buf.is_modified());

        buf.set_modified(false);
        buf.delete(2, 2).unwrap();
        assert_eq!(buf.lines(), &["a".to_string()]);
        assert_eq!(buf.current(), 1);

        buf.delete(1, 1).unwrap();
        assert!(buf.is_empty());
        assert_eq!(buf.current(), 0);

        let mut buf = loaded(&["a", "b", "c"]);
        let before = buf.clone();
        assert_eq!(buf.delete(0, 1), Err(BufferError::InvalidAddress));
        assert_eq!(buf.delete(2, 1), Err(BufferError::InvalidAddress));
        assert_eq!(buf.delete(1, 4), Err(BufferError::InvalidAddress));
        assert_eq!(buf, before);
    }

    #[test]
    fn replace_and_join() {
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.replace(2, 3, ["Z"]).unwrap();
        assert_eq!(buf.lines(), &["a", "Z", "d", "e"].map(str::to_string));
        assert_eq!(buf.current(), 2);
        assert!(buf.is_modified());

        buf.replace(3, 4, Vec::<&str>::new()).unwrap();
        assert_eq!(buf.lines(), &["a", "Z"].map(str::to_string));
        assert_eq!(buf.current(), 2);

        buf.replace(2, 2, ["Q", "R"]).unwrap();
        assert_eq!(buf.lines(), &["a", "Q", "R"].map(str::to_string));
        assert_eq!(buf.current(), 3);

        // Replacing the tail appends rather than inserting before a missing line.
        buf.replace(2, 3, ["T"]).unwrap();
        assert_eq!(buf.lines(), &["a", "T"].map(str::to_string));
        assert_eq!(buf.current(), 2);

        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.join(2, 4).unwrap();
        assert_eq!(buf.lines(), &["a", "bcd", "e"].map(str::to_string));
        assert_eq!(buf.current(), 2);

        buf.set_modified(false);
        buf.set_current(1).unwrap();
        buf.join(2, 2).unwrap();
        assert_eq!(buf.line(2).unwrap(), "bcd");
        assert_eq!(buf.current(), 1);
        assert!(!buf.is_modified());

        buf.join(1, 3).unwrap();
        assert_eq!(buf.lines(), &["abcde".to_string()]);
        assert_eq!(buf.current(), 1);
    }

    #[test]
    fn copy_and_move() {
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.copy(2, 3, 0).unwrap();
        assert_eq!(
            buf.lines(),
            &["b", "c", "a", "b", "c", "d", "e"].map(str::to_string)
        );
        assert_eq!(buf.current(), 2);
        assert!(buf.is_modified());

        // Destination inside the source copies the original text.
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.copy(2, 4, 3).unwrap();
        assert_eq!(
            buf.lines(),
            &["a", "b", "c", "b", "c", "d", "d", "e"].map(str::to_string)
        );
        assert_eq!(buf.current(), 6);

        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.move_lines(2, 3, 5).unwrap();
        assert_eq!(buf.lines(), &["a", "d", "e", "b", "c"].map(str::to_string));
        assert_eq!(buf.current(), 5);

        buf.move_lines(4, 5, 0).unwrap();
        assert_eq!(buf.lines(), &["b", "c", "a", "d", "e"].map(str::to_string));
        assert_eq!(buf.current(), 2);

        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_current(1).unwrap();
        buf.set_modified(false);
        // dest == end is a structural no-op that still dirties and moves `.`.
        buf.move_lines(2, 3, 3).unwrap();
        assert_eq!(buf.lines(), &["a", "b", "c", "d", "e"].map(str::to_string));
        assert_eq!(buf.current(), 3);
        assert!(buf.is_modified());

        buf.set_modified(false);
        buf.move_lines(2, 3, 1).unwrap();
        assert_eq!(buf.lines(), &["a", "b", "c", "d", "e"].map(str::to_string));
        assert_eq!(buf.current(), 3);
        assert!(buf.is_modified());

        assert_eq!(
            buf.move_lines(2, 4, 3),
            Err(BufferError::InvalidDestination)
        );
        assert_eq!(buf.move_lines(2, 4, 9), Err(BufferError::InvalidAddress));
    }

    /// Marks follow the line they name. Deleting that line clears them.
    /// Several letters may name the same line. A copy does not take the mark.
    #[test]
    fn marks_follow_lines_and_clear_on_delete() {
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_mark(2, 'a').unwrap();
        buf.set_mark(2, 'b').unwrap();
        buf.set_mark(5, 'c').unwrap();
        assert_eq!(buf.get_mark('a').unwrap(), 2);
        assert_eq!(buf.get_mark('b').unwrap(), 2);
        assert_eq!(buf.get_mark('c').unwrap(), 5);
        assert!(!buf.is_modified());
        assert_eq!(buf.current(), 5);

        // Moving the letter drops it from the old line.
        buf.set_mark(4, 'a').unwrap();
        assert_eq!(buf.get_mark('a').unwrap(), 4);
        assert_eq!(buf.get_mark('b').unwrap(), 2);

        assert_eq!(buf.set_mark(0, 'd'), Err(BufferError::InvalidAddress));
        assert_eq!(buf.set_mark(2, 'A'), Err(BufferError::InvalidMark));
        assert_eq!(buf.set_mark(2, '1'), Err(BufferError::InvalidMark));
        assert_eq!(buf.get_mark('d'), Err(BufferError::InvalidAddress));
        assert_eq!(buf.get_mark('A'), Err(BufferError::InvalidMark));

        // Insert before the marked line: the mark moves with the text.
        buf.insert(1, ["Z"]).unwrap();
        // Z a b c d e ; 'b' was on old 2 (b) and is now line 3; 'a' was on old 4 (d) now 5.
        assert_eq!(buf.get_mark('b').unwrap(), 3);
        assert_eq!(buf.line(buf.get_mark('b').unwrap()).unwrap(), "b");
        assert_eq!(buf.get_mark('a').unwrap(), 5);
        assert_eq!(buf.line(5).unwrap(), "d");
        assert_eq!(buf.get_mark('c').unwrap(), 6);

        // Delete the line named by 'b'. The mark disappears. Later marks shift up.
        buf.delete(3, 3).unwrap();
        assert_eq!(buf.get_mark('b'), Err(BufferError::InvalidAddress));
        assert_eq!(buf.get_mark('a').unwrap(), 4);
        assert_eq!(buf.line(4).unwrap(), "d");

        // Copy does not copy marks. The original keeps the letter.
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_mark(2, 'a').unwrap();
        buf.copy(2, 2, 4).unwrap();
        assert_eq!(buf.line(2).unwrap(), "b");
        assert_eq!(buf.line(5).unwrap(), "b");
        assert_eq!(buf.get_mark('a').unwrap(), 2);

        // A copy inserted before the marked line shifts the mark's number.
        let mut buf = loaded(&["a", "b", "c"]);
        buf.set_mark(2, 'a').unwrap();
        buf.copy(2, 2, 0).unwrap();
        assert_eq!(buf.lines(), &["b", "a", "b", "c"].map(str::to_string));
        assert_eq!(buf.get_mark('a').unwrap(), 3);

        // Move carries the mark to the line's new number.
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_mark(2, 'a').unwrap();
        buf.set_mark(5, 'z').unwrap();
        buf.move_lines(2, 3, 5).unwrap();
        // a d e b c
        assert_eq!(buf.get_mark('a').unwrap(), 4);
        assert_eq!(buf.line(4).unwrap(), "b");
        assert_eq!(buf.get_mark('z').unwrap(), 3);
        assert_eq!(buf.line(3).unwrap(), "e");

        // Join clears marks on every joined line. A mark past the range follows.
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_mark(1, 'a').unwrap();
        buf.set_mark(3, 'b').unwrap();
        buf.set_mark(5, 'c').unwrap();
        buf.join(1, 3).unwrap();
        assert_eq!(buf.lines(), &["abc", "d", "e"].map(str::to_string));
        assert_eq!(buf.get_mark('a'), Err(BufferError::InvalidAddress));
        assert_eq!(buf.get_mark('b'), Err(BufferError::InvalidAddress));
        assert_eq!(buf.get_mark('c').unwrap(), 3);

        // A one-line join does not clear marks.
        buf.set_mark(1, 'a').unwrap();
        buf.set_modified(false);
        buf.join(1, 1).unwrap();
        assert_eq!(buf.get_mark('a').unwrap(), 1);
        assert!(!buf.is_modified());

        // Replace clears marks on the lines it removes.
        buf.replace(1, 1, ["ZZ"]).unwrap();
        assert_eq!(buf.line(1).unwrap(), "ZZ");
        assert_eq!(buf.get_mark('a'), Err(BufferError::InvalidAddress));
    }

    #[test]
    fn dirty_flag_is_independent_of_current_line() {
        let mut buf = loaded(&["a", "b"]);
        assert!(!buf.is_modified());
        buf.set_current(0).unwrap();
        assert!(!buf.is_modified());
        buf.set_modified(true);
        assert!(buf.is_modified());
        buf.set_modified(false);
        assert!(!buf.is_modified());
        assert_eq!(buf.current(), 0);
    }

    /// One snapshot covers lines, current line, marks, and the dirty flag.
    /// Restore swaps with that snapshot. A new snapshot drops the older one.
    /// Restore with nothing saved is an error.
    #[test]
    fn undo_snapshot_swaps_full_state() {
        let mut buf = loaded(&["a", "b", "c", "d", "e"]);
        buf.set_current(3).unwrap();
        buf.set_mark(2, 'a').unwrap();
        buf.set_mark(4, 'b').unwrap();
        assert_eq!(buf.restore(), Err(BufferError::NothingToUndo));

        buf.snapshot();
        buf.delete(2, 2).unwrap();
        // a c d e ; current slides to 2; mark a is gone; mark b was on 4, now 3.
        assert_eq!(buf.lines(), &["a", "c", "d", "e"].map(str::to_string));
        assert_eq!(buf.current(), 2);
        assert!(buf.is_modified());
        assert!(buf.get_mark('a').is_err());
        assert_eq!(buf.get_mark('b').unwrap(), 3);

        buf.restore().unwrap();
        assert_eq!(buf.lines(), &["a", "b", "c", "d", "e"].map(str::to_string));
        assert_eq!(buf.current(), 3);
        assert!(!buf.is_modified());
        assert_eq!(buf.get_mark('a').unwrap(), 2);
        assert_eq!(buf.get_mark('b').unwrap(), 4);
        assert!(buf.has_snapshot());

        // Second restore redoes the delete (the snapshot is a swap).
        buf.restore().unwrap();
        assert_eq!(buf.lines(), &["a", "c", "d", "e"].map(str::to_string));
        assert_eq!(buf.current(), 2);
        assert!(buf.is_modified());
        assert!(buf.get_mark('a').is_err());

        // A new snapshot replaces the previous one.
        buf.snapshot();
        buf.append(buf.len(), ["z"]).unwrap();
        buf.restore().unwrap();
        assert_eq!(buf.lines(), &["a", "c", "d", "e"].map(str::to_string));
        assert!(buf.line(5).is_err());

        buf.clear_snapshot();
        assert_eq!(buf.restore(), Err(BufferError::NothingToUndo));
    }

    #[test]
    fn set_current_accepts_zero_and_last_only() {
        let mut buf = loaded(&["a", "b", "c"]);
        buf.set_current(0).unwrap();
        assert_eq!(buf.current(), 0);
        buf.set_current(3).unwrap();
        assert_eq!(buf.current(), 3);
        assert_eq!(buf.set_current(4), Err(BufferError::InvalidAddress));
        assert_eq!(buf.line_range(1, 3).unwrap().len(), 3);
        assert_eq!(buf.line_range(2, 2).unwrap(), &["b".to_string()]);
        assert!(buf.line_range(0, 1).is_err());
    }
}
