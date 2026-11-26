//! Build the new text of one line for an s command (regex.c line_replace
//! and replace_matched_text).

#![allow(dead_code)]

use super::Regex;
use crate::commands::CommandError;

/// Ports regex.c `line_replace` with `replace_matched_text`.
///
/// Find matches of `re` in `line` from left to right. `snum` is the
/// occurrence to replace (1 is the first). `snum <= 0` replaces every
/// match. In the replacement template `&` is the whole match, `\1` to `\9`
/// are groups (only when the number is at most `re.nsub()`), and every other
/// backslash pair is the second character. After the first match, search the
/// rest of the line with `Regex::exec(rest, true)` (notbol) so `^` does not
/// match again. In global mode, an empty match at the start of the rest is
/// allowed once; a second one in a row is
/// `Message("Infinite substitution loop")`, as in the C code.
///
/// Return `Ok(Some(new_text))` if something was replaced and `Ok(None)` if
/// the line had no match or fewer than `snum` matches. The new text may
/// contain newlines (from a `\` newline in the template); the caller
/// splits it into lines.
pub fn line_replace(
    re: &Regex,
    template: &str,
    line: &str,
    snum: i32,
) -> Result<Option<String>, CommandError> {
    let global = snum <= 0;
    let mut rest = line;
    let mut out = String::new();
    let mut changed = false;
    let mut matchno = 0i32;
    let mut notbol = false;
    let mut saw_empty = false;
    while let Some(caps) = re.exec(rest, notbol) {
        let (so, eo) = caps[0].unwrap_or((0, 0));
        if global || { matchno += 1; matchno == snum } {
            changed = true;
            out.push_str(&rest[..so]);
            splice(&mut out, template, rest, &caps, re.nsub());
        } else {
            out.push_str(&rest[..eo]);
        }
        if global && eo == 0 {
            if saw_empty {
                return Err(CommandError::Message("Infinite substitution loop"));
            }
            saw_empty = true;
        }
        rest = &rest[eo..];
        if rest.is_empty() || (changed && !global) {
            break;
        }
        notbol = true;
    }
    if !changed {
        return Ok(None);
    }
    out.push_str(rest);
    Ok(Some(out))
}

fn splice(out: &mut String, template: &str, text: &str, caps: &[Option<(usize, usize)>], nsub: usize) {
    let mut it = template.chars();
    while let Some(c) = it.next() {
        if c == '&' {
            push_span(out, text, caps.first().copied().flatten());
        } else if c == '\\' {
            match it.next() {
                Some(d @ '1'..='9') => {
                    let n = (d as u8 - b'0') as usize;
                    if n <= nsub {
                        push_span(out, text, caps.get(n).copied().flatten());
                    } else {
                        out.push(d);
                    }
                }
                Some(d) => out.push(d),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
}

fn push_span(out: &mut String, text: &str, span: Option<(usize, usize)>) {
    if let Some((a, b)) = span {
        out.push_str(&text[a..b]);
    }
}

#[cfg(test)]
mod tests {
    use super::line_replace;
    use super::super::Regex;
    use crate::commands::CommandError::Message;

    fn sub(pat: &str, template: &str, line: &str, snum: i32) -> Result<Option<String>, crate::commands::CommandError> {
        line_replace(&Regex::compile(pat, false, false).unwrap(), template, line, snum)
    }

    #[test]
    fn replaces_like_ed() {
        assert_eq!(sub("a*", "X", "b", 0).unwrap_err(), Message("Infinite substitution loop"));
        assert_eq!(sub("a*", "X", "a", 0).unwrap().as_deref(), Some("X"));
        assert_eq!(sub("^", "#", "ab", 0).unwrap().as_deref(), Some("#ab"));
        assert_eq!(sub("$", "#", "ab", 0).unwrap().as_deref(), Some("ab#"));
        assert_eq!(sub("$", "#", "", 0).unwrap().as_deref(), Some("#"));
        assert_eq!(sub("a*", "X", "b", 1).unwrap().as_deref(), Some("Xb"));
        assert_eq!(sub("a", "X", "baa", 2).unwrap().as_deref(), Some("baX"));
        assert!(sub("a", "X", "baa", 3).unwrap().is_none());
        assert_eq!(sub("o", "&X", "foo", 1).unwrap().as_deref(), Some("foXo"));
        assert_eq!(sub(r"\(a\)\(b\)", r"\2\1", "abc", 1).unwrap().as_deref(), Some("bac"));
        assert_eq!(sub("a", r"\&", "ab", 1).unwrap().as_deref(), Some("&b"));
        assert_eq!(sub("a", r"\1", "a", 1).unwrap().as_deref(), Some("1"));
        assert_eq!(sub(r"\(a\)", r"\2", "ab", 1).unwrap().as_deref(), Some("2b"));
        assert_eq!(sub("x", "a\\\nb", "xy", 1).unwrap().as_deref(), Some("a\nby"));
    }
}
