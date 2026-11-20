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
    let _ = (arg, def_filename, traditional, out);
    todo!("port GNU ed 1.22.4 exec_command 'f', set_def_filename, and print_escaped")
}
