//! Read a file or command output into the buffer (io.c read_file and
//! read_stream).

#![allow(dead_code)]

use crate::buffer::Buffer;
use crate::commands::CommandError;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

/// Binary-file state of the buffer (io.c and buffer.c statics).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BinaryState {
    /// The buffer holds a NUL byte from a read (C isbinary_). Reset by e
    /// and E.
    pub isbinary: bool,
    /// The last line came from a binary read with no final newline and has
    /// not been changed since (C unterminated_last_line()). w then writes it
    /// without a newline.
    pub unterminated: bool,
}

/// What a read did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOutcome {
    /// Lines added to the buffer.
    pub lines: usize,
    /// True when the file could be opened for reading but not for writing
    /// (C read_onlyp).
    pub read_only: bool,
}

/// Ports io.c `read_file`, `read_stream`, and `read_stream_line`.
///
/// `name` is a path (open it with `io::bytes::to_path`), or a shell
/// command when it starts with `!` (use `shell::shell_read` on the text
/// after the `!`). Read all bytes and split them into lines at each
/// newline. A NUL byte sets `bin.isbinary`. With `strip_cr`, remove a CR
/// only when it sits right before a newline. Decode each line with
/// `io::bytes::decode`. A last line with no newline is kept and counts as
/// "newline added".
///
/// Append the lines to `buffer` after `addr` with `Buffer::append`. Then put
/// the dirty flag back to what it was before the read (a read does not dirty
/// the buffer; the r command sets it). The current line is the last line
/// read, or `addr` when nothing was read.
///
/// When not `scripted`, follow read_stream exactly: print "Newline
/// inserted" when appending at the end after an unterminated last line,
/// else "Newline appended" when a newline was added and (not appending at
/// the end, or the buffer is not binary). Then print the byte count and a
/// newline (C `%lu\n`), counting one more byte in the case read_stream
/// does. Set `bin.unterminated` when appending at the end of a binary
/// buffer and a newline was added (or nothing was read), as read_stream
/// does; appending after an unterminated last line clears it.
///
/// Errors: `Message("Cannot open input file")` when the file or command
/// cannot be opened or run, `Message("Cannot read input file")` on a read
/// error. A missing file must not change the buffer.
pub fn read_file(
    buffer: &mut Buffer,
    bin: &mut BinaryState,
    name: &str,
    addr: usize,
    strip_cr: bool,
    scripted: bool,
    out: &mut dyn Write,
) -> Result<ReadOutcome, CommandError> {
    let (data, read_only) = load_bytes(name, buffer.is_modified())?;
    let split = split_lines(&data, strip_cr, bin.isbinary);
    let appended = addr == buffer.len();
    let was_unterminated = bin.unterminated;
    let was_binary = bin.isbinary;
    let dirty = buffer.is_modified();
    let decode: fn(&[u8]) -> String = crate::io::bytes::decode;
    let text: Vec<String> = split.lines.iter().map(|line| decode(line)).collect();
    let n = text.len();
    buffer.append(addr, text).map_err(|_| CommandError::InvalidAddress)?;
    buffer.set_modified(dirty);
    if split.saw_nul { bin.isbinary = true; }
    if appended && was_unterminated { bin.unterminated = false; }
    if appended && bin.isbinary && (split.newline_added || split.total == 0) {
        bin.unterminated = true;
    }
    let mut total = split.total;
    if !appended && bin.isbinary && !was_binary && split.newline_added {
        total += 1;
    }
    if !scripted {
        if addr != 0 && appended && split.total > 0 && was_unterminated {
            let _ = writeln!(out, "Newline inserted");
        } else if split.newline_added && (!appended || !bin.isbinary) {
            let _ = writeln!(out, "Newline appended");
        }
        let _ = writeln!(out, "{total}");
    }
    Ok(ReadOutcome { lines: n, read_only })
}

struct Split {
    lines: Vec<Vec<u8>>,
    total: usize,
    newline_added: bool,
    saw_nul: bool,
}

fn split_lines(data: &[u8], strip_cr: bool, already_binary: bool) -> Split {
    let mut lines = Vec::new();
    let mut cur = Vec::new();
    let (mut total, mut saw_nul) = (0usize, false);
    for &b in data {
        saw_nul |= b == 0;
        if b != b'\n' {
            cur.push(b);
            continue;
        }
        if strip_cr && cur.last() == Some(&b'\r') {
            cur.pop();
        }
        total += cur.len() + 1;
        lines.push(std::mem::take(&mut cur));
    }
    let newline_added = !cur.is_empty();
    if newline_added {
        total += cur.len() + usize::from(!(saw_nul || already_binary));
        lines.push(cur);
    }
    Split { lines, total, newline_added, saw_nul }
}

fn load_bytes(name: &str, modified: bool) -> Result<(Vec<u8>, bool), CommandError> {
    let open_err = CommandError::Message("Cannot open input file");
    if let Some(cmd) = name.strip_prefix('!') {
        return Ok((crate::io::shell::shell_read(cmd).map_err(|_| open_err)?, false));
    }
    let to_path: fn(&str) -> PathBuf = crate::io::bytes::to_path;
    let path = to_path(name);
    let (mut file, read_only) = match std::fs::OpenOptions::new().read(true).write(true).open(&path) {
        Ok(file) => (file, false),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Err(open_err),
        Err(_) => (File::open(&path).map_err(|_| open_err)?, !modified),
    };
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|_| CommandError::Message("Cannot read input file"))?;
    Ok((data, read_only))
}
