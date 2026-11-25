//! Read a file name operand (main_loop.c get_filename, and main.c
//! may_access_filename).

#![allow(dead_code)]

use crate::commands::editor::Editor;
use crate::commands::CommandError;

/// Ports main.c `may_access_filename`.
///
/// Errors, in the C order: an empty name or one ending in `/` is
/// `Message("Is a directory")`. In restricted mode, a name starting with
/// `!` is `Message("Shell access restricted")` and `..` or any name with a
/// `/` is `Message("Directory access restricted")`. A newline in the name is
/// `Message("Newline character not allowed in file names")`. Unless
/// `unsafe_names`, a byte in 7..=13, 27 or 127 is
/// `Message("Control characters not allowed in file names")`.
pub fn may_access_filename(
    name: &str,
    restricted: bool,
    unsafe_names: bool,
) -> Result<(), CommandError> {
    let _ = (name, restricted, unsafe_names);
    todo!("port main.c may_access_filename")
}

/// Ports main_loop.c `get_filename`.
///
/// The name stays in ed-rs's one-char-per-byte form; callers open it with
/// `io::bytes::to_path`.
///
/// Skip blanks at `ed.pos`. If the line ends there: with
/// `traditional_f_command` false and an empty `ed.def_filename` fail with
/// `NoCurrentFilename`; otherwise consume the newline and return an empty
/// string. Otherwise join continuation lines with
/// `io::extended::get_extended_line(.., true, ..)` (use `ed.next_line`),
/// and update `ed.cmd`. If the operand starts with `!`, consume it and
/// return `shellcmd::get_shell_command(ed)`. A leading `~/` becomes the
/// HOME directory when HOME is set and not empty. A name longer than
/// 4096 bytes (path_max) is `Message("Filename too long")`. Read up to the
/// newline, consume it, then check the name with [`may_access_filename`]
/// using `ed.opts.restricted` and `ed.opts.unsafe_names`.
pub fn get_filename(
    ed: &mut Editor,
    traditional_f_command: bool,
) -> Result<String, CommandError> {
    let _ = (ed, traditional_f_command);
    todo!("port main_loop.c get_filename")
}
