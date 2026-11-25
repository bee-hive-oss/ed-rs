//! Apply a substitution to a range (regex.c search_and_replace).

#![allow(dead_code)]

use super::editor::Editor;
use super::CommandError;

/// Ports regex.c `search_and_replace`.
///
/// For each line `from..=to`, call `regex::subst::line_replace` with
/// `ed.regex.subst`, `ed.regex.replacement`, and `snum`. When it returns
/// new text, replace that line with the lines of the new text (split on
/// newline) using `ed.buffer.replace`, keep going after the inserted
/// lines, and make the last new line current. When `isglobal`, call
/// `ed.active.on_delete` and `ed.active.on_insert` for each replaced line.
/// If no line changed: `Message("No match")` unless `isglobal`, where it
/// is not an error. Mark the buffer modified when something changed.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn search_and_replace(
    ed: &mut Editor,
    from: usize,
    to: usize,
    snum: i32,
    isglobal: bool,
) -> Result<(), CommandError> {
    let _ = (ed, from, to, snum, isglobal);
    todo!("port regex.c search_and_replace")
}
