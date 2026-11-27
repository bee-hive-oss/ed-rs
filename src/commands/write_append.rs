//! Append the buffer to a file (the W command).
//!
//! Ports the 'W' arm of GNU ed 1.22.6 main_loop.c exec_command (write_file with mode "a").

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'W': call `write::write_lines` with `append` true.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::write::write_lines(ed, isglobal, true)
}
