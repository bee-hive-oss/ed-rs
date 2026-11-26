//! Shell escape and filter (the ! command).
//!
//! Ports the '!' arm of GNU ed 1.22.4 main_loop.c exec_command and main_loop.c command_shell.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// '!': `io::shellcmd::get_shell_command`. With no address run it with
/// `io::shell::shell_system` (`Message("Can't create shell process")` on
/// failure) and print "!" and a newline unless `ed.opts.silent`. With an
/// address, filter the range: `range::set_addr_range2`; a `<` or `>` in
/// the command is `Message("Redirection not allowed")`; send the range
/// (encoded with `io::bytes::encode`) through `io::shell::shell_filter`
/// and decode the output with `io::bytes::decode`; unless `isglobal` take an undo
/// snapshot with `ed.snapshot()`; delete the range (with the cut buffer as delete does) and add
/// the output lines in its place, then if the current line is 0 and the
/// buffer is not empty make it 1. Print the byte count unless silent.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the '!' arm of GNU ed 1.22.4 main_loop.c exec_command and main_loop.c command_shell")
}
