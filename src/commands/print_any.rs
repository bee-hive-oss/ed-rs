//! Print a range with any mix of the l, n, p suffixes (io.c print_lines
//! and print_line).
//!
//! Used for command suffixes, z, the null command, and G and V. The plain,
//! numbered, and list-only paths also live in print.rs, number.rs, and
//! list.rs for the p, n, and l commands.

#![allow(dead_code)]

use super::editor::Editor;
use super::{CommandError, PFlags};

/// Ports io.c `print_lines` and `print_line` for any `pflags`.
///
/// Address 0 or `from > to` or `to > $` is `InvalidAddress`. For each
/// line: with `n`, print the line number and a tab first; with `l`, print
/// in list mode (escchar escapes, octal for other non-printable bytes, a
/// `\` before a long-line break at `ed.window_columns()`, and `$` at the
/// end, using `ed.opts.traditional` as list.rs does); otherwise print the
/// text. Then a newline. Each printed line becomes the current line.
/// List mode escapes by byte: a char from U+0080 to U+00FF is one byte and
/// prints as three-digit octal. Write the text with
/// `io::bytes::write_text` so each char goes out as its byte, and map write
/// errors to `CommandError::Io`.
pub fn print_lines(ed: &mut Editor, from: usize, to: usize, pflags: PFlags) -> Result<(), CommandError> {
    let _ = (ed, from, to, pflags);
    todo!("port io.c print_lines and print_line for any pflags")
}
