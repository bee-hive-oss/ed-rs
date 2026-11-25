//! Move lines (the m command).
//!
//! Ports the 'm' arm of GNU ed 1.22.4 main_loop.c exec_command and buffer.c move_lines.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'm': `range::set_addr_range2`, then `range::get_third_addr`. A
/// destination with first <= dest < second is `Message("Invalid
/// destination")`. Then `args::get_command_suffix`; unless `isglobal`,
/// take an undo snapshot
/// (`ed.snapshot()`). Moving to first-1 or second changes nothing but
/// the current line (second) and still marks the buffer modified;
/// otherwise `ed.buffer.move_lines`. When `isglobal`, call
/// `ed.active.on_move`.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    super::range::set_addr_range2(ed)?;
    let dest = super::range::get_third_addr(ed)?;
    let (from, to) = (ed.first, ed.second);
    if from <= dest && dest < to {
        return Err(super::CommandError::Message("Invalid destination"));
    }
    super::args::get_command_suffix(ed)?;
    if !isglobal {
        ed.snapshot();
    }
    let old_len = ed.buffer.len();
    let noop = dest + 1 == from || dest == to;
    if isglobal {
        ed.active.on_move(from, to, dest);
    }
    ed.buffer.move_lines(from, to, dest).map_err(super::buf_err)?;
    if !noop && (to == old_len || dest == old_len) {
        ed.bin.unterminated = false;
    }
    Ok(super::Status::Ok)
}

#[cfg(test)]
mod tests {
    use super::arm;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::Status;

    #[test]
    fn moves_the_current_line_after_zero() {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(["a", "b", "c"]);
        e.buffer.set_current(2).unwrap();
        e.cmd = "0\n".into();
        assert_eq!(arm(&mut e, false).unwrap(), Status::Ok);
        assert_eq!(e.buffer.lines(), ["b".to_string(), "a".into(), "c".into()]);
        assert_eq!(e.buffer.current(), 1);
    }
}
