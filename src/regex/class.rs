//! Bracket expression parser (the `[...]` part of a pattern).
//!
//! Ports the bracket handling of POSIX regcomp as GNU ed 1.22.4 gets it from
//! glibc, for the cases ed scripts use: a leading `^`, a `]` first in the
//! list, ranges, and the classes `[:alpha:]` and the other eleven names.
//! Collating symbols `[.x.]` and equivalence classes `[=x=]` are accepted
//! for a single character only.
//!
//! Classes follow the C locale (the tests run with LC_ALL=C): only ASCII
//! characters are alpha, digit, and so on. A char from U+0080 to U+00FF is
//! a raw byte and belongs to no named class.

#![allow(dead_code)]

use super::ast::ClassSet;
use super::RegexError;

/// Parse a bracket expression.
///
/// `chars` is the whole pattern as characters. `pos` is the index just after
/// the opening `[`. On success return the set and the index just after the
/// closing `]`.
///
/// Errors: UnmatchedBracket when there is no closing `]`, BadClassName for
/// an unknown `[:name:]`, BadRange when a range ends before it starts, and
/// BadCollate for a multi-character `[.xx.]`.
pub fn parse_bracket(chars: &[char], pos: usize) -> Result<(ClassSet, usize), RegexError> {
    let _ = (chars, pos);
    todo!("parse a POSIX bracket expression")
}
