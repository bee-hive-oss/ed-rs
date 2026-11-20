//! Numbered print of a resolved line range (the n command).
//!
//! Ports the pf_n path of GNU ed 1.22.4 io.c print_lines and print_line,
//! which exec_command in main_loop.c calls for the n case (the pflabel
//! arm with n = pf_n). Plain print and list mode live in their own files.
//!
//! This function only formats already-resolved line text. Address parsing,
//! default ranges, and command-suffix collection stay in the dispatcher.

#![allow(dead_code)]

use super::CommandError;
use std::io::Write;

/// Print the inclusive 1-indexed range `from..=to`, prefixing each line
/// with its address and a tab.
///
/// `lines[0]` is line 1. Address 0 is invalid, matching print_lines in
/// io.c (`if( !from ) { invalid_address(); return false; }`). A reversed
/// or past-the-end range is also an invalid address.
///
/// Matches print_line when pf_n is set: `printf( "%d\\t", current_addr() )`
/// then the line text and a newline. GNU ed sets current_addr to the line
/// being printed, so the number is that 1-indexed address. On success
/// the current line becomes `to` and that value is returned. Use write!
/// and writeln! so a write error becomes CommandError::Io.
pub fn number_lines(
    lines: &[String],
    from: usize,
    to: usize,
    out: &mut dyn Write,
) -> Result<usize, CommandError> {
    let _ = (lines, from, to, out);
    todo!("port GNU ed 1.22.4 io.c print_lines / print_line for n")
}
