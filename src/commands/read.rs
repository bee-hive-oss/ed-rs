//! Read a file into the buffer (the r command).
//!
//! Ports the 'r' arm of GNU ed 1.22.6 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'r': `args::unexpected_command_suffix`; with no address use `$`.
/// `io::getname::get_filename(ed, false)`. If there is no default filename
/// and the operand does not start with `!`, make it the default. Unless
/// `isglobal`, take an undo snapshot
/// (`ed.snapshot()`). `io::read::read_file` (with `&mut ed.bin`) after the
/// address (operand, or the default filename when empty), honoring
/// `ed.opts.strip_trailing_cr` and printing the byte count unless
/// `ed.opts.silent`. If any line was read, mark the buffer modified. When
/// `isglobal`, call `ed.active.on_insert`.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::args::unexpected_command_suffix(ed)?;
    if ed.addr_cnt == 0 {
        ed.second = ed.buffer.len();
    }
    let name = crate::io::getname::get_filename(ed, false)?;
    if ed.def_filename.is_empty() && !name.is_empty() && !name.starts_with('!') {
        ed.def_filename.clone_from(&name);
        ed.read_only = false;
    }
    if !isglobal {
        ed.snapshot();
    }
    let addr = ed.second;
    let target = if name.is_empty() { ed.def_filename.clone() } else { name };
    let strip = ed.opts.strip_trailing_cr;
    let scripted = ed.opts.silent;
    let outcome = crate::io::read::read_file(
        &mut ed.buffer,
        &mut ed.bin,
        &target,
        addr,
        strip,
        scripted,
        &mut std::io::stdout(),
    )?;
    if outcome.lines > 0 {
        ed.buffer.set_modified(true);
        if isglobal {
            ed.active.on_insert(addr, outcome.lines);
        }
    }
    Ok(super::Status::Ok)
}
