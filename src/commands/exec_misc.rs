//! Dispatcher arms for q, Q, =, f, and # (main_loop.c exec_command).

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'q' and 'Q': `args::unexpected_address`, `args::get_command_suffix`,
/// then `quit::quit(force, ed.buffer.is_modified(), ed.warned)` with
/// `force` true for Q.
pub fn arm_quit(ed: &mut Editor, force: bool) -> ArmResult {
    let _ = (ed, force);
    todo!("q and Q arms of main_loop.c exec_command")
}

/// '=': `args::get_command_suffix`, then `equal::print_line_number`.
pub fn arm_equal(ed: &mut Editor) -> ArmResult {
    let _ = ed;
    todo!("= arm of main_loop.c exec_command")
}

/// 'f': `args::unexpected_address`, `args::unexpected_command_suffix`,
/// `io::getname::get_filename(ed, ed.opts.traditional)`, then
/// `filename::command_f` on `ed.def_filename`. A new name also clears
/// `ed.read_only` (C set_def_filename).
pub fn arm_f(ed: &mut Editor) -> ArmResult {
    let _ = ed;
    todo!("f arm of main_loop.c exec_command")
}

/// '#': skip to just past the newline (a comment).
pub fn arm_comment(ed: &mut Editor) -> ArmResult {
    let _ = ed;
    todo!("# arm of main_loop.c exec_command")
}
