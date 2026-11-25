//! Dispatcher arms for a, i, and d (the 'a', 'i', 'd' cases of
//! main_loop.c exec_command).
//!
//! The line work is in append.rs, insert.rs, and delete.rs. These arms add
//! the suffix check, the undo snapshot, the cut buffer, and global upkeep.
//! Each arm also clears `ed.bin.unterminated` when it changes the last line
//! or adds lines after it (in C that line node is then no longer the
//! unterminated last line).

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'a': `args::get_command_suffix`; unless `isglobal`, take an undo
/// snapshot with `ed.snapshot()`; then `append::append_lines` after `ed.second`, reading lines
/// from `editor::InputLines`. If no line was added, drop the snapshot
/// (GNU pushes no undo atom). When `isglobal`, call
/// `ed.active.on_insert` for the added lines.
pub fn arm_a(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("a arm of main_loop.c exec_command")
}

/// 'i': as [`arm_a`] with `insert::insert_lines` at `ed.second`.
pub fn arm_i(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("i arm of main_loop.c exec_command")
}

/// 'd': `range::set_addr_range2`, `args::get_command_suffix`, unless
/// `isglobal` an undo snapshot (`ed.snapshot()`), then `yank::yank_lines` of the range (as
/// buffer.c delete_lines does), `delete::delete_lines`, and when
/// `isglobal` `ed.active.on_delete`.
pub fn arm_d(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("d arm of main_loop.c exec_command")
}
