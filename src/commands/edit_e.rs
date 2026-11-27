//! Edit a file (the e command).
//!
//! Ports the 'e' arm of GNU ed 1.22.6 main_loop.c exec_command.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'e': if the buffer is modified and `ed.warned` is false return
/// `Ok(Status::Emod)`. Otherwise do exactly what E does by calling
/// `edit::edit_file`.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    if ed.buffer.is_modified() && !ed.warned {
        return Ok(super::Status::Emod);
    }
    super::edit::edit_file(ed, isglobal)
}

#[cfg(test)]
mod tests {
    use super::arm;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::Status;

    #[test]
    fn refuses_a_modified_buffer_until_warned() {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(["a"]);
        e.buffer.set_modified(true);
        e.cmd = "\n".into();
        assert_eq!(arm(&mut e, false).unwrap(), Status::Emod);
        assert_eq!(e.buffer.line(1).unwrap(), "a");
    }
}
