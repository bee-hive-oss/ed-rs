//! Bytes in, bytes out.
//!
//! GNU ed works on raw bytes, and the official tests read and write files
//! that are not UTF-8 (test.bin holds every byte 0 to 255). ed-rs keeps
//! text in `String`, one char per byte: byte b is char U+00bb (Latin-1).
//! Everything that reads bytes (stdin, files, command output) decodes with
//! [`decode`], and everything that writes them (stdout, files, pipes, file
//! names) encodes with [`encode`]. Then every byte round-trips exactly, and
//! a regex `.` matches one byte, as GNU ed does under LC_ALL=C.

#![allow(dead_code)]

use std::io::{self, Write};
use std::path::PathBuf;

/// One char per byte: byte b becomes `char::from(b)`.
pub fn decode(bytes: &[u8]) -> String {
    let _ = bytes;
    todo!("map each byte to one char")
}

/// The inverse of [`decode`]: each char becomes one byte. A char above
/// U+00FF cannot come from [`decode`]; write it as `?` (it never should
/// happen).
pub fn encode(text: &str) -> Vec<u8> {
    let _ = text;
    todo!("map each char back to one byte")
}

/// A file name from the command line or the buffer, as an OS path built
/// from the encoded bytes (use `std::os::unix::ffi::OsStrExt`).
pub fn to_path(name: &str) -> PathBuf {
    let _ = name;
    todo!("build a path from encoded bytes")
}

/// Write `text` to `out` as encoded bytes.
pub fn write_text(out: &mut dyn Write, text: &str) -> io::Result<()> {
    let _ = (out, text);
    todo!("write encoded bytes")
}
