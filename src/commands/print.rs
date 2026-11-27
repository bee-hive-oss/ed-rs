//! Plain print of a resolved line range (the p command).
//!
//! Ports the ordinary-print path of GNU ed 1.22.6 io.c print_lines and
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
    // GNU ed rejects address 0 inside print_lines. A reversed range or a
    // range past the last line is an invalid address too; reject it before
    // writing so a bad range never produces a partial listing.
    if from == 0 || to == 0 || from > to || to > lines.len() {
        return Err(CommandError::InvalidAddress);
    }
    // print_line with pf_l and pf_n clear: the stored text, then '\n'.
    // GNU ed sets the current line to each address as it prints, so the
    // line left current is `to`.
    for line in &lines[from - 1..to] {
        writeln!(out, "{line}")?;
    }
    Ok(to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    fn sample() -> Vec<String> {
        ["alpha", "", r"a$b\c", "has\ttab"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn prints_an_inclusive_range_and_returns_to() {
        let lines = sample();
        let mut out = Vec::new();

        let current = print_lines(&lines, 2, 4, &mut out).unwrap();

        assert_eq!(current, 4);
        assert_eq!(String::from_utf8(out).unwrap(), "\na$b\\c\nhas\ttab\n");
    }

    #[test]
    fn prints_every_line_for_a_full_range() {
        let lines = sample();
        let mut out = Vec::new();

        let current = print_lines(&lines, 1, lines.len(), &mut out).unwrap();

        assert_eq!(current, 4);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "alpha\n\na$b\\c\nhas\ttab\n"
        );
    }

    #[test]
    fn prints_one_line_without_a_number_or_list_escapes() {
        let lines = sample();
        let mut out = Vec::new();

        let current = print_lines(&lines, 3, 3, &mut out).unwrap();

        assert_eq!(current, 3);
        // Plain p: no "3\t" prefix, and '$' / '\\' stay literal.
        assert_eq!(String::from_utf8(out).unwrap(), "a$b\\c\n");
    }

    #[test]
    fn empty_line_is_a_bare_newline() {
        let lines = vec![String::new()];
        let mut out = Vec::new();

        let current = print_lines(&lines, 1, 1, &mut out).unwrap();

        assert_eq!(current, 1);
        assert_eq!(out, b"\n");
    }

    #[test]
    fn bad_ranges_are_invalid_address_and_write_nothing() {
        let lines = sample();
        for (from, to) in [
            (0, 0),
            (0, 1),
            (0, 4),
            (1, 0),
            (2, 1),
            (3, 2),
            (1, 5),
            (4, 5),
            (5, 5),
            (5, 4),
        ] {
            let mut out = Vec::new();
            let err = print_lines(&lines, from, to, &mut out).unwrap_err();
            assert_eq!(err, CommandError::InvalidAddress);
            assert_eq!(err.message(), "Invalid address");
            assert!(out.is_empty(), "from={from} to={to} wrote {out:?}");
        }

        let mut out = Vec::new();
        assert_eq!(
            print_lines(&[], 0, 0, &mut out),
            Err(CommandError::InvalidAddress)
        );
        assert_eq!(
            print_lines(&[], 1, 1, &mut out),
            Err(CommandError::InvalidAddress)
        );
        assert!(out.is_empty());
    }

    #[test]
    fn a_write_error_is_command_error_io() {
        struct BrokenPipe;

        impl Write for BrokenPipe {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let lines = sample();
        let err = print_lines(&lines, 1, 2, &mut BrokenPipe).unwrap_err();
        assert_eq!(err, CommandError::Io(io::ErrorKind::BrokenPipe));
        assert_eq!(err.message(), "Cannot write output");
    }
}
