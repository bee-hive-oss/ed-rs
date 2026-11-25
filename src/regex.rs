//! Basic regular expression matching and substitution.
//!
//! GNU ed 1.22.4 calls the C library regcomp and regexec. This crate has no
//! dependencies, so the engine is split into small files under src/regex/:
//! bracket expressions (class), intervals, a BRE parser, an ERE parser for
//! the -E flag, a backtracking matcher (matcher and repeat), and the ed
//! side of regex.c (pattern, state, search, replace, subst).
//!
//! This file owns the public face: [`Regex`] and [`RegexError`]. The syntax
//! tree types are in `ast` and are shared by every parser and the matcher.
//!
//! Matching works on `&str` and on whole characters, so a match never splits
//! a UTF-8 sequence. Offsets are byte offsets into the text.

#![allow(dead_code)]

pub mod ast;
pub mod bre_escape;
pub mod class;
pub mod interval;
pub mod matcher;
pub mod parse_bre;
pub mod parse_ere;
pub mod pattern;
pub mod repeat;
pub mod replace;
pub mod search;
pub mod state;
pub mod subst;
pub mod subst_re;

use ast::{Captures, Program};

/// Failure from compiling a pattern. Message text is the glibc regerror
/// text that GNU ed 1.22.4 shows after `H` or `h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexError {
    BadPattern,
    BadCollate,
    BadClassName,
    TrailingBackslash,
    BadBackref,
    UnmatchedBracket,
    UnmatchedParen,
    UnmatchedBrace,
    BadInterval,
    BadRange,
    BadRepeat,
    PrematureEnd,
    UnmatchedCloseParen,
}

impl RegexError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::BadPattern => "Invalid regular expression",
            Self::BadCollate => "Invalid collation character",
            Self::BadClassName => "Invalid character class name",
            Self::TrailingBackslash => "Trailing backslash",
            Self::BadBackref => "Invalid back reference",
            Self::UnmatchedBracket => "Unmatched [, [^, [:, [., or [=",
            Self::UnmatchedParen => "Unmatched ( or \\(",
            Self::UnmatchedBrace => "Unmatched \\{",
            Self::BadInterval => "Invalid content of \\{\\}",
            Self::BadRange => "Invalid range end",
            Self::BadRepeat => "Invalid preceding regular expression",
            Self::PrematureEnd => "Premature end of regular expression",
            Self::UnmatchedCloseParen => "Unmatched ) or \\)",
        }
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for RegexError {}

/// A compiled pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regex {
    pub program: Program,
    pub icase: bool,
}

impl Regex {
    /// Compile `pattern`: BRE when `extended` is false, ERE when true.
    ///
    /// Same as regcomp with REG_ICASE when `icase` and REG_EXTENDED when
    /// `extended`. Pick `parse_bre::parse_bre` or `parse_ere::parse_ere`.
    pub fn compile(pattern: &str, icase: bool, extended: bool) -> Result<Regex, RegexError> {
        let program = if extended {
            parse_ere::parse_ere(pattern)?
        } else {
            parse_bre::parse_bre(pattern)?
        };
        Ok(Regex { program, icase })
    }

    /// Number of parenthesized groups (C re_nsub).
    pub fn nsub(&self) -> usize {
        self.program.nsub
    }

    /// Leftmost match in `text`, like regexec. Slot 0 is the whole match and
    /// slots 1..=nsub are the groups (None when a group did not take part).
    /// `notbol` is REG_NOTBOL: `^` does not match at offset 0.
    pub fn exec(&self, text: &str, notbol: bool) -> Option<Captures> {
        let mut m = matcher::Matcher::new(text, self.icase, notbol, self.program.nsub);
        let mut pos = 0;
        loop {
            m.caps.fill(None);
            if m.run(&self.program.root, pos, &mut |mm, end| {
                mm.caps[0] = Some((pos, end));
                true
            }) {
                return Some(m.caps.clone());
            }
            let ch = text[pos..].chars().next()?;
            pos += ch.len_utf8();
        }
    }

    /// True when [`Regex::exec`] finds a match (regexec with no captures).
    pub fn is_match(&self, text: &str) -> bool {
        self.exec(text, false).is_some()
    }
}

/// Early placeholder kept so older callers still compile.
pub fn matches(_pattern: &str, _line: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::Regex;
    #[test]
    fn compile_and_leftmost() {
        let re = Regex::compile(r"\(a\+\)", false, false).unwrap();
        assert_eq!(re.exec("baa", false).unwrap()[1], Some((1, 3)));
        assert!(Regex::compile("^a", false, false).unwrap().exec("a", true).is_none());
        let ere = Regex::compile("A+", true, true).unwrap();
        assert_eq!(ere.exec("xa", false).unwrap()[0], Some((1, 2)));
        assert!(Regex::compile("*", false, true).is_err());
    }
}
