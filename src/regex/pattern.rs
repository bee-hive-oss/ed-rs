//! Copy a pattern out of the command buffer (regex.c extract_pattern).

#![allow(dead_code)]

use crate::commands::CommandError;

/// Ports regex.c `extract_pattern` and `parse_char_class`.
///
/// `input` is the command text just after the opening delimiter. Scan up to
/// the first `delimiter`, newline, or end of text. A `[` starts a bracket
/// expression that is skipped as a unit, so a delimiter inside it does not
/// end the pattern (`[]...]`, `[^]...]`, and `[:`, `[.`, `[=` pairs inside
/// it). A backslash skips the next character.
///
/// Return the pattern text (without the delimiter) and the byte offset in
/// `input` where scanning stopped. That offset points at the closing
/// delimiter, the newline, or the end.
///
/// Errors: `CommandError::Message("Unbalanced brackets ([])")` when a
/// bracket expression never closes, and
/// `CommandError::Message("Trailing backslash (\\)")` for a backslash right
/// before the newline or end.
pub fn extract_pattern(input: &str, delimiter: char) -> Result<(String, usize), CommandError> {
    let b = input.as_bytes();
    let mut i = 0;
    while i < b.len() && !stop(b[i], delimiter) {
        if b[i] == b'[' {
            let Some(end) = class_end(b, i + 1) else {
                return Err(CommandError::Message("Unbalanced brackets ([])"));
            };
            i = end;
        } else if b[i] == b'\\' {
            i += 1;
            if i >= b.len() || b[i] == b'\n' || b[i] == 0 {
                return Err(CommandError::Message("Trailing backslash (\\)"));
            }
        }
        i += 1;
    }
    Ok((input[..i].to_string(), i))
}

fn stop(c: u8, delimiter: char) -> bool {
    c == b'\n' || c == 0 || (delimiter.is_ascii() && c == delimiter as u8)
}

/// Index of the closing `]`, or None. `start` is the byte after `[`.
fn class_end(b: &[u8], mut p: usize) -> Option<usize> {
    let byte = |p: usize| b.get(p).copied().unwrap_or(0);
    if byte(p) == b'^' {
        p += 1;
    }
    if byte(p) == b']' {
        p += 1;
    }
    while byte(p) != b']' && byte(p) != b'\n' && byte(p) != 0 {
        if byte(p) == b'[' {
            let d = byte(p + 1);
            if d == b'.' || d == b':' || d == b'=' {
                p += 1;
                let mut c = byte(p + 1);
                p += 1;
                while byte(p) != b']' || c != d {
                    c = byte(p);
                    if c == b'\n' || c == 0 {
                        return None;
                    }
                    p += 1;
                }
            }
        }
        p += 1;
    }
    if byte(p) == b']' { Some(p) } else { None }
}

#[cfg(test)]
mod tests {
    use super::extract_pattern;
    use crate::commands::CommandError::Message;

    fn ok(input: &str) -> (String, usize) {
        extract_pattern(input, '/').unwrap()
    }

    #[test]
    fn brackets_and_escapes() {
        assert_eq!(ok("abc/def"), ("abc".into(), 3));
        assert_eq!(ok("ab\ncd").0, "ab");
        assert_eq!(ok("a\\/b/c"), ("a\\/b".into(), 4));
        assert_eq!(ok("[/]/x").0, "[/]");
        assert_eq!(ok("[]]/x").0, "[]]");
        assert_eq!(ok("[]a]/x").0, "[]a]");
        assert_eq!(ok("[[:digit:]]/x").0, "[[:digit:]]");
        assert_eq!(ok("[[:digit:]/]/x").0, "[[:digit:]/]");
        assert_eq!(ok("a\\[b/c").0, "a\\[b");
        assert_eq!(ok("[^]]/x").0, "[^]]");
        assert_eq!(ok("[[.a.]b]/x").0, "[[.a.]b]");
        assert_eq!(ok("[[=a=]]/x").0, "[[=a=]]");
        assert_eq!(ok("[^/]/x").0, "[^/]");
        assert_eq!(extract_pattern("[abc", '/').unwrap_err(), Message("Unbalanced brackets ([])"));
        assert_eq!(extract_pattern("[]/x", '/').unwrap_err(), Message("Unbalanced brackets ([])"));
        assert_eq!(extract_pattern("[[:alpha:]", '/').unwrap_err(), Message("Unbalanced brackets ([])"));
        assert_eq!(extract_pattern("foo\\", '/').unwrap_err(), Message("Trailing backslash (\\)"));
        assert_eq!(extract_pattern("foo\\\n", '/').unwrap_err(), Message("Trailing backslash (\\)"));
    }
}
