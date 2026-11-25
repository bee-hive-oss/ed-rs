//! Backtracking matcher: everything except `Node::Repeat`, which lives in
//! `repeat.rs`.
//!
//! Matching uses continuations. [`Matcher::run`] tries one node at `pos`
//! and, for each way it can match, calls the continuation with the position
//! after the match. If the continuation returns true the whole match
//! succeeded and `run` returns true. If it returns false, `run` backtracks
//! and tries the next way. Captures set on the way are undone on failure.

#![allow(dead_code)]

use super::ast::{Captures, Node};

/// Matching state for one `exec` attempt.
pub struct Matcher<'t> {
    pub text: &'t str,
    pub icase: bool,
    pub notbol: bool,
    /// Slot 0 is filled by the caller. Slots 1.. are groups.
    pub caps: Captures,
}

/// Continuation: called with the matcher and the position after a match.
pub type Cont<'c, 't> = &'c mut dyn FnMut(&mut Matcher<'t>, usize) -> bool;

impl<'t> Matcher<'t> {
    pub fn new(text: &'t str, icase: bool, notbol: bool, nsub: usize) -> Self {
        Matcher {
            text,
            icase,
            notbol,
            caps: vec![None; nsub + 1],
        }
    }

    /// Match `node` at byte offset `pos`, then call `k`.
    ///
    /// Char and Set consume one whole character (use `str::chars` on
    /// `&self.text[pos..]`). Compare case-insensitively when `icase`, ASCII
/// letters only (C locale). Any
    /// matches one character. Bol matches at 0 unless `notbol`, Eol at the
    /// end of the text. Empty matches without consuming. Concat chains its
    /// parts, Alt tries each branch in order, Group records its span in
    /// `caps[n]` when its inner node finishes and clears it again on
    /// backtrack, and Backref matches the text of an earlier group (and
    /// fails if that group did not take part). Repeat calls
    /// `repeat::run_repeat`.
    pub fn run(&mut self, node: &Node, pos: usize, k: Cont<'_, 't>) -> bool {
        let _ = (node, pos, k);
        todo!("match one node with continuations")
    }
}
