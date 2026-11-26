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
    let (min, mut i) = number(chars, pos)?;
    let mut max = Some(min);
    if chars.get(i) == Some(&',') {
        i += 1;
        if chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
            let (n, j) = number(chars, i)?;
            if n < min {
                return Err(RegexError::BadInterval);
            }
            max = Some(n);
            i = j;
        } else if at_close(chars, i, extended).is_some() {
            max = None;
        } else if chars.get(i).is_none() || incomplete_close(chars, i, extended) {
            return Err(RegexError::UnmatchedBrace);
        } else {
            return Err(RegexError::BadInterval);
        }
    }
    match at_close(chars, i, extended) {
        Some(next) => Ok((min, max, next)),
        None if chars.get(i).is_none() || incomplete_close(chars, i, extended) => {
            Err(RegexError::UnmatchedBrace)
        }
        None => Err(RegexError::BadInterval),
    }
}

fn number(chars: &[char], mut i: usize) -> Result<(u32, usize), RegexError> {
    if !chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
        return Err(RegexError::BadInterval);
    }
    let mut n: u64 = 0;
    while chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
        n = n
            .saturating_mul(10)
            .saturating_add(u64::from(chars[i] as u8 - b'0'));
        if n > u64::from(DUP_MAX) {
            return Err(RegexError::BadInterval);
        }
        i += 1;
    }
    Ok((n as u32, i))
}

fn at_close(chars: &[char], i: usize, extended: bool) -> Option<usize> {
    if extended {
        (chars.get(i) == Some(&'}')).then_some(i + 1)
    } else if chars.get(i) == Some(&'\\') && chars.get(i + 1) == Some(&'}') {
        Some(i + 2)
    } else {
        None
    }
}

fn incomplete_close(chars: &[char], i: usize, extended: bool) -> bool {
    !extended && chars.get(i) == Some(&'\\') && chars.get(i + 1).is_none()
}

#[cfg(test)]
mod tests {
    use super::{parse_interval, DUP_MAX};
    use crate::regex::RegexError;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn m_mcomma_and_mn() {
        let c = chars(r"2\}");
        assert_eq!(parse_interval(&c, 0, false).unwrap(), (2, Some(2), 3));
        let c = chars(r"2,\}");
        assert_eq!(parse_interval(&c, 0, false).unwrap(), (2, None, 4));
        let c = chars("3,5}");
        assert_eq!(parse_interval(&c, 0, true).unwrap(), (3, Some(5), 4));
        let c = chars("0007}");
        assert_eq!(parse_interval(&c, 0, true).unwrap(), (7, Some(7), 5));
    }

    #[test]
    fn missing_close_bad_count_and_dup_max() {
        assert_eq!(parse_interval(&chars("2"), 0, true), Err(RegexError::UnmatchedBrace));
        assert_eq!(parse_interval(&chars("2,"), 0, true), Err(RegexError::UnmatchedBrace));
        assert_eq!(parse_interval(&chars("}"), 0, true), Err(RegexError::BadInterval));
        assert_eq!(parse_interval(&chars(",2}"), 0, true), Err(RegexError::BadInterval));
        assert_eq!(parse_interval(&chars("5,3}"), 0, true), Err(RegexError::BadInterval));
        assert_eq!(parse_interval(&chars("2a}"), 0, true), Err(RegexError::BadInterval));
        let big = format!("{},1}}", u64::from(DUP_MAX) + 1);
        assert_eq!(parse_interval(&chars(&big), 0, true), Err(RegexError::BadInterval));
        let ok = format!("{DUP_MAX}}}");
        assert_eq!(parse_interval(&chars(&ok), 0, true).unwrap().0, DUP_MAX);
    }
}
