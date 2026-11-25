//! Read the replacement part of an s command (regex.c extract_replacement).

#![allow(dead_code)]

use super::state::RegexState;
use crate::commands::CommandError;

/// Ports regex.c `extract_replacement`.
///
/// `cmd[*pos..]` starts at the delimiter that follows the pattern, for
/// example the second `/` in `s/a/b/`. Read the replacement up to the next
/// unescaped delimiter, or up to the newline when the last delimiter is
/// omitted (in a global command the newline only ends it when it is also the
/// end of the command list, because newlines inside it are unescaped). On
/// return `*pos` is at that closing delimiter or newline.
///
/// A replacement that is just `%` (followed by the delimiter or the
/// newline) means "repeat the previous replacement": keep the stored one, or
/// `Message("No previous substitution")` if there is none. An empty
/// replacement is stored as an empty string. A backslash keeps the next
/// character as it is in the stored text (`subst::line_replace` interprets
/// `\&`, `\1`..`\9`, and `&` later). When the backslash is followed by a
/// newline and `isglobal` is false, call `next_line` for more text,
/// continue with it, and make `cmd` that new line with `*pos` moved into it.
/// A missing delimiter right at the start (a newline) is
/// `Message("Missing pattern delimiter")`.
///
/// The replacement is stored in `state.replacement`.
pub fn extract_replacement(
    state: &mut RegexState,
    cmd: &mut String,
    pos: &mut usize,
    isglobal: bool,
    next_line: &mut dyn FnMut() -> Option<String>,
) -> Result<(), CommandError> {
    let _ = (state, cmd, pos, isglobal, next_line);
    todo!("port regex.c extract_replacement")
}
