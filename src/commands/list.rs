//! List-mode print of a resolved line range (the l command).
//!
//! Ports the pf_l path of GNU ed 1.22.4 io.c print_lines and print_line,
//! including escchar, which exec_command in main_loop.c calls for the l
//! case (the pflabel arm with n = pf_l). Plain print and numbered print
//! live in their own files.
//!
//! This function only formats already-resolved line text. Address parsing,
//! default ranges, and command-suffix collection stay in the dispatcher.

#![allow(dead_code)]

use super::CommandError;
use std::io::Write;

/// Default window width used by GNU ed 1.22.4 list mode when no SIGWINCH
/// size is available (signal.c window_columns_ starts at 76).
pub const DEFAULT_WINDOW_COLUMNS: usize = 76;

/// Print the inclusive 1-indexed range `from..=to` in list mode.
///
/// `lines[0]` is line 1. Address 0 is invalid, matching print_lines in
/// io.c (`if( !from ) { invalid_address(); return false; }`). A reversed
/// or past-the-end range is also an invalid address.
///
/// Matches print_line when pf_l is set: wrap with a backslash and newline
/// when the column runs past `window_columns` (GNU default 76), escape
/// `$` and `\\`, use escchar letter escapes for the C escapes, otherwise
/// a backslash plus three octal digits, then a trailing `$` unless
/// `traditional` is set, then a newline. On success the current line
/// becomes `to` and that value is returned. Use write! and writeln! so a
/// write error becomes CommandError::Io.
pub fn list_lines(
    lines: &[String],
    from: usize,
    to: usize,
    traditional: bool,
    window_columns: usize,
    out: &mut dyn Write,
) -> Result<usize, CommandError> {
    let _ = (lines, from, to, traditional, window_columns, out);
    todo!("port GNU ed 1.22.4 io.c print_lines / print_line / escchar for l")
}
