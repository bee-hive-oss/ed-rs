//! Global command (the g command) and the shared global runner.
//!
//! Ports the 'g' arm of GNU ed 1.22.4 main_loop.c exec_command, regex.c build_active_list, and the non-interactive part of main_loop.c exec_global.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'g' (and 'v' through `matching` false, from global_v.rs):
/// `isglobal` is `Message("Cannot nest global commands")`. Then
/// `range::set_addr_range(ed, 1, $)`; build the active list (C
/// build_active_list): compile the pattern at `ed.pos` with
/// `RegexState::get_compiled_regex`, clear `ed.active`, and push every line
/// in the range whose match result equals `matching`. Read the command list
/// (the rest of the line, joined with `io::extended::get_extended_line`
/// with strip false; with `ed.opts.traditional` an empty list is `p`).
/// Take one undo snapshot (`ed.snapshot()`) for the whole command. For each line from
/// `ed.active.next()`: make it current, set `ed.cmd` to the command list
/// with `ed.pos` 0, and run `exec::exec_command(ed, true)` until the list
/// is used up. Stop on the first status that is not Ok and return it.
pub fn run_global(ed: &mut Editor, isglobal: bool, matching: bool) -> ArmResult {
    let _ = (ed, isglobal, matching);
    todo!("port the 'g' arm of GNU ed 1.22.4 main_loop.c exec_command, regex.c build_active_list, and the non-interactive part of main_loop.c exec_global")
}

/// 'g' arm: [`run_global`] with `matching` true.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("g arm of main_loop.c exec_command")
}
