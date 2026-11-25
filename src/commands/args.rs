//! Small command-line readers shared by the arms (main_loop.c
//! skip_blanks, parse_int, get_command_suffix, unexpected_address,
//! unexpected_command_suffix).
//!
//! Each reads `ed.cmd` at `ed.pos` and moves `ed.pos` forward.

#![allow(dead_code)]

use super::editor::Editor;
use super::CommandError;

/// Ports main_loop.c `skip_blanks`: skip C `isspace` characters other than
/// newline.
pub fn skip_blanks(ed: &mut Editor) {
    let _ = ed;
    todo!("port main_loop.c skip_blanks")
}

/// Ports main_loop.c `parse_int` (strtol base 10, optional sign).
///
/// No digits is `Message("Invalid number")`. A value above `i32::MAX` or
/// below `-i32::MAX` is `Message("Number out of range")`. On success move
/// past the number and return it.
pub fn parse_int(ed: &mut Editor) -> Result<i32, CommandError> {
    let _ = ed;
    todo!("port main_loop.c parse_int")
}

/// Ports main_loop.c `get_command_suffix`.
///
/// Read any of `l`, `n`, `p` into `ed.pflags`, each at most once (a repeat
/// stops the scan). Then the next character must be a newline, which is
/// consumed; anything else is `InvalidCommandSuffix`.
pub fn get_command_suffix(ed: &mut Editor) -> Result<(), CommandError> {
    let _ = ed;
    todo!("port main_loop.c get_command_suffix")
}

/// Ports main_loop.c `unexpected_address`: `UnexpectedAddress` when
/// `addr_cnt > 0`.
pub fn unexpected_address(addr_cnt: usize) -> Result<(), CommandError> {
    let _ = addr_cnt;
    todo!("port main_loop.c unexpected_address")
}

/// Ports main_loop.c `unexpected_command_suffix`: the next character must
/// be C `isspace` (newline included) or the error is
/// `UnexpectedCommandSuffix`. Does not move `ed.pos`.
pub fn unexpected_command_suffix(ed: &Editor) -> Result<(), CommandError> {
    let _ = ed;
    todo!("port main_loop.c unexpected_command_suffix")
}
