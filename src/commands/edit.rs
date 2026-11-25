//! Edit a file without the modified check (the E command).
//!
//! Ports the 'E' arm of GNU ed 1.22.4 main_loop.c exec_command (the e arm falls through to it).

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 'E': `args::unexpected_address` and `args::unexpected_command_suffix`;
/// `io::getname::get_filename(ed, false)`. Empty the buffer
/// (`ed.buffer.load` with no lines), mark it unmodified, reset `ed.bin`
/// (C open_sbuf), and set the
/// default filename to the operand when it is not empty and does not start
/// with `!` (this also clears `ed.read_only`). Read the operand, or the
/// default filename when the operand is empty, with `io::read::read_file`
/// at address 0 with `&mut ed.bin`, honoring `ed.opts.strip_trailing_cr` and printing the byte
/// count unless `ed.opts.silent`. Store its read-only result in
/// `ed.read_only`. Then drop the undo snapshot (C reset_undo_state) so the
/// read cannot be undone. Marks are already cleared by `load`.
pub fn edit_file(ed: &mut Editor, _isglobal: bool) -> ArmResult {
    super::args::unexpected_address(ed.addr_cnt)?;
    super::args::unexpected_command_suffix(ed)?;
    let name = crate::io::getname::get_filename(ed, false)?;
    ed.buffer.load(std::iter::empty::<String>());
    ed.bin = crate::io::read::BinaryState::default();
    if !name.is_empty() && !name.starts_with('!') {
        ed.def_filename.clone_from(&name);
        ed.read_only = false;
    }
    let target = if name.is_empty() { ed.def_filename.clone() } else { name };
    let strip = ed.opts.strip_trailing_cr;
    let scripted = ed.opts.silent;
    let outcome = crate::io::read::read_file(
        &mut ed.buffer,
        &mut ed.bin,
        &target,
        0,
        strip,
        scripted,
        &mut std::io::stdout(),
    )?;
    ed.read_only = outcome.read_only;
    ed.buffer.clear_snapshot();
    Ok(super::Status::Ok)
}

#[cfg(test)]
mod tests {
    use super::edit_file;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::Status;

    #[test]
    fn loads_a_file_and_drops_undo() {
        let dir = std::env::temp_dir().join(format!("ed-rs-edit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t");
        std::fs::write(&path, b"hi\n").unwrap();
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(["old"]);
        e.buffer.set_modified(true);
        e.opts.silent = true;
        e.cmd = format!(" {}\n", path.display());
        assert_eq!(edit_file(&mut e, false).unwrap(), Status::Ok);
        assert_eq!(e.buffer.lines(), ["hi".to_string()]);
        assert!(!e.buffer.is_modified());
        assert_eq!(e.def_filename, path.display().to_string());
        assert!(!e.buffer.has_snapshot());
        let _ = std::fs::remove_dir_all(dir);
    }
}
