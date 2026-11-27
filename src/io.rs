//! File I/O, shell escapes, prompts, and errors.
//!
//! Small files under src/io/ hold the ports of GNU ed 1.22.6 io.c and the
//! file-name side of main_loop.c. The old `read_file` placeholder stays
//! until the command loop is wired, so existing callers still compile.
#![allow(dead_code)]

pub mod bytes;
pub mod extended;
pub mod getname;
pub mod read;
pub mod shell;
pub mod shellcmd;
pub mod stdin_line;
pub mod write;

pub fn read_file(_path: &str) -> std::io::Result<Vec<String>> {
    Ok(Vec::new())
}
