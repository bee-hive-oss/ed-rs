//! Suffixes of a full s command (main_loop.c get_command_s_suffix).

#![allow(dead_code)]

use super::editor::Editor;
use super::{CommandError, PFlags};

/// What [`get_command_s_suffix`] read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SSuffix {
    pub pflags: PFlags,
    /// Occurrence to replace: 1 by default, n for a count, 0 for g.
    pub snum: i32,
    pub ignore_case: bool,
}

/// Ports main_loop.c `get_command_s_suffix`.
///
/// Read at `ed.pos`, in any order: a count starting with 1 to 9
/// (read with `args::parse_int`, must be > 0) or `g`, but only one of the two; `i` or
/// `I` once; and `l`, `n`, `p` once each. A second count or g, a count of
/// 0, or anything but a newline at the end is `InvalidCommandSuffix`. A
/// repeated `i`, `l`, `n` or `p` stops the scan (so the newline check then
/// fails). Consume the newline. Start from `pflags` empty, `snum` 1.
pub fn get_command_s_suffix(ed: &mut Editor) -> Result<SSuffix, CommandError> {
    let _ = ed;
    todo!("port main_loop.c get_command_s_suffix")
}
