//! The global-active list (GNU ed 1.22.4 global.c).
//!
//! C keeps line node pointers, which follow a line wherever it moves. ed-rs
//! keeps line numbers instead, so every arm that adds, deletes, or moves
//! lines while `isglobal` is set calls the matching hook below. That keeps
//! the numbers pointing at the same lines GNU ed would still visit.

#![allow(dead_code)]

/// Lines still to visit, in buffer order. `None` is a line that left the
/// list (C sets the slot to 0). `idx` is the next slot to look at.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ActiveList {
    pub lines: Vec<Option<usize>>,
    pub idx: usize,
}

impl ActiveList {
    /// Ports global.c `clear_active_list`: empty the list and reset `idx`.
    pub fn clear(&mut self) {
        todo!("port global.c clear_active_list")
    }

    /// Ports global.c `set_active_node`: add line `addr` at the end.
    pub fn push(&mut self, addr: usize) {
        let _ = addr;
        todo!("port global.c set_active_node")
    }

    /// Ports global.c `next_active_node`: skip `None` slots and return the
    /// next line number, or None when the list is used up.
    pub fn next(&mut self) -> Option<usize> {
        todo!("port global.c next_active_node")
    }

    /// `count` new lines were added after line `after`. Every listed line
    /// above `after` moves down by `count`. New lines are not added.
    pub fn on_insert(&mut self, after: usize, count: usize) {
        let _ = (after, count);
        todo!("shift line numbers after an insert")
    }

    /// Lines `from..=to` were deleted. Ports global.c `unset_active_nodes`
    /// (those lines leave the list), and moves every listed line above `to`
    /// up by the number of deleted lines.
    pub fn on_delete(&mut self, from: usize, to: usize) {
        let _ = (from, to);
        todo!("port global.c unset_active_nodes for a delete")
    }

    /// Lines `from..=to` moved after line `dest`, as in buffer.c
    /// `move_lines`. The moved lines leave the list (move_lines calls
    /// unset_active_nodes on them when isglobal). Other listed lines get the
    /// numbers they have after the move.
    pub fn on_move(&mut self, from: usize, to: usize, dest: usize) {
        let _ = (from, to, dest);
        todo!("renumber after a move, as buffer.c move_lines does")
    }
}
