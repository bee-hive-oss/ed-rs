//! Show the last error (the h command).
//!
//! Ports the 'h' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'h': `args::unexpected_address` and `args::get_command_suffix`; if
/// `ed.errmsg` is not empty print it and a newline to stdout.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'h' arm of GNU ed 1.22.4 main_loop.c exec_command")
}
