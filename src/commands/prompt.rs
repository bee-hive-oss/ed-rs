//! Toggle the prompt (the P command).
//!
//! Ports the 'P' arm of GNU ed 1.22.6 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'P': `args::unexpected_address` and `args::get_command_suffix`, then
/// flip `ed.prompt_on`.
pub fn arm(ed: &mut Editor, _isglobal: bool) -> ArmResult {
    super::args::unexpected_address(ed.addr_cnt)?;
    super::args::get_command_suffix(ed)?;
    ed.prompt_on = !ed.prompt_on;
    Ok(super::Status::Ok)
}
