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
    // GNU ed rejects address 0 inside print_lines. A reversed range or a
    // range past the last line is an invalid address too; reject it before
    // writing so a bad range never produces a partial listing.
    if from == 0 || to == 0 || from > to || to > lines.len() {
        return Err(CommandError::InvalidAddress);
    }
    // print_line with pf_l. GNU ed sets the current line to each address
    // as it prints, so the line left current is `to`.
    for line in &lines[from - 1..to] {
        list_line(line.as_bytes(), traditional, window_columns, out)?;
    }
    Ok(to)
}

/// Letter escape from GNU ed escchar, or `None` when the byte is written
/// as a backslash and three octal digits. NUL is octal: escchar requires
/// a non-zero byte before it accepts a match.
fn escchar(ch: u8) -> Option<char> {
    match ch {
        0x07 => Some('a'),
        0x08 => Some('b'),
        0x0c => Some('f'),
        b'\n' => Some('n'),
        b'\r' => Some('r'),
        b'\t' => Some('t'),
        0x0b => Some('v'),
        _ => None,
    }
}

/// One line of list mode. Column counting matches GNU ed print_line: the
/// width check happens once per byte, before that byte's escape prefix is
/// added, and a wrap is a backslash plus a newline with the column reset
/// to 1. The trailing `$` is not part of the wrap.
fn list_line(
    bytes: &[u8],
    traditional: bool,
    window_columns: usize,
    out: &mut dyn Write,
) -> Result<(), CommandError> {
    let mut col = 0usize;
    for &ch in bytes {
        col += 1;
        if col > window_columns {
            col = 1;
            // writeln!, not write! with a newline in the format string.
            writeln!(out, "\\")?;
        }
        if (32..=126).contains(&ch) {
            if ch == b'$' || ch == b'\\' {
                col += 1;
                write!(out, "\\")?;
            }
            write!(out, "{}", ch as char)?;
        } else {
            col += 1;
            write!(out, "\\")?;
            if let Some(letter) = escchar(ch) {
                write!(out, "{letter}")?;
            } else {
                col += 2;
                let d1 = (ch >> 6) & 7;
                let d2 = (ch >> 3) & 7;
                let d3 = ch & 7;
                write!(out, "{d1}{d2}{d3}")?;
            }
        }
    }
    if traditional {
        writeln!(out)?;
    } else {
        writeln!(out, "$")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    fn lines_of(items: &[&str]) -> Vec<String> {
        items.iter().copied().map(str::to_string).collect()
    }

    #[test]
    fn lists_plain_text_with_an_end_marker_and_returns_to() {
        let lines = lines_of(&["hello", "world"]);
        let mut out = Vec::new();

        let current = list_lines(&lines, 1, 2, false, DEFAULT_WINDOW_COLUMNS, &mut out).unwrap();

        assert_eq!(current, 2);
        assert_eq!(String::from_utf8(out).unwrap(), "hello$\nworld$\n");
    }

    #[test]
    fn traditional_omits_the_end_marker() {
        let lines = lines_of(&["hello"]);
        let mut out = Vec::new();

        list_lines(&lines, 1, 1, true, DEFAULT_WINDOW_COLUMNS, &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "hello\n");
    }

    #[test]
    fn empty_line_is_only_the_end_marker() {
        let lines = lines_of(&[""]);
        let mut plain = Vec::new();
        let mut traditional = Vec::new();

        list_lines(&lines, 1, 1, false, DEFAULT_WINDOW_COLUMNS, &mut plain).unwrap();
        list_lines(&lines, 1, 1, true, DEFAULT_WINDOW_COLUMNS, &mut traditional).unwrap();

        assert_eq!(plain, b"$\n");
        assert_eq!(traditional, b"\n");
    }

    #[test]
    fn escapes_dollar_backslash_and_the_c_escapes() {
        let lines = lines_of(&["a$b\\c", " \t\x07\x08\x0c\n\r\x0b"]);
        let mut out = Vec::new();

        list_lines(&lines, 1, 2, false, DEFAULT_WINDOW_COLUMNS, &mut out).unwrap();

        assert_eq!(
            String::from_utf8(out).unwrap(),
            "a\\$b\\\\c$\n \\t\\a\\b\\f\\n\\r\\v$\n"
        );
    }

    #[test]
    fn non_printable_bytes_are_three_octal_digits() {
        // U+00FF is the UTF-8 bytes C3 BF, listed one byte at a time.
        let lines = vec!["\u{0}\u{1}\u{1f}\u{7f}\u{00ff}".to_string()];
        let mut out = Vec::new();

        list_lines(&lines, 1, 1, false, DEFAULT_WINDOW_COLUMNS, &mut out).unwrap();

        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\\000\\001\\037\\177\\303\\277$\n"
        );
    }

    #[test]
    fn wraps_with_a_backslash_when_the_column_passes_the_window() {
        let lines = lines_of(&["abcdefghijk"]);
        let mut out = Vec::new();

        // 10 columns: the 11th byte wraps, then the end marker follows it.
        list_lines(&lines, 1, 1, false, 10, &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "abcdefghij\\\nk$\n");
    }

    #[test]
    fn a_dollar_on_the_last_column_still_takes_its_escape_prefix() {
        let lines = lines_of(&["abcd$"]);
        let mut out = Vec::new();

        // The width check runs before the extra column for the backslash,
        // so `\$` is emitted on this row and the line runs past the window.
        list_lines(&lines, 1, 1, false, 5, &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "abcd\\$$\n");
    }

    #[test]
    fn an_escape_that_would_start_past_the_window_wraps_first() {
        let lines = lines_of(&["ab\u{1}"]);
        let mut out = Vec::new();

        list_lines(&lines, 1, 1, false, 2, &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "ab\\\n\\001$\n");
    }

    #[test]
    fn bad_ranges_are_invalid_address_and_write_nothing() {
        let lines = lines_of(&["a", "b", "c", "d"]);
        for (from, to) in [
            (0, 0),
            (0, 1),
            (0, 4),
            (1, 0),
            (2, 1),
            (1, 5),
            (5, 5),
        ] {
            let mut out = Vec::new();
            let err = list_lines(&lines, from, to, false, DEFAULT_WINDOW_COLUMNS, &mut out)
                .unwrap_err();
            assert_eq!(err, CommandError::InvalidAddress);
            assert_eq!(err.message(), "Invalid address");
            assert!(out.is_empty(), "from={from} to={to} wrote {out:?}");
        }

        let mut out = Vec::new();
        assert_eq!(
            list_lines(&[], 1, 1, false, DEFAULT_WINDOW_COLUMNS, &mut out),
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

        let lines = lines_of(&["hello"]);
        let err = list_lines(
            &lines,
            1,
            1,
            false,
            DEFAULT_WINDOW_COLUMNS,
            &mut BrokenPipe,
        )
        .unwrap_err();
        assert_eq!(err, CommandError::Io(io::ErrorKind::BrokenPipe));
        assert_eq!(err.message(), "Cannot write output");
    }
}
