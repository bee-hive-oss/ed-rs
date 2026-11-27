//! Numbered print of a resolved line range (the n command).
//!
//! Ports the pf_n path of GNU ed 1.22.6 io.c print_lines and print_line,
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
    // GNU ed rejects address 0 inside print_lines. A reversed range or a
    // range past the last line is an invalid address too; reject it before
    // writing so a bad range never produces a partial listing.
    if from == 0 || to == 0 || from > to || to > lines.len() {
        return Err(CommandError::InvalidAddress);
    }
    // print_line with pf_n: "%d\t" then the stored text and a newline.
    // GNU ed sets the current line to each address as it prints, so the
    // line left current is `to`.
    for addr in from..=to {
        let line = &lines[addr - 1];
        write!(out, "{addr}\t")?;
        writeln!(out, "{line}")?;
    }
    Ok(to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    fn sample() -> Vec<String> {
        ["alpha", "", "a$b\\c", "has\ttab"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn numbers_an_inclusive_range_and_returns_to() {
        let lines = sample();
        let mut out = Vec::new();

        let current = number_lines(&lines, 2, 4, &mut out).unwrap();

        assert_eq!(current, 4);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "2\t\n3\ta$b\\c\n4\thas\ttab\n"
        );
    }

    #[test]
    fn numbers_every_line_for_a_full_range() {
        let lines = sample();
        let mut out = Vec::new();

        let current = number_lines(&lines, 1, lines.len(), &mut out).unwrap();

        assert_eq!(current, 4);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "1\talpha\n2\t\n3\ta$b\\c\n4\thas\ttab\n"
        );
    }

    #[test]
    fn prefixes_one_line_with_its_address_and_a_tab() {
        let lines = sample();
        let mut out = Vec::new();

        let current = number_lines(&lines, 3, 3, &mut out).unwrap();

        assert_eq!(current, 3);
        // n is not list mode: '$' and '\\' stay literal, and the number
        // is the 1-indexed address, not a list-mode escape.
        assert_eq!(String::from_utf8(out).unwrap(), "3\ta$b\\c\n");
    }

    #[test]
    fn empty_line_is_the_address_a_tab_and_a_newline() {
        let lines = vec![String::new()];
        let mut out = Vec::new();

        let current = number_lines(&lines, 1, 1, &mut out).unwrap();

        assert_eq!(current, 1);
        assert_eq!(out, b"1\t\n");
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
            let err = number_lines(&lines, from, to, &mut out).unwrap_err();
            assert_eq!(err, CommandError::InvalidAddress);
            assert_eq!(err.message(), "Invalid address");
            assert!(out.is_empty(), "from={from} to={to} wrote {out:?}");
        }

        let mut out = Vec::new();
        assert_eq!(
            number_lines(&[], 0, 0, &mut out),
            Err(CommandError::InvalidAddress)
        );
        assert_eq!(
            number_lines(&[], 1, 1, &mut out),
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
        let err = number_lines(&lines, 1, 2, &mut BrokenPipe).unwrap_err();
        assert_eq!(err, CommandError::Io(io::ErrorKind::BrokenPipe));
        assert_eq!(err.message(), "Cannot write output");
    }
}
