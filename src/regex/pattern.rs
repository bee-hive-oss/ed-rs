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
    let _ = (input, delimiter);
    todo!("port regex.c extract_pattern")
}
