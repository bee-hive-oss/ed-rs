//! Read a shell command operand (main_loop.c get_shell_command).

#![allow(dead_code)]

use crate::commands::editor::Editor;
use crate::commands::CommandError;

/// Ports main_loop.c `get_shell_command`.
///
/// `ed.pos` is just after the `!`. In restricted mode fail with
/// `Message("Shell access restricted")`. Join continuation lines with
/// `io::extended::get_extended_line(.., true, ..)` (use `ed.next_line`).
/// Build the command text with a leading `!` marker. A `!` as the first
/// character is replaced by the previous command in `ed.shell_cmd`
/// (`Message("No previous command")` when it is empty, or when
/// `ed.opts.traditional` is set and the previous command is just `!`). A `%`
/// is replaced by `ed.def_filename` (`NoCurrentFilename` when empty);
/// `\%` is a literal `%`; other backslash pairs are kept. Stop at the
/// newline and consume it. Store the result in `ed.shell_cmd`. If a `!` or
/// `%` was replaced, print the final command (without the marker) and a
/// newline to stdout.
///
/// Return the command including its leading `!`.
pub fn get_shell_command(ed: &mut Editor) -> Result<String, CommandError> {
    let _ = ed;
    todo!("port main_loop.c get_shell_command")
}
