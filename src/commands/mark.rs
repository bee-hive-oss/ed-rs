//! Mark a line (the k command).
//!
//! Ports the 'k' arm of GNU ed 1.22.4 main_loop.c exec_command and main_loop.c mark_line_node.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'k': take the mark letter (the next character). `ed.second == 0` is
/// `InvalidAddress`. Then `args::get_command_suffix`, then
/// `ed.buffer.set_mark`; a bad letter is `Message("Invalid mark
/// character")` (use `buf_err`).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'k' arm of GNU ed 1.22.4 main_loop.c exec_command and main_loop.c mark_line_node")
}
