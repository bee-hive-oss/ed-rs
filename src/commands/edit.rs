//! Edit a file without the modified check (the E command).
//!
//! Ports the 'E' arm of GNU ed 1.22.4 main_loop.c exec_command (the e arm falls through to it).

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'E': `args::unexpected_address` and `args::unexpected_command_suffix`;
/// `io::getname::get_filename(ed, false)`. Empty the buffer
/// (`ed.buffer.load` with no lines), mark it unmodified, reset `ed.bin`
/// (C open_sbuf), and set the
/// default filename to the operand when it is not empty and does not start
/// with `!` (this also clears `ed.read_only`). Read the operand, or the
/// default filename when the operand is empty, with `io::read::read_file`
/// at address 0 with `&mut ed.bin`, honoring `ed.opts.strip_trailing_cr` and printing the byte
/// count unless `ed.opts.silent`. Store its read-only result in
/// `ed.read_only`. Then drop the undo snapshot (C reset_undo_state) so the
/// read cannot be undone. Marks are already cleared by `load`.
pub fn edit_file(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'E' arm of GNU ed 1.22.4 main_loop.c exec_command (the e arm falls through to it)")
}
