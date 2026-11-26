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
    super::args::unexpected_address(ed.addr_cnt)?;
    super::args::get_command_suffix(ed)?;
    ed.restore().map_err(super::buf_err)?;
    if isglobal {
        ed.active.clear();
    }
    Ok(super::Status::Ok)
}
