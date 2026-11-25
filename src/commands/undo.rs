//! Undo the last change (the u command).
//!
//! Ports the 'u' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c undo.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'u': `args::unexpected_address` and `args::get_command_suffix`, then
/// `ed.restore()`; no snapshot is `Message("Nothing to undo")`
/// (use `buf_err`). A second u redoes, as restore swaps. When `isglobal`,
/// clear `ed.active` (C clear_active_list), which ends the global
/// command.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'u' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c undo")
}
