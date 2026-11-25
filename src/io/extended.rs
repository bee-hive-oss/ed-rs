//! Lines continued with a backslash-newline (io.c get_extended_line).

#![allow(dead_code)]

use crate::commands::CommandError;

/// Ports io.c `get_extended_line` (with `trailing_escape`).
///
/// `first` is the current command line including its trailing newline. If
/// the characters before the newline end in an odd number of backslashes,
/// the newline is escaped: replace that final backslash (and, when
/// `strip_escaped_newlines`, also the newline) and append the next line from
/// `next_line` (which returns lines with their newline). Repeat while the
/// newly added line also ends in an escaped newline. A line shorter than two
/// characters never continues. If `next_line` returns None while more is
/// needed, return `Err(Message("Unexpected end-of-file"))`.
///
/// Return the joined text, still ending in a newline.
pub fn get_extended_line(
    first: &str,
    strip_escaped_newlines: bool,
    next_line: &mut dyn FnMut() -> Option<String>,
) -> Result<String, CommandError> {
    let Some(nl) = first.find('\n') else {
        return Ok(first.to_string());
    };
    let len = nl + 1;
    if len < 2 || !odd_trailing_escape(&first[..nl]) {
        return Ok(first[..len].to_string());
    }
    let mut buf = String::from(&first[..len]);
    strip_escape(&mut buf, strip_escaped_newlines);
    loop {
        let Some(next) = next_line() else {
            return Err(CommandError::Message("Unexpected end-of-file"));
        };
        if next.is_empty() {
            return Err(CommandError::Message("Unexpected end-of-file"));
        }
        let len2 = next.len();
        buf.push_str(&next);
        let end = buf.rfind('\n').unwrap_or(buf.len());
        if len2 < 2 || !odd_trailing_escape(&buf[..end]) {
            break;
        }
        strip_escape(&mut buf, strip_escaped_newlines);
    }
    Ok(buf)
}

/// Parity of trailing backslashes (io.c `trailing_escape`).
fn odd_trailing_escape(before_nl: &str) -> bool {
    let bytes = before_nl.as_bytes();
    let mut i = bytes.len();
    let mut odd = false;
    while i > 0 && bytes[i - 1] == b'\\' {
        odd = !odd;
        i -= 1;
    }
    odd
}

/// Replace the final backslash with a newline, then drop that newline when
/// `strip` is set (io.c `--len; buf[len-1] = '\n'`).
fn strip_escape(buf: &mut String, strip: bool) {
    buf.pop();
    buf.pop();
    if !strip {
        buf.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::get_extended_line;
    use crate::commands::CommandError;

    fn join(first: &str, strip: bool, more: &[&str]) -> Result<String, CommandError> {
        let mut rest = more.iter().map(|s| (*s).to_string());
        get_extended_line(first, strip, &mut || rest.next())
    }

    #[test]
    fn plain_line_is_unchanged() {
        assert_eq!(join("abc\n", true, &[]).unwrap(), "abc\n");
        assert_eq!(join("\\\n", false, &["x\n"]).unwrap(), "\nx\n");
    }

    #[test]
    fn even_backslashes_do_not_continue() {
        assert_eq!(join("\\\\\n", true, &["no\n"]).unwrap(), "\\\\\n");
    }

    #[test]
    fn strips_escaped_newlines_and_keeps_them() {
        assert_eq!(join("ab\\\n", true, &["cd\n"]).unwrap(), "abcd\n");
        assert_eq!(join("ab\\\n", false, &["cd\n"]).unwrap(), "ab\ncd\n");
        assert_eq!(join("a\\\n", true, &["b\\\n", "c\n"]).unwrap(), "abc\n");
    }

    #[test]
    fn short_line_never_continues() {
        assert_eq!(join("\n", true, &["x\n"]).unwrap(), "\n");
    }

    #[test]
    fn eof_while_pending_is_an_error() {
        let err = join("ab\\\n", true, &[]).unwrap_err();
        assert_eq!(err, CommandError::Message("Unexpected end-of-file"));
    }
}
