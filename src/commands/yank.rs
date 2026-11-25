//! Copy lines to the cut buffer (the y command).
//!
//! Ports the 'y' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c yank_lines.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'y': `range::set_addr_range2`, `args::get_command_suffix`, then
/// [`yank_lines`]. y does not change the buffer, the current line, or the
/// undo snapshot.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'y' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c yank_lines")
}

/// Ports buffer.c `yank_lines`: replace `ed.cut` with a copy of lines
/// `from..=to`. Also used by d, c, j and ! before they delete.
pub fn yank_lines(ed: &mut Editor, from: usize, to: usize) {
    let _ = (ed, from, to);
    todo!("port buffer.c yank_lines")
}
