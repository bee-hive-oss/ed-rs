//! The global-active list (GNU ed 1.22.6 global.c).
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
        self.lines.clear();
        self.idx = 0;
    }

    /// Ports global.c `set_active_node`: add line `addr` at the end.
    pub fn push(&mut self, addr: usize) {
        self.lines.push(Some(addr));
    }

    /// Ports global.c `next_active_node`: skip `None` slots and return the
    /// next line number, or None when the list is used up.
    pub fn next(&mut self) -> Option<usize> {
        while self.idx < self.lines.len() && self.lines[self.idx].is_none() {
            self.idx += 1;
        }
        let addr = self.lines.get(self.idx).copied().flatten();
        if addr.is_some() {
            self.idx += 1;
        }
        addr
    }

    /// `count` new lines were added after line `after`. Every listed line
    /// above `after` moves down by `count`. New lines are not added.
    pub fn on_insert(&mut self, after: usize, count: usize) {
        if count == 0 {
            return;
        }
        for n in self.lines.iter_mut().flatten() {
            if *n > after {
                *n += count;
            }
        }
    }

    /// Lines `from..=to` were deleted. Ports global.c `unset_active_nodes`
    /// (those lines leave the list), and moves every listed line above `to`
    /// up by the number of deleted lines.
    pub fn on_delete(&mut self, from: usize, to: usize) {
        let n = to + 1 - from;
        for slot in &mut self.lines {
            if let Some(addr) = *slot {
                if addr >= from && addr <= to {
                    *slot = None;
                } else if addr > to {
                    *slot = Some(addr - n);
                }
            }
        }
    }

    /// Lines `from..=to` moved after line `dest`, as in buffer.c
    /// `move_lines`. The moved lines leave the list (move_lines calls
    /// unset_active_nodes on them when isglobal). Other listed lines get the
    /// numbers they have after the move.
    pub fn on_move(&mut self, from: usize, to: usize, dest: usize) {
        let count = to + 1 - from;
        let stay = dest + 1 == from || dest == to;
        for slot in &mut self.lines {
            let Some(addr) = *slot else { continue };
            if addr >= from && addr <= to {
                *slot = None;
                continue;
            }
            if stay {
                continue;
            }
            if dest < from && addr > dest && addr < from {
                *slot = Some(addr + count);
            } else if dest > to && addr > to && addr <= dest {
                *slot = Some(addr - count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ActiveList;

    fn list(xs: &[usize]) -> ActiveList {
        let mut a = ActiveList::default();
        for x in xs {
            a.push(*x);
        }
        a
    }

    #[test]
    fn visits_and_renumbers() {
        let mut a = list(&[1, 2, 3, 4, 5]);
        assert_eq!(a.next(), Some(1));
        a.on_delete(2, 3);
        assert_eq!(a.lines, vec![Some(1), None, None, Some(2), Some(3)]);
        assert_eq!(a.next(), Some(2));
        let mut b = list(&[1, 2, 3, 4, 5]);
        b.on_insert(1, 2);
        assert_eq!(b.lines, vec![Some(1), Some(4), Some(5), Some(6), Some(7)]);
        let mut c = list(&[1, 2, 3, 4, 5]);
        c.on_move(4, 5, 1);
        assert_eq!(c.lines, vec![Some(1), Some(4), Some(5), None, None]);
        let mut d = list(&[1, 2, 3, 4, 5]);
        d.on_move(2, 3, 5);
        assert_eq!(d.lines, vec![Some(1), None, None, Some(2), Some(3)]);
        let mut e = list(&[1, 2, 3]);
        e.on_move(1, 2, 2);
        assert_eq!(e.lines, vec![None, None, Some(3)]);
        e.clear();
        assert!(e.next().is_none());
    }
}
