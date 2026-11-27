//! Global command on lines that do not match (the v command).
//!
//! Ports the 'v' arm of GNU ed 1.22.6 main_loop.c exec_command (build_active_list with match false).

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'v': call `global::run_global` with `matching` false.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::global::run_global(ed, isglobal, false)
}
