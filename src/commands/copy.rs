//! Copy lines (the t command).
//!
//! Ports the 't' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c copy_lines.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 't': `range::set_addr_range2`, `range::get_third_addr`,
/// `args::get_command_suffix`; unless `isglobal`, take an undo snapshot
/// (`ed.snapshot()`).
/// Then `ed.buffer.copy`. When `isglobal`, call `ed.active.on_insert`
/// for the copied lines.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 't' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c copy_lines")
}
