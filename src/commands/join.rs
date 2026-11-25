//! Join lines (the j command).
//!
//! Ports the 'j' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c join_lines.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'j': `range::set_addr_range(ed, ., .+1)` and
/// `args::get_command_suffix`; unless `isglobal`, take an undo snapshot
/// (`ed.snapshot()`).
/// Only when first < second: copy the range to the cut buffer
/// (`yank::yank_lines`, since join_lines calls delete_lines), then
/// `ed.buffer.join`. When `isglobal`, call `ed.active.on_delete` for the
/// range and `ed.active.on_insert` for the one new line.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'j' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c join_lines")
}
