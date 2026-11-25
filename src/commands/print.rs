//! Plain print of a resolved line range (the p command).
//!
//! Ports the ordinary-print path of GNU ed 1.22.4 io.c print_lines and
//! print_line, which exec_command in main_loop.c calls for the p case
//! (the pflabel arm with n = pf_p). Numbered print and list mode live
//! in their own files.
//!
//! This function only formats already-resolved line text. Address parsing,
//! default ranges, and command-suffix collection stay in the dispatcher.

#![allow(dead_code)]

use super::CommandError;
use std::io::Write;

/// Print the inclusive 1-indexed range `from..=to` with no number prefix
/// and no list-mode escapes.
///
/// `lines[0]` is line 1. Address 0 is invalid, matching print_lines in
/// io.c (`if( !from ) { invalid_address(); return false; }`). A reversed
/// or past-the-end range is also an invalid address.
///
/// Each line is written as stored (no trailing newline in the buffer),
/// then a newline, matching print_line when pf_l and pf_n are clear.
/// On success the current line becomes `to`, as GNU ed does by calling
/// set_current_addr for each printed line, and that new current line is
/// returned. Use writeln! so a write error becomes CommandError::Io.
pub fn print_lines(
    lines: &[String],
    from: usize,
    to: usize,
    out: &mut dyn Write,
) -> Result<usize, CommandError> {
    let _ = (lines, from, to, out);
    todo!("port GNU ed 1.22.4 io.c print_lines / print_line for plain p")
}
