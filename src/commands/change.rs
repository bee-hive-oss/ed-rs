//! Change lines (the c command).
//!
//! Ports the 'c' arm of GNU ed 1.22.4 main_loop.c exec_command, with buffer.c delete_lines and append_lines.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'c': `range::set_addr_range2` and `args::get_command_suffix`; unless
/// `isglobal`, take an undo snapshot
/// (`ed.snapshot()`). Copy the range to the cut buffer
/// (`yank::yank_lines`), delete it (`ed.buffer.delete`), then append input
/// lines (`editor::InputLines`) after the new current line, as an insert
/// when the current line is at or past the old first line (C
/// `current_addr() >= first_addr`). When `isglobal`, call
/// `ed.active.on_delete` and `ed.active.on_insert`.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'c' arm of GNU ed 1.22.4 main_loop.c exec_command, with buffer.c delete_lines and append_lines")
}
