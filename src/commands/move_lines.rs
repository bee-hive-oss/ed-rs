//! Move lines (the m command).
//!
//! Ports the 'm' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c move_lines.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'm': `range::set_addr_range2`, then `range::get_third_addr`. A
/// destination with first <= dest < second is `Message("Invalid
/// destination")`. Then `args::get_command_suffix`; unless `isglobal`,
/// take an undo snapshot
/// (`ed.snapshot()`). Moving to first-1 or second changes nothing but
/// the current line (second) and still marks the buffer modified;
/// otherwise `ed.buffer.move_lines`. When `isglobal`, call
/// `ed.active.on_move`.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'm' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c move_lines")
}
