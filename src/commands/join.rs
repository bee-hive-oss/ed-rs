//! Join lines (the j command).
//!
//! Ports the 'j' arm of GNU ed 1.22.6 main_loop.c exec_command and buffer.c join_lines.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'j': `range::set_addr_range(ed, ., .+1)` and
/// `args::get_command_suffix`; unless `isglobal`, take an undo snapshot
/// (`ed.snapshot()`).
/// Only when first < second: copy the range to the cut buffer
/// (`yank::yank_lines`, since join_lines calls delete_lines), then
/// `ed.buffer.join`. When `isglobal`, call `ed.active.on_delete` for the
/// range and `ed.active.on_insert` for the one new line.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let dot = ed.buffer.current();
    super::range::set_addr_range(ed, dot, dot + 1)?;
    super::args::get_command_suffix(ed)?;
    if !isglobal {
        ed.snapshot();
    }
    let (from, to) = (ed.first, ed.second);
    if from < to {
        let old_len = ed.buffer.len();
        super::yank::yank_lines(ed, from, to);
        if isglobal {
            ed.active.on_delete(from, to);
        }
        ed.buffer.join(from, to).map_err(super::buf_err)?;
        if isglobal {
            ed.active.on_insert(from - 1, 1);
        }
        if to == old_len {
            ed.bin.unterminated = false;
        }
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
    fn joins_the_current_line_with_the_next() {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(["a", "b", "c"]);
        e.buffer.set_current(1).unwrap();
        e.cmd = "\n".into();
        assert_eq!(arm(&mut e, false).unwrap(), Status::Ok);
        assert_eq!(e.buffer.lines(), ["ab".to_string(), "c".into()]);
        assert_eq!(e.cut, ["a".to_string(), "b".into()]);
    }
}
