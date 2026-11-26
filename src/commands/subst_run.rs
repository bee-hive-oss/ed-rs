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
    if from == 0 || from > to || to > ed.buffer.len() {
        return Err(CommandError::InvalidAddress);
    }
    let re = ed
        .regex
        .subst
        .clone()
        .ok_or(CommandError::Message("No previous substitution"))?;
    let template = ed.regex.replacement.clone().unwrap_or_default();
    let mut addr = from;
    let mut found = false;
    for _ in 0..(to - from + 1) {
        let line = ed.buffer.line(addr).map_err(super::buf_err)?.to_string();
        match crate::regex::subst::line_replace(&re, &template, &line, snum)? {
            None => addr += 1,
            Some(text) => {
                let old_len = ed.buffer.len();
                let parts: Vec<String> = text.split('\n').map(str::to_string).collect();
                let n = parts.len();
                if isglobal {
                    ed.active.on_delete(addr, addr);
                }
                ed.buffer.replace(addr, addr, parts).map_err(super::buf_err)?;
                if isglobal {
                    ed.active.on_insert(addr.saturating_sub(1), n);
                }
                if addr == old_len {
                    ed.bin.unterminated = false;
                }
                found = true;
                addr = ed.buffer.current() + 1;
            }
        }
    }
    if !found && !isglobal {
        return Err(CommandError::Message("No match"));
    }
    if found {
        ed.buffer.set_modified(true);
    }
    Ok(())
}
