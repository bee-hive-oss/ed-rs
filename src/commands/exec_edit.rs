//! Dispatcher arms for a, i, and d (the 'a', 'i', 'd' cases of
//! main_loop.c exec_command).
//!
//! The line work is in append.rs, insert.rs, and delete.rs. These arms add
//! the suffix check, the undo snapshot, the cut buffer, and global upkeep.
//! Each arm also clears `ed.bin.unterminated` when it changes the last line
//! or adds lines after it (in C that line node is then no longer the
//! unterminated last line).

#![allow(dead_code)]

use super::editor::{Editor, InputLines};
use super::ArmResult;

/// 'a': `args::get_command_suffix`; unless `isglobal`, take an undo
/// snapshot with `ed.snapshot()`; then `append::append_lines` after `ed.second`, reading lines
/// from `editor::InputLines`. If no line was added, drop the snapshot
/// (GNU pushes no undo atom). When `isglobal`, call
/// `ed.active.on_insert` for the added lines.
pub fn arm_a(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::args::get_command_suffix(ed)?;
    feed(ed, isglobal, false)
}

/// 'i': as [`arm_a`] with `insert::insert_lines` at `ed.second`.
pub fn arm_i(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::args::get_command_suffix(ed)?;
    feed(ed, isglobal, true)
}

/// 'd': `range::set_addr_range2`, `args::get_command_suffix`, unless
/// `isglobal` an undo snapshot (`ed.snapshot()`), then `yank::yank_lines` of the range (as
/// buffer.c delete_lines does), `delete::delete_lines`, and when
/// `isglobal` `ed.active.on_delete`.
pub fn arm_d(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::range::set_addr_range2(ed)?;
    super::args::get_command_suffix(ed)?;
    if !isglobal {
        ed.snapshot();
    }
    let (from, to) = (ed.first, ed.second);
    let old_len = ed.buffer.len();
    super::yank::yank_lines(ed, from, to);
    super::delete::delete_lines(&mut ed.buffer, from, to)?;
    if to == old_len {
        ed.bin.unterminated = false;
    }
    if isglobal {
        ed.active.on_delete(from, to);
    }
    Ok(super::Status::Ok)
}

fn feed(ed: &mut Editor, isglobal: bool, insert: bool) -> ArmResult {
    if !isglobal {
        ed.snapshot();
    }
    let old_len = ed.buffer.len();
    let addr = ed.second;
    let added = {
        let Editor { buffer, cmd, pos, stdin, linenum, .. } = &mut *ed;
        let before = buffer.len();
        let input = InputLines { isglobal, cmd, pos, stdin: stdin.as_mut(), linenum };
        if insert {
            super::insert::insert_lines(buffer, input, addr)?;
        } else {
            super::append::append_lines(buffer, input, addr)?;
        }
        buffer.len() - before
    };
    if added == 0 && !isglobal {
        ed.buffer.clear_snapshot();
    }
    let at_end = if insert { addr > old_len } else { addr == old_len };
    if added > 0 && at_end {
        ed.bin.unterminated = false;
    }
    if isglobal && added > 0 {
        ed.active.on_insert(addr.saturating_sub(usize::from(insert)), added);
    }
    Ok(super::Status::Ok)
}
