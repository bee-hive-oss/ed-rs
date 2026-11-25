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
    let _ = (re, template, line, snum);
    todo!("port regex.c line_replace and replace_matched_text")
}
