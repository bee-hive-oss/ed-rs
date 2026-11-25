//! Read a file into the buffer (the r command).
//!
//! Ports the 'r' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'r': `args::unexpected_command_suffix`; with no address use `$`.
/// `io::getname::get_filename(ed, false)`. If there is no default filename
/// and the operand does not start with `!`, make it the default. Unless
/// `isglobal`, take an undo snapshot
/// (`ed.snapshot()`). `io::read::read_file` (with `&mut ed.bin`) after the
/// address (operand, or the default filename when empty), honoring
/// `ed.opts.strip_trailing_cr` and printing the byte count unless
/// `ed.opts.silent`. If any line was read, mark the buffer modified. When
/// `isglobal`, call `ed.active.on_insert`.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'r' arm of GNU ed 1.22.4 main_loop.c exec_command")
}
