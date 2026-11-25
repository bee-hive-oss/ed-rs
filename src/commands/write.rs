//! Write the buffer (the w command, with wq).
//!
//! Ports the 'w' arm of GNU ed 1.22.4 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'w' (and 'W' through `append` true, from write_append.rs): a `q` or
/// `Q` right after the letter is consumed and remembered. Then
/// `args::unexpected_command_suffix` and
/// `io::getname::get_filename(ed, false)`. With no address and an empty
/// buffer write nothing (0,0); otherwise `range::set_addr_range(ed, 1,
/// $)`. If there is no default filename and the operand does not start
/// with `!`, make it the default. `io::write::write_file` with `unterminated_last` set to
/// `ed.bin.isbinary && ed.bin.unterminated` (operand, or the
/// default filename when empty), printing the byte count unless
/// `ed.opts.silent`. If all lines went to a file (not `!`), mark the buffer
/// unmodified; else with `q` and a modified, unwarned buffer return
/// `Status::Emod`. With `q` or `Q` return `Status::Quit`.
pub fn write_lines(ed: &mut Editor, _isglobal: bool, append: bool) -> ArmResult {
    let quit_q = ed.peek() == Some('q');
    let quit = quit_q || ed.peek() == Some('Q');
    if quit {
        ed.bump();
    }
    super::args::unexpected_command_suffix(ed)?;
    let name = crate::io::getname::get_filename(ed, false)?;
    if ed.addr_cnt == 0 && ed.buffer.is_empty() {
        ed.first = 0;
        ed.second = 0;
    } else {
        super::range::set_addr_range(ed, 1, ed.buffer.len())?;
    }
    if ed.def_filename.is_empty() && !name.is_empty() && !name.starts_with('!') {
        ed.def_filename.clone_from(&name);
        ed.read_only = false;
    }
    let target = if name.is_empty() { ed.def_filename.clone() } else { name.clone() };
    let (from, to) = (ed.first, ed.second);
    let unterminated_last = ed.bin.isbinary && ed.bin.unterminated;
    let scripted = ed.opts.silent;
    let len = ed.buffer.len();
    let written = crate::io::write::write_file(
        ed.buffer.lines(),
        unterminated_last,
        &target,
        append,
        (from, to),
        scripted,
        &mut std::io::stdout(),
    )?;
    if written == len && !name.starts_with('!') {
        ed.buffer.set_modified(false);
    } else if quit_q && ed.buffer.is_modified() && !ed.warned {
        return Ok(super::Status::Emod);
    }
    if quit {
        return Ok(super::Status::Quit);
    }
    Ok(super::Status::Ok)
}

/// 'w' arm: [`write_lines`] with `append` false.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    write_lines(ed, isglobal, false)
}
