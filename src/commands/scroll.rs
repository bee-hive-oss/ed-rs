//! Scroll (the z command).
//!
//! Ports the 'z' arm of GNU ed 1.22.6 main_loop.c exec_command.

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
    let dot = ed.buffer.current();
    let addr = if isglobal { dot } else { dot + 1 };
    super::range::set_second_addr(ed, addr)?;
    if matches!(ed.peek(), Some('1'..='9')) {
        let n = super::args::parse_int(ed)?;
        if n > 0 {
            ed.user_lines = usize::try_from(n).unwrap_or(0);
        }
    }
    super::args::get_command_suffix(ed)?;
    let end = ed.buffer.len().min(ed.second + ed.window_lines() - 1);
    let flags = ed.pflags;
    super::print_any::print_lines(ed, ed.second, end, flags)?;
    ed.pflags = super::PFlags::default();
    Ok(super::Status::Ok)
}
