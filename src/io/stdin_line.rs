//! Read one script line from stdin (io.c get_stdin_line).

#![allow(dead_code)]

use crate::commands::CommandError;
use std::io::BufRead;

/// Ports io.c `get_stdin_line`.
///
/// Read up to and including the next newline. Return `Ok(Some(line))` with
/// the trailing newline kept, and add 1 to `*linenum`. At end of input
/// return `Ok(None)`; a final line with no newline is discarded (but still
/// counted in `*linenum` when it had any text), as the C code does. A read
/// error is `Err(Message("Cannot read stdin"))`. Read bytes with
/// `read_until(b'\n', ..)` and convert with `io::bytes::decode`.
pub fn get_stdin_line(
    input: &mut dyn BufRead,
    linenum: &mut usize,
) -> Result<Option<String>, CommandError> {
    let mut buf = Vec::new();
    let n = input
        .read_until(b'\n', &mut buf)
        .map_err(|_| CommandError::Message("Cannot read stdin"))?;
    if n == 0 {
        return Ok(None);
    }
    if buf.last() != Some(&b'\n') {
        if !buf.is_empty() {
            *linenum += 1;
        }
        return Ok(None);
    }
    *linenum += 1;
    Ok(Some(crate::io::bytes::decode(&buf)))
}

#[cfg(test)]
mod tests {
    use super::get_stdin_line;
    use std::io::Cursor;

    #[test]
    fn keeps_newlines_drops_a_partial_last_line_and_counts() {
        let mut input = Cursor::new(b"ab\n\nxy".to_vec());
        let mut n = 0usize;
        assert_eq!(get_stdin_line(&mut input, &mut n).unwrap().as_deref(), Some("ab\n"));
        assert_eq!(get_stdin_line(&mut input, &mut n).unwrap().as_deref(), Some("\n"));
        assert_eq!(get_stdin_line(&mut input, &mut n).unwrap(), None);
        assert_eq!(n, 3);
        assert_eq!(get_stdin_line(&mut input, &mut n).unwrap(), None);
        assert_eq!(n, 3);
        let mut latin = Cursor::new([0xff, b'\n']);
        let mut n = 0usize;
        let line = get_stdin_line(&mut latin, &mut n).unwrap().unwrap();
        assert_eq!(line.chars().next().map(u32::from), Some(0xff));
        assert!(line.ends_with('\n'));
    }
}
