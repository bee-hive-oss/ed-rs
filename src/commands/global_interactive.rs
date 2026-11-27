//! Interactive global command (the G command).
//!
//! Ports the 'G' arm of GNU ed 1.22.6 main_loop.c exec_command and the interactive part of main_loop.c exec_global.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'G' (and 'V' through `matching` false, from global_interactive_v.rs):
/// as `global::run_global` up to the active list, then
/// `args::get_command_suffix`. Take one undo snapshot (`ed.snapshot()`). For each active
/// line: make it current, print it with `print_any::print_lines` and the
/// suffix flags, then read one command line from stdin
/// (`io::stdin_line::get_stdin_line`; end of input or an error ends with
/// Err). A bare newline skips the line. `&` reruns the previous command
/// list (`Message("No previous command")` if none). Otherwise join
/// continuation lines and keep it as the command list. Run the list with
/// `exec::exec_command(ed, true)` as `run_global` does. Clear `ed.pflags`
/// so the dispatcher does not print again.
pub fn run_global_interactive(ed: &mut Editor, isglobal: bool, matching: bool) -> ArmResult {
    if isglobal {
        return Err(super::CommandError::Message("Cannot nest global commands"));
    }
    super::global::build_active(ed, matching)?;
    super::args::get_command_suffix(ed)?;
    let flags = ed.pflags;
    ed.snapshot();
    let mut prev: Option<String> = None;
    while let Some(addr) = ed.active.next() {
        ed.buffer.set_current(addr).map_err(super::buf_err)?;
        super::print_any::print_lines(ed, addr, addr, flags)?;
        let raw = match crate::io::stdin_line::get_stdin_line(&mut *ed.stdin, &mut ed.linenum) {
            Ok(Some(line)) => line,
            Ok(None) => return Err(super::CommandError::Message("Unexpected end-of-file")),
            Err(err) => return Err(err),
        };
        let list = if raw == "\n" {
            continue;
        } else if raw == "&\n" {
            prev.clone().ok_or(super::CommandError::Message("No previous command"))?
        } else {
            let mut next = || ed.next_line();
            let joined = crate::io::extended::get_extended_line(&raw, false, &mut next)?;
            prev = Some(joined.clone());
            joined
        };
        ed.cmd = list;
        ed.pos = 0;
        while ed.pos < ed.cmd.len() {
            let status = super::exec::exec_command(ed, true);
            if status != super::Status::Ok {
                ed.pflags = super::PFlags::default();
                return Ok(status);
            }
        }
    }
    ed.pflags = super::PFlags::default();
    Ok(super::Status::Ok)
}

/// 'G' arm: [`run_global_interactive`] with `matching` true.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    run_global_interactive(ed, isglobal, true)
}
