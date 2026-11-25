//! Parser for POSIX extended regular expressions (the -E flag).

#![allow(dead_code)]

use super::ast::Program;
use super::RegexError;

/// Parse an ERE into a [`Program`].
///
/// Syntax to support: literals; `.`; `[...]` through
/// `class::parse_bracket`; `*`, `+`, `?`; `{m,n}` through
/// `interval::parse_interval`; `( )` groups numbered by open paren; `|`
/// alternation; `^` and `$` always anchors; `\1` to `\9` back references;
/// and a backslash before any other character is that character.
///
/// Errors: TrailingBackslash, UnmatchedParen, UnmatchedCloseParen,
/// BadBackref, BadRepeat (a repeat operator with nothing before it), and
/// whatever the bracket and interval parsers return.
pub fn parse_ere(pattern: &str) -> Result<Program, RegexError> {
    let _ = pattern;
    todo!("parse an extended regular expression")
}
