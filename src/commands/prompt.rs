//! Toggle the prompt (the P command).
//!
//! Ports the 'P' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'P': `args::unexpected_address` and `args::get_command_suffix`, then
/// flip `ed.prompt_on`.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'P' arm of GNU ed 1.22.4 main_loop.c exec_command")
}
