//! Print a line number (the = command).
//!
//! Ports the '=' arm of GNU ed 1.22.4 main_loop.c exec_command:
//! `printf( "%d\\n", addr_cnt ? second_addr : last_addr() );`
//!
//! Suffix parsing stays in the dispatcher. This function only chooses
//! which address to print and writes it as a decimal number plus newline.

#![allow(dead_code)]

use super::CommandError;
use std::io::Write;

/// Print `second_addr` when `addr_cnt` is not 0, otherwise `last_addr`.
///
/// GNU ed prints the number with "%d\\n". Addresses are 0-indexed only
/// for the empty-buffer last line (0); otherwise they are 1-indexed.
pub fn print_line_number(
    addr_cnt: usize,
    second_addr: usize,
    last_addr: usize,
    out: &mut dyn Write,
) -> Result<(), CommandError> {
    let _ = (addr_cnt, second_addr, last_addr, out);
    todo!("port GNU ed 1.22.4 main_loop.c exec_command '='")
}
