//! Edit a file (the e command).
//!
//! Ports the 'e' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'e': if the buffer is modified and `ed.warned` is false return
/// `Ok(Status::Emod)`. Otherwise do exactly what E does by calling
/// `edit::edit_file`.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'e' arm of GNU ed 1.22.4 main_loop.c exec_command")
}
