//! Print the next line (the null command, a bare newline).
//!
//! Ports the '\\n' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// Newline: `range::set_second_addr(ed, . + 1)`, or `.` when `isglobal`
/// and not `ed.opts.traditional`; then print that one line with
/// `print_any::print_lines` and no flags.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let dot = ed.buffer.current();
    let addr = if isglobal && !ed.opts.traditional { dot } else { dot + 1 };
    super::range::set_second_addr(ed, addr)?;
    super::print_any::print_lines(ed, ed.second, ed.second, super::PFlags::default())?;
    Ok(super::Status::Ok)
}
