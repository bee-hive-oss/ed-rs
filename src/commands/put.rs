//! Put the cut buffer (the x command).
//!
//! Ports the 'x' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c put_lines.

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
    let _ = (ed, isglobal);
    todo!("port the 'x' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c put_lines")
}
