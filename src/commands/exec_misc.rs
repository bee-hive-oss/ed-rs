//! Dispatcher arms for q, Q, =, f, and # (main_loop.c exec_command).

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'q' and 'Q': `args::unexpected_address`, `args::get_command_suffix`,
/// then `quit::quit(force, ed.buffer.is_modified(), ed.warned)` with
/// `force` true for Q.
pub fn arm_quit(ed: &mut Editor, force: bool) -> ArmResult {
    super::args::unexpected_address(ed.addr_cnt)?;
    super::args::get_command_suffix(ed)?;
    Ok(super::quit::quit(force, ed.buffer.is_modified(), ed.warned))
}

/// '=': `args::get_command_suffix`, then `equal::print_line_number`.
pub fn arm_equal(ed: &mut Editor) -> ArmResult {
    super::args::get_command_suffix(ed)?;
    super::equal::print_line_number(ed.addr_cnt, ed.second, ed.buffer.len(), &mut std::io::stdout())?;
    Ok(super::Status::Ok)
}

/// 'f': `args::unexpected_address`, `args::unexpected_command_suffix`,
/// `io::getname::get_filename(ed, ed.opts.traditional)`, then
/// `filename::command_f` on `ed.def_filename`. A new name also clears
/// `ed.read_only` (C set_def_filename).
pub fn arm_f(ed: &mut Editor) -> ArmResult {
    super::args::unexpected_address(ed.addr_cnt)?;
    super::args::unexpected_command_suffix(ed)?;
    let name = crate::io::getname::get_filename(ed, ed.opts.traditional)?;
    if !name.is_empty() {
        ed.read_only = false;
    }
    super::filename::command_f(&name, &mut ed.def_filename, ed.opts.traditional, &mut std::io::stdout())?;
    Ok(super::Status::Ok)
}

/// '#': skip to just past the newline (a comment).
pub fn arm_comment(ed: &mut Editor) -> ArmResult {
    match ed.rest().find('\n') {
        Some(i) => ed.pos += i + 1,
        None => ed.pos = ed.cmd.len(),
    }
    Ok(super::Status::Ok)
}
