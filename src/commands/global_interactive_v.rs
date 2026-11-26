//! Interactive global command on lines that do not match (the V command).
//!
//! Ports the 'V' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'V': call `global_interactive::run_global_interactive` with
/// `matching` false.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::global_interactive::run_global_interactive(ed, isglobal, false)
}
