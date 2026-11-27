//! Change lines (the c command).
//!
//! Ports the 'c' arm of GNU ed 1.22.6 main_loop.c exec_command, with buffer.c delete_lines and append_lines.

#![allow(dead_code)]

use super::editor::{Editor, InputLines};
use super::ArmResult;

/// 'c': `range::set_addr_range2` and `args::get_command_suffix`; unless
/// `isglobal`, take an undo snapshot
/// (`ed.snapshot()`). Copy the range to the cut buffer
/// (`yank::yank_lines`), delete it (`ed.buffer.delete`), then append input
/// lines (`editor::InputLines`) after the new current line, as an insert
/// when the current line is at or past the old first line (C
/// `current_addr() >= first_addr`). When `isglobal`, call
/// `ed.active.on_delete` and `ed.active.on_insert`.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::range::set_addr_range2(ed)?;
    super::args::get_command_suffix(ed)?;
    if !isglobal {
        ed.snapshot();
    }
    let (from, to) = (ed.first, ed.second);
    let old_len = ed.buffer.len();
    super::yank::yank_lines(ed, from, to);
    if isglobal {
        ed.active.on_delete(from, to);
    }
    ed.buffer.delete(from, to).map_err(super::buf_err)?;
    let addr = ed.buffer.current();
    let insert = addr >= from;
    let added = feed(ed, isglobal, insert, addr)?;
    if isglobal && added > 0 {
        let after = if insert { addr.saturating_sub(1) } else { addr };
        ed.active.on_insert(after, added);
    }
    if to == old_len {
        ed.bin.unterminated = false;
    }
    Ok(super::Status::Ok)
}

fn feed(ed: &mut Editor, isglobal: bool, insert: bool, addr: usize) -> Result<usize, super::CommandError> {
    let Editor { buffer, cmd, pos, stdin, linenum, .. } = &mut *ed;
    let before = buffer.len();
    let input = InputLines { isglobal, cmd, pos, stdin: stdin.as_mut(), linenum };
    if insert {
        super::insert::insert_lines(buffer, input, addr)?;
    } else {
        super::append::append_lines(buffer, input, addr)?;
    }
    Ok(buffer.len() - before)
}
