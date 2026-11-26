//! Write buffer lines to a file or a command (io.c write_file and
//! write_stream).

#![allow(dead_code)]

use crate::commands::CommandError;
use std::io::Write;
use std::path::PathBuf;

/// Ports io.c `write_file` and `write_stream`.
///
/// Write lines `range.0..=range.1` (from, to) of `lines` (1-indexed, each encoded with
/// `io::bytes::encode` and followed by a newline) to `name`. When
/// `unterminated_last` is true (the caller passes `ed.bin.isbinary &&
/// ed.bin.unterminated`), the buffer's last line gets no newline, as
/// write_stream does. A `!` first means a shell command: send the bytes to
/// `shell::shell_write` with the text after the `!`. Otherwise create or
/// truncate the file (open it with `io::bytes::to_path`), or append when `append` is true (the W command).
/// `from == 0` writes nothing (an empty buffer writes an empty file).
///
/// When not `scripted` print the byte count and a newline to `out` (C
/// `%lu\n`). Return the number of lines written: `to - from + 1` when
/// `from > 0 && from <= to`, else 0.
///
/// Errors: `Message("Cannot open output file")`,
/// `Message("Cannot write file")`, `Message("Cannot close output file")`.
pub fn write_file(
    lines: &[String],
    unterminated_last: bool,
    name: &str,
    append: bool,
    range: (usize, usize),
    scripted: bool,
    out: &mut dyn Write,
) -> Result<usize, CommandError> {
    let (from, to) = range;
    let encode: fn(&str) -> Vec<u8> = crate::io::bytes::encode;
    let mut bytes = Vec::new();
    let count = if from > 0 && from <= to { to - from + 1 } else { 0 };
    let last = lines.len();
    if from > 0 && from <= to && from <= last {
        for addr in from..=to.min(last) {
            bytes.extend(encode(&lines[addr - 1]));
            if !(unterminated_last && addr == last) {
                bytes.push(b'\n');
            }
        }
    }
    if let Some(cmd) = name.strip_prefix('!') {
        crate::io::shell::shell_write(cmd, &bytes).map_err(shell_err)?;
    } else {
        let to_path: fn(&str) -> PathBuf = crate::io::bytes::to_path;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(!append)
            .append(append)
            .open(to_path(name))
            .map_err(|_| CommandError::Message("Cannot open output file"))?;
        file.write_all(&bytes)
            .map_err(|_| CommandError::Message("Cannot write file"))?;
        file.flush()
            .map_err(|_| CommandError::Message("Cannot close output file"))?;
    }
    if !scripted {
        writeln!(out, "{}", bytes.len()).map_err(|_| CommandError::Message("Cannot write file"))?;
    }
    Ok(count)
}

fn shell_err(err: std::io::Error) -> CommandError {
    let msg = match err.kind() {
        std::io::ErrorKind::NotFound => "Cannot open output file",
        std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::WriteZero => "Cannot write file",
        _ => "Cannot close output file",
    };
    CommandError::Message(msg)
}

#[cfg(test)]
mod tests {
    use super::write_file;
    use crate::commands::CommandError;

    #[test]
    fn writes_lines_and_omits_a_binary_tail_newline() {
        let dir = std::env::temp_dir().join(format!("ed-rs-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t");
        let name = path.to_str().unwrap();
        let lines = vec!["a".to_string(), "b\u{ff}".to_string()];
        let mut out = Vec::new();
        let n = write_file(&lines, true, name, false, (1, 2), false, &mut out).unwrap();
        assert_eq!(n, 2);
        assert_eq!(std::fs::read(&path).unwrap(), b"a\nb\xff");
        assert_eq!(String::from_utf8(out).unwrap(), "4\n");
        let err = write_file(&lines, false, "/no/such/dir/ed-rs", false, (1, 1), true, &mut Vec::new());
        assert_eq!(err, Err(CommandError::Message("Cannot open output file")));
        let _ = std::fs::remove_dir_all(dir);
    }
}
