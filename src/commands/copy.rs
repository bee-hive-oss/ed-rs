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
    super::range::set_addr_range2(ed)?;
    let dest = super::range::get_third_addr(ed)?;
    super::args::get_command_suffix(ed)?;
    if !isglobal {
        ed.snapshot();
    }
    let (from, to) = (ed.first, ed.second);
    let old_len = ed.buffer.len();
    let count = to - from + 1;
    ed.buffer.copy(from, to, dest).map_err(super::buf_err)?;
    if isglobal {
        ed.active.on_insert(dest, count);
    }
    if dest == old_len {
        ed.bin.unterminated = false;
    }
    Ok(super::Status::Ok)
}
