//! Mark a line (the k command).
//!
//! Ports the 'k' arm of GNU ed 1.22.4 main_loop.c exec_command and main_loop.c mark_line_node.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'k': take the mark letter (the next character). `ed.second == 0` is
/// `InvalidAddress`. Then `args::get_command_suffix`, then
/// `ed.buffer.set_mark`; a bad letter is `Message("Invalid mark
/// character")` (use `buf_err`).
pub fn arm(ed: &mut Editor, _isglobal: bool) -> ArmResult {
    let mark = ed.bump().unwrap_or('\0');
    if ed.second == 0 {
        return Err(super::CommandError::InvalidAddress);
    }
    super::args::get_command_suffix(ed)?;
    ed.buffer.set_mark(ed.second, mark).map_err(super::buf_err)?;
    Ok(super::Status::Ok)
}

#[cfg(test)]
mod tests {
    use super::arm;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::Status;

    #[test]
    fn marks_the_addressed_line() {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(["a", "b"]);
        e.second = 2;
        e.cmd = "a\n".into();
        assert_eq!(arm(&mut e, false).unwrap(), Status::Ok);
        assert_eq!(e.buffer.get_mark('a').unwrap(), 2);
    }
}
