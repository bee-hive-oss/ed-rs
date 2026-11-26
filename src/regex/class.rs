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

use super::ast::{ClassItem, ClassSet, NamedClass};
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
    let mut i = pos;
    let mut negate = false;
    if chars.get(i) == Some(&'^') {
        negate = true;
        i += 1;
    }
    let mut items = Vec::new();
    let mut first = true;
    while i < chars.len() {
        if chars[i] == ']' && !first {
            return Ok((ClassSet { negate, items }, i + 1));
        }
        first = false;
        if special_open(chars, i) {
            let (item, next) = special(chars, i)?;
            items.push(item);
            i = next;
            continue;
        }
        let start = chars[i];
        i += 1;
        if chars.get(i) == Some(&'-')
            && chars.get(i + 1).is_some_and(|c| *c != ']' && !special_open(chars, i + 1))
        {
            let end = chars[i + 1];
            if (end as u32) < (start as u32) {
                return Err(RegexError::BadRange);
            }
            items.push(ClassItem::Range(start, end));
            i += 2;
        } else {
            items.push(ClassItem::Ch(start));
        }
    }
    Err(RegexError::UnmatchedBracket)
}

fn special_open(chars: &[char], i: usize) -> bool {
    chars.get(i) == Some(&'[') && matches!(chars.get(i + 1), Some('.' | ':' | '='))
}

fn special(chars: &[char], i: usize) -> Result<(ClassItem, usize), RegexError> {
    let kind = chars[i + 1];
    let mut j = i + 2;
    let body = j;
    while j + 1 < chars.len() {
        if chars[j] == kind && chars[j + 1] == ']' {
            let text: String = chars[body..j].iter().collect();
            let next = j + 2;
            return match kind {
                ':' => Ok((ClassItem::Named(class_name(&text)?), next)),
                _ => one_char(&text).map(|c| (ClassItem::Ch(c), next)),
            };
        }
        j += 1;
    }
    Err(RegexError::UnmatchedBracket)
}
fn one_char(text: &str) -> Result<char, RegexError> {
    let mut cs = text.chars();
    match (cs.next(), cs.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(RegexError::BadCollate),
    }
}

fn class_name(name: &str) -> Result<NamedClass, RegexError> {
    Ok(match name {
        "alnum" => NamedClass::Alnum,
        "alpha" => NamedClass::Alpha,
        "blank" => NamedClass::Blank,
        "cntrl" => NamedClass::Cntrl,
        "digit" => NamedClass::Digit,
        "graph" => NamedClass::Graph,
        "lower" => NamedClass::Lower,
        "print" => NamedClass::Print,
        "punct" => NamedClass::Punct,
        "space" => NamedClass::Space,
        "upper" => NamedClass::Upper,
        "xdigit" => NamedClass::Xdigit,
        _ => return Err(RegexError::BadClassName),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_bracket;
    use crate::regex::ast::{ClassItem, NamedClass};
    use crate::regex::RegexError;

    fn go(s: &str) -> Result<(bool, Vec<ClassItem>, usize), RegexError> {
        let c: Vec<char> = s.chars().collect();
        parse_bracket(&c, 1).map(|(set, n)| (set.negate, set.items, n))
    }

    #[test]
    fn caret_literal_bracket_range_and_class() {
        let (neg, items, n) = go("[^a-c]").unwrap();
        assert!(neg);
        assert_eq!(items, vec![ClassItem::Range('a', 'c')]);
        assert_eq!(n, 6);
        let (_, items, _) = go("[]]").unwrap();
        assert_eq!(items, vec![ClassItem::Ch(']')]);
        let (_, items, _) = go("[[:digit:][]").unwrap();
        assert_eq!(
            items,
            vec![ClassItem::Named(NamedClass::Digit), ClassItem::Ch('[')]
        );
        let (_, items, _) = go("[[.a.]b]").unwrap();
        assert_eq!(items, vec![ClassItem::Ch('a'), ClassItem::Ch('b')]);
    }

    #[test]
    fn errors() {
        assert_eq!(go("[xyx").unwrap_err(), RegexError::UnmatchedBracket);
        assert_eq!(go("[h[.]").unwrap_err(), RegexError::UnmatchedBracket);
        assert_eq!(go("[h[:").unwrap_err(), RegexError::UnmatchedBracket);
        assert_eq!(go("[h[=").unwrap_err(), RegexError::UnmatchedBracket);
        assert_eq!(go("[[:nope:]]").unwrap_err(), RegexError::BadClassName);
        assert_eq!(go("[z-a]").unwrap_err(), RegexError::BadRange);
        assert_eq!(go("[[.ab.]]").unwrap_err(), RegexError::BadCollate);
    }
}
