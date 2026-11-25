//! Backslash pairs in a basic regular expression.
//!
//! glibc regcomp without REG_EXTENDED gives a backslash before some
//! characters a special meaning. parse_bre.rs calls [`classify_escape`] for
//! every backslash so the parser itself stays small.

#![allow(dead_code)]

use super::RegexError;

/// What a backslash pair means in a BRE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreEscape {
    /// `\(`
    GroupOpen,
    /// `\)`
    GroupClose,
    /// `\|` (GNU extension)
    Alt,
    /// `\+` (GNU extension)
    Plus,
    /// `\?` (GNU extension)
    Question,
    /// `\{`, the start of an interval
    Interval,
    /// `\1` to `\9`
    Backref(usize),
    /// Any other character, taken literally (`\.` is a dot, `\n` is n).
    Literal(char),
}

/// Classify the character at `pos`, which follows a backslash.
///
/// `pos` past the end is TrailingBackslash. A digit 1 to 9 greater than
/// `closed` (the number of groups already closed) is BadBackref, matching
/// regcomp's REG_ESUBREG.
pub fn classify_escape(chars: &[char], pos: usize, closed: usize) -> Result<BreEscape, RegexError> {
    let _ = (chars, pos, closed);
    todo!("classify a BRE backslash pair")
}
