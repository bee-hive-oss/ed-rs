//! Running shell commands for `!`, `r !cmd` and `w !cmd` (the popen,
//! pclose and system calls in io.c and main_loop.c).
//!
//! Pass the command to `sh -c` as bytes from `io::bytes::encode` (through
//! `std::os::unix::ffi::OsStrExt`), so non-ASCII bytes reach the shell
//! unchanged.

#![allow(dead_code)]

use std::io;

/// Run `cmd` with `sh -c` and return its standard output (popen "r").
/// Standard input and error are inherited. Wait for it to finish.
pub fn shell_read(cmd: &str) -> io::Result<Vec<u8>> {
    let _ = cmd;
    todo!("popen r")
}

/// Run `cmd` with `sh -c`, feed `data` to its standard input, and wait
/// (popen "w"). Standard output and error are inherited.
pub fn shell_write(cmd: &str, data: &[u8]) -> io::Result<()> {
    let _ = (cmd, data);
    todo!("popen w")
}

/// Run `cmd` with `sh -c`, feed `data` to its standard input, and return
/// its standard output and standard error together, in the order they were
/// produced (the `cmd > tmpfile 2>&1` filter in command_shell).
pub fn shell_filter(cmd: &str, data: &[u8]) -> io::Result<Vec<u8>> {
    let _ = (cmd, data);
    todo!("filter through a shell command")
}

/// Run `cmd` with `sh -c` with all three streams inherited and wait
/// (system). Err only when the shell cannot be started.
pub fn shell_system(cmd: &str) -> io::Result<()> {
    let _ = cmd;
    todo!("system")
}
