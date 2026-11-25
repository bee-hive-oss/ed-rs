//! Parser for POSIX basic regular expressions, with the GNU extensions
//! glibc gives `regcomp` without REG_EXTENDED.

#![allow(dead_code)]

use super::ast::Program;
use super::RegexError;

/// Parse a BRE into a [`Program`].
///
/// Syntax to support: literals; `.`; `[...]` through
/// `class::parse_bracket`; `*`; `\{m,n\}` through `interval::parse_interval`;
/// `\(` `\)` groups numbered by open paren; `\1` to `\9` back references;
/// `^` as an anchor only at the start of the pattern or after `\(` or `\|`,
/// and `$` only at the end or before `\)` or `\|` (elsewhere they are
/// literals); `*` first in the pattern or right after `\(` or `^` is a
/// literal; and the GNU operators `\+`, `\?`, `\|`. A backslash before any
/// other character is that character. `Node::Empty` stands for an empty
/// branch. Classify every backslash pair with
/// `bre_escape::classify_escape`.
///
/// Errors: TrailingBackslash, UnmatchedParen, UnmatchedCloseParen,
/// BadBackref (a number above the groups closed so far), BadRepeat, and
/// whatever the bracket and interval parsers return.
pub fn parse_bre(pattern: &str) -> Result<Program, RegexError> {
    let _ = pattern;
    todo!("parse a basic regular expression")
}
