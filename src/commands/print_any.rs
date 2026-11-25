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
    let last = ed.buffer.len();
    if from == 0 || from > to || to > last {
        return Err(CommandError::InvalidAddress);
    }
    let traditional = ed.opts.traditional;
    let cols = ed.window_columns();
    for addr in from..=to {
        let text = ed.buffer.line(addr).unwrap_or("").to_string();
        ed.buffer.set_current(addr).map_err(|_| CommandError::InvalidAddress)?;
        let rendered = render(&text, addr, pflags, traditional, cols);
        crate::io::bytes::write_text(&mut std::io::stdout(), &rendered)
            .map_err(|err| CommandError::Io(err.kind()))?;
    }
    Ok(())
}

fn render(text: &str, addr: usize, flags: PFlags, traditional: bool, cols: usize) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    if flags.n {
        out.push_str(&format!("{addr}\t"));
        col = 8;
    }
    if flags.l {
        for ch in text.chars() {
            let b = u8::try_from(ch as u32).unwrap_or(b'?');
            col += 1;
            if col > cols {
                col = 1;
                out.push_str("\\\n");
            }
            if (32..=126).contains(&b) {
                if b == b'$' || b == b'\\' {
                    col += 1;
                    out.push('\\');
                }
                out.push(ch);
            } else {
                col += 1;
                out.push('\\');
                match esc(b) {
                    Some(letter) => out.push(letter),
                    None => {
                        col += 2;
                        out.push((b'0' + (b >> 6)) as char);
                        out.push((b'0' + ((b >> 3) & 7)) as char);
                        out.push((b'0' + (b & 7)) as char);
                    }
                }
            }
        }
        if !traditional {
            out.push('$');
        }
    } else {
        out.push_str(text);
    }
    out.push('\n');
    out
}

fn esc(b: u8) -> Option<char> {
    match b {
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
