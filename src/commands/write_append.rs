//! Append the buffer to a file (the W command).
//!
//! Ports the 'W' arm of GNU ed 1.22.4 main_loop.c exec_command (write_file with mode "a").

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'W': call `write::write_lines` with `append` true.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("W arm of main_loop.c exec_command")
}
