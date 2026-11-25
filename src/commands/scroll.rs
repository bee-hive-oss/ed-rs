//! Scroll (the z command).
//!
//! Ports the 'z' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'z': `range::set_second_addr(ed, . + 1)` (just `.` when `isglobal`).
/// A digit 1 to 9 next starts a count read with `args::parse_int`, which
/// sets `ed.user_lines`. Then `args::get_command_suffix` and
/// `print_any::print_lines` from `ed.second` to
/// `min($, second + ed.window_lines() - 1)` with the suffix flags; then
/// clear `ed.pflags`.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port the 'z' arm of GNU ed 1.22.4 main_loop.c exec_command")
}
