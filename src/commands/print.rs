//! Print a range of lines (the p, n, and l commands).
//!
//! Ports GNU ed 1.22.4 io.c print_lines and the helper print_line, which
//! exec_command in main_loop.c calls for the p, n, and l cases (the
//! pflabel arm). List mode also uses escchar from io.c.
//!
//! This function only formats already-resolved line text. Address parsing,
//! default ranges, and command-suffix collection stay in the dispatcher.

#![allow(dead_code)]

use super::CommandError;
use std::io::Write;

/// Default window width used by GNU ed 1.22.4 list mode when no SIGWINCH
/// size is available (signal.c window_columns_ starts at 76).
pub const DEFAULT_WINDOW_COLUMNS: usize = 76;

/// GNU ed 1.22.4 Pflags from ed.h (pf_l, pf_n, pf_p).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrintFlags {
    /// pf_l: list mode (escape unprintable bytes, wrap, trailing $).
    pub list: bool,
    /// pf_n: prefix each line with its address and a tab.
    pub number: bool,
    /// pf_p: ordinary print. Combined with list or number when both are set.
    pub print: bool,
}

impl PrintFlags {
    pub const L: u8 = 0x01;
    pub const N: u8 = 0x02;
    pub const P: u8 = 0x04;

    pub fn from_bits(bits: u8) -> Self {
        Self {
            list: bits & Self::L != 0,
            number: bits & Self::N != 0,
            print: bits & Self::P != 0,
        }
    }

    pub fn bits(self) -> u8 {
        let mut bits = 0;
        if self.list {
            bits |= Self::L;
        }
        if self.number {
            bits |= Self::N;
        }
        if self.print {
            bits |= Self::P;
        }
        bits
    }
}

/// Print the inclusive 1-indexed range `from..=to`.
///
/// `lines[0]` is line 1. Address 0 is invalid, matching print_lines in
/// io.c (`if( !from ) { invalid_address(); return false; }`). A reversed
/// or past-the-end range is also an invalid address.
///
/// On success the current line becomes `to`, as GNU ed does by calling
/// set_current_addr for each printed line, and that new current line is
/// returned. `traditional` is Options.traditional: GNU ed omits the
/// trailing `$` in list mode when traditional is set.
///
/// `window_columns` is the wrap width for list mode (GNU default 76).
pub fn print_lines(
    lines: &[String],
    from: usize,
    to: usize,
    pflags: PrintFlags,
    traditional: bool,
    window_columns: usize,
    out: &mut dyn Write,
) -> Result<usize, CommandError> {
    let _ = (
        lines,
        from,
        to,
        pflags,
        traditional,
        window_columns,
        out,
    );
    todo!("port GNU ed 1.22.4 io.c print_lines and print_line")
}
