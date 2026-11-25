//! Write the buffer (the w command, with wq).
//!
//! Ports the 'w' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'w' (and 'W' through `append` true, from write_append.rs): a `q` or
/// `Q` right after the letter is consumed and remembered. Then
/// `args::unexpected_command_suffix` and
/// `io::getname::get_filename(ed, false)`. With no address and an empty
/// buffer write nothing (0,0); otherwise `range::set_addr_range(ed, 1,
/// $)`. If there is no default filename and the operand does not start
/// with `!`, make it the default. `io::write::write_file` with `unterminated_last` set to
/// `ed.bin.isbinary && ed.bin.unterminated` (operand, or the
/// default filename when empty), printing the byte count unless
/// `ed.opts.silent`. If all lines went to a file (not `!`), mark the buffer
/// unmodified; else with `q` and a modified, unwarned buffer return
/// `Status::Emod`. With `q` or `Q` return `Status::Quit`.
pub fn write_lines(ed: &mut Editor, isglobal: bool, append: bool) -> ArmResult {
    let _ = (ed, isglobal, append);
    todo!("port the 'w' arm of GNU ed 1.22.4 main_loop.c exec_command")
}

/// 'w' arm: [`write_lines`] with `append` false.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("w arm of main_loop.c exec_command")
}
