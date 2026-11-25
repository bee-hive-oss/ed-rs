//! Running shell commands for `!`, `r !cmd` and `w !cmd` (the popen,
//! pclose and system calls in io.c and main_loop.c).
//!
//! Pass the command to `sh -c` as bytes from `io::bytes::encode` (through
//! `std::os::unix::ffi::OsStrExt`), so non-ASCII bytes reach the shell
//! unchanged.

#![allow(dead_code)]

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::process::{Command, Stdio};

/// `sh -c` with the command encoded the way file names and buffer text are.
fn sh(cmd: &str) -> Command {
    let encode: fn(&str) -> Vec<u8> = crate::io::bytes::encode;
    let arg = OsString::from(OsStr::from_bytes(&encode(cmd)));
    let mut command = Command::new("sh");
    command.arg("-c").arg(arg);
    command
}

/// Run `cmd` with `sh -c` and return its standard output (popen "r").
/// Standard input and error are inherited. Wait for it to finish.
pub fn shell_read(cmd: &str) -> io::Result<Vec<u8>> {
    Ok(sh(cmd)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .output()?
        .stdout)
}

/// Run `cmd` with `sh -c`, feed `data` to its standard input, and wait
/// (popen "w"). Standard output and error are inherited.
pub fn shell_write(cmd: &str, data: &[u8]) -> io::Result<()> {
    let mut child = sh(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let written = write_all_stdin(&mut child, data);
    let waited = child.wait();
    written?;
    waited?;
    Ok(())
}

/// Run `cmd` with `sh -c`, feed `data` to its standard input, and return
/// its standard output and standard error together, in the order they were
/// produced (the `cmd > tmpfile 2>&1` filter in command_shell).
pub fn shell_filter(cmd: &str, data: &[u8]) -> io::Result<Vec<u8>> {
    let path = std::env::temp_dir().join(format!(
        "ed-rs-filter-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let file = std::fs::File::create(&path)?;
    let stdout = Stdio::from(file.try_clone()?);
    let stderr = Stdio::from(file.try_clone()?);
    drop(file);
    let mut child = sh(cmd)
        .stdin(Stdio::piped())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()?;
    let written = write_all_stdin(&mut child, data);
    let waited = child.wait();
    let bytes = std::fs::read(&path);
    let _ = std::fs::remove_file(&path);
    written?;
    waited?;
    bytes
}

/// Run `cmd` with `sh -c` with all three streams inherited and wait
/// (system). Err only when the shell cannot be started.
pub fn shell_system(cmd: &str) -> io::Result<()> {
    sh(cmd).status()?;
    Ok(())
}

fn write_all_stdin(child: &mut std::process::Child, data: &[u8]) -> io::Result<()> {
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(data)?;
    }
    Ok(())
}
