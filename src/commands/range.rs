//! Default ranges and the third address (main_loop.c set_addr_range,
//! set_addr_range2, set_second_addr, get_third_addr).
//!
//! These read and write `ed.first`, `ed.second`, and `ed.addr_cnt`, which
//! `exec::exec_command` fills from the address parser.

#![allow(dead_code)]

use super::editor::Editor;
use super::CommandError;

/// Ports main_loop.c `set_addr_range`. When `ed.addr_cnt == 0` use `n,m`.
/// Then `first < 1`, `first > second`, or `second > $` is `InvalidAddress`.
pub fn set_addr_range(ed: &mut Editor, n: usize, m: usize) -> Result<(), CommandError> {
    let _ = (ed, n, m);
    todo!("port main_loop.c set_addr_range")
}

/// Ports main_loop.c `set_addr_range2`: [`set_addr_range`] with `.,.`.
pub fn set_addr_range2(ed: &mut Editor) -> Result<(), CommandError> {
    let _ = ed;
    todo!("port main_loop.c set_addr_range2")
}

/// Ports main_loop.c `set_second_addr`. When `ed.addr_cnt == 0` use
/// `addr`. Then `second < 1` or `second > $` is `InvalidAddress`.
pub fn set_second_addr(ed: &mut Editor, addr: usize) -> Result<(), CommandError> {
    let _ = (ed, addr);
    todo!("port main_loop.c set_second_addr")
}

/// Ports main_loop.c `get_third_addr` for m and t.
///
/// Parse one more address at `ed.pos` with
/// `exec_search::extract_addresses`, then put `first`, `second`, and
/// `addr_cnt` back as they were. With `ed.opts.traditional` and no address
/// written, fail with `Message("Destination expected")`. A result outside
/// `0..=$` is `InvalidAddress`. Return the address.
pub fn get_third_addr(ed: &mut Editor) -> Result<usize, CommandError> {
    let _ = ed;
    todo!("port main_loop.c get_third_addr")
}
