//! Put the cut buffer (the x command).
//!
//! Ports the 'x' arm of GNU ed 1.22.6 main_loop.c exec_command and buffer.c put_lines.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'x': `ed.second > $` is `InvalidAddress`. `args::get_command_suffix`;
/// unless `isglobal`, take an undo snapshot
/// (`ed.snapshot()`). An empty `ed.cut` is
/// `Message("Nothing to put")`. Otherwise add the cut lines after
/// `ed.second` with `ed.buffer.append`; the current line is the last one
/// added. When `isglobal`, call `ed.active.on_insert`.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    if ed.second > ed.buffer.len() {
        return Err(super::CommandError::InvalidAddress);
    }
    super::args::get_command_suffix(ed)?;
    if !isglobal {
        ed.snapshot();
    }
    if ed.cut.is_empty() {
        return Err(super::CommandError::Message("Nothing to put"));
    }
    let n = ed.cut.len();
    let addr = ed.second;
    let old_len = ed.buffer.len();
    let lines = ed.cut.clone();
    ed.buffer.append(addr, lines).map_err(super::buf_err)?;
    if isglobal {
        ed.active.on_insert(addr, n);
    }
    if addr == old_len {
        ed.bin.unterminated = false;
    }
    Ok(super::Status::Ok)
}
