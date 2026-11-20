//! Default filename (the f command).
//!
//! Ports the 'f' arm of GNU ed 1.22.4 main_loop.c exec_command, plus
//! set_def_filename in main_loop.c and the stdout side of print_escaped
//! in main.c (which uses escchar from io.c).
//!
//! Address and suffix checks stay in the dispatcher: f rejects an
//! address with "Unexpected address", and a letter stuck to f (for
//! example ftest.bin) is "Unexpected command suffix". This function
//! receives the operand after those checks, with leading blanks already
//! trimmed and the trailing newline removed.
//!
//! Tilde expansion, shell-command filenames, restricted mode, and the
//! unsafe-names check in get_filename are out of scope here. A later
//! I/O ticket can extend this. An operand that starts with '!' is still
//! rejected here as "Invalid redirection", as the f arm does after
//! get_filename returns.

#![allow(dead_code)]

use super::CommandError;
use std::io::Write;

/// Apply the f command to `def_filename` and print the result.
///
/// Empty `arg`: print the current default filename. If it is empty and
/// `traditional` is false, return CommandError::NoCurrentFilename
/// ("No current filename"), matching get_filename when
/// traditional_f_command is false. If traditional is true and both are
/// empty, print an empty name followed by a newline.
///
/// An `arg` that starts with '!' is CommandError::InvalidRedirection.
/// Any other non-empty `arg` becomes the default filename
/// (set_def_filename) and is then printed.
///
/// The printed name uses print_escaped (backslash, and non-printable
/// bytes as letter escapes or octal), then a newline.
pub fn command_f(
    arg: &str,
    def_filename: &mut String,
    traditional: bool,
    out: &mut dyn Write,
) -> Result<(), CommandError> {
    if arg.is_empty() {
        if def_filename.is_empty() && !traditional {
            return Err(CommandError::NoCurrentFilename);
        }
    } else if arg.starts_with('!') {
        // The f arm rejects a name that still begins with '!' after
        // get_filename. Shell expansion of that form is out of scope.
        return Err(CommandError::InvalidRedirection);
    } else {
        // set_def_filename copies the operand and installs it. The
        // read-only flag lives with file I/O, not in this seat.
        *def_filename = arg.to_owned();
    }
    print_escaped(def_filename, out)?;
    out.write_all(b"\n")?;
    Ok(())
}

/// Stdout side of GNU ed 1.22.4 main.c `print_escaped`.
///
/// Bytes in 0x20..=0x7e are copied, except `'\\'`, which is doubled.
/// Every other byte is a backslash plus an [`escchar`] letter, or three
/// octal digits when [`escchar`] returns 0. The C function stops at a
/// NUL because its argument is a C string; a Rust string ends at its
/// length, so every byte here is visited.
fn print_escaped(s: &str, out: &mut dyn Write) -> Result<(), CommandError> {
    for &ch in s.as_bytes() {
        if ch == b'\\' {
            out.write_all(b"\\\\")?;
        } else if (0x20..0x7f).contains(&ch) {
            out.write_all(&[ch])?;
        } else {
            out.write_all(b"\\")?;
            let mapped = escchar(ch);
            if mapped == 0 {
                let octal = [b'0' + (ch >> 6), b'0' + ((ch >> 3) & 7), b'0' + (ch & 7)];
                out.write_all(&octal)?;
            } else {
                out.write_all(&[mapped])?;
            }
        }
    }
    Ok(())
}

/// GNU ed 1.22.4 io.c `escchar`.
///
/// `strchr("\a\b\f\n\r\t\v", ch)` indexes `"abfnrtv"`. Any other byte,
/// including NUL, returns 0 so the caller prints octal.
fn escchar(ch: u8) -> u8 {
    match ch {
        0x07 => b'a',
        0x08 => b'b',
        0x0c => b'f',
        b'\n' => b'n',
        b'\r' => b'r',
        b'\t' => b't',
        0x0b => b'v',
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(arg: &str, name: &str, traditional: bool) -> (Result<(), CommandError>, String, String) {
        let mut filename = name.to_owned();
        let mut out = Vec::new();
        let result = command_f(arg, &mut filename, traditional, &mut out);
        let text = String::from_utf8(out).expect("escaped filename is ASCII");
        (result, text, filename)
    }

    #[test]
    fn prints_current_name_then_sets_and_prints_again() {
        let (result, text, filename) = run("", "in.txt", false);
        assert!(result.is_ok());
        assert_eq!(text, "in.txt\n");
        assert_eq!(filename, "in.txt");

        let (result, text, filename) = run("renamed.txt", "in.txt", false);
        assert!(result.is_ok());
        assert_eq!(text, "renamed.txt\n");
        assert_eq!(filename, "renamed.txt");

        let (result, text, filename) = run("", &filename, false);
        assert!(result.is_ok());
        assert_eq!(text, "renamed.txt\n");
        assert_eq!(filename, "renamed.txt");
    }

    #[test]
    fn empty_name_without_traditional_is_an_error() {
        let (result, text, filename) = run("", "", false);
        assert_eq!(result, Err(CommandError::NoCurrentFilename));
        assert_eq!(text, "");
        assert_eq!(filename, "");
    }

    #[test]
    fn traditional_prints_an_empty_name() {
        let (result, text, filename) = run("", "", true);
        assert!(result.is_ok());
        assert_eq!(text, "\n");
        assert_eq!(filename, "");
    }

    #[test]
    fn bang_is_invalid_redirection_and_does_not_replace_the_name() {
        for arg in ["!", "!ls", "! cmd"] {
            let (result, text, filename) = run(arg, "in.txt", false);
            assert_eq!(result, Err(CommandError::InvalidRedirection));
            assert_eq!(text, "");
            assert_eq!(filename, "in.txt");
        }
    }

    #[test]
    fn escapes_like_gnu_print_escaped() {
        let name = "A\u{7}\u{8}\u{c}\r\t\u{b}\u{1}\u{1f} ~\u{7f}C\\D café";
        let (result, text, filename) = run(name, "", false);
        assert!(result.is_ok());
        assert_eq!(
            text,
            "A\\a\\b\\f\\r\\t\\v\\001\\037 ~\\177C\\\\D caf\\303\\251\n"
        );
        assert_eq!(filename, name);
    }

    #[test]
    fn keeps_spaces_in_the_stored_name() {
        let (result, text, filename) = run("foo bar ", "", true);
        assert!(result.is_ok());
        assert_eq!(text, "foo bar \n");
        assert_eq!(filename, "foo bar ");
    }
}
