//! Interval parser for `\{m,n\}` (BRE) and `{m,n}` (ERE).

#![allow(dead_code)]

use super::RegexError;

/// Largest repeat count glibc accepts (RE_DUP_MAX).
pub const DUP_MAX: u32 = 0x7fff;

/// Parse the inside of an interval.
///
/// `chars` is the whole pattern. `pos` is the index just after the opening
/// `\{` (or `{` when `extended`). Accept `m`, `m,` and `m,n`. On success
/// return `(min, max, next)` where `max` is None for `m,` and `next` is the
/// index just after the closing `\}` (or `}`).
///
/// Errors: UnmatchedBrace when the close is missing, BadInterval for a
/// missing or non-digit count, `m > n`, or a count above [`DUP_MAX`].
pub fn parse_interval(
    chars: &[char],
    pos: usize,
    extended: bool,
) -> Result<(u32, Option<u32>, usize), RegexError> {
    let _ = (chars, pos, extended);
    todo!("parse an interval")
}
