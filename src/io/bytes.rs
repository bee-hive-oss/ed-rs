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

use std::ffi::OsStr;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// One char per byte: byte b becomes `char::from(b)`.
pub fn decode(bytes: &[u8]) -> String {
    bytes.iter().copied().map(char::from).collect()
}

/// The inverse of [`decode`]: each char becomes one byte. A char above
/// U+00FF cannot come from [`decode`]; write it as `?` (it never should
/// happen).
pub fn encode(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
        .collect()
}

/// A file name from the command line or the buffer, as an OS path built
/// from the encoded bytes (use `std::os::unix::ffi::OsStrExt`).
pub fn to_path(name: &str) -> PathBuf {
    PathBuf::from(OsStr::from_bytes(&encode(name)))
}

/// Write `text` to `out` as encoded bytes.
pub fn write_text(out: &mut dyn Write, text: &str) -> io::Result<()> {
    out.write_all(&encode(text))
}

#[cfg(test)]
mod tests {
    use super::{decode, encode, to_path, write_text};
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn every_byte_round_trips_and_high_chars_become_question() {
        let raw: Vec<u8> = (0..=255).collect();
        let text = decode(&raw);
        assert_eq!(text.chars().count(), 256);
        assert_eq!(encode(&text), raw);
        assert_eq!(encode("A\u{100}B"), b"A?B");
        let mut out = Vec::new();
        write_text(&mut out, &text).unwrap();
        assert_eq!(out, raw);
        assert_eq!(to_path("caf\u{e9}").as_os_str().as_bytes(), b"caf\xe9");
    }
}
