//! Global command (the g command) and the shared global runner.
//!
//! Ports the 'g' arm of GNU ed 1.22.6 main_loop.c exec_command, regex.c build_active_list, and the non-interactive part of main_loop.c exec_global.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// Fill `ed.active` with lines in the default range whose match equals `matching`.
pub fn build_active(ed: &mut Editor, matching: bool) -> Result<(), super::CommandError> {
    super::range::set_addr_range(ed, 1, ed.buffer.len())?;
    let input = ed.rest().to_string();
    let extended = ed.opts.extended_regexp;
    let (re, used) = ed.regex.get_compiled_regex(&input, extended)?;
    ed.pos += used;
    ed.active.clear();
    let (from, to) = (ed.first, ed.second);
    for addr in from..=to {
        let hit = re.is_match(ed.buffer.line(addr).map_err(super::buf_err)?);
        if hit == matching {
            ed.active.push(addr);
        }
    }
    Ok(())
}

/// 'g' (and 'v' through `matching` false, from global_v.rs):
/// `isglobal` is `Message("Cannot nest global commands")`. Then
/// `range::set_addr_range(ed, 1, $)`; build the active list (C
/// build_active_list): compile the pattern at `ed.pos` with
/// `RegexState::get_compiled_regex`, clear `ed.active`, and push every line
/// in the range whose match result equals `matching`. Read the command list
/// (the rest of the line, joined with `io::extended::get_extended_line`
/// with strip false; with `ed.opts.traditional` an empty list is `p`).
/// Take one undo snapshot (`ed.snapshot()`) for the whole command. For each line from
/// `ed.active.next()`: make it current, set `ed.cmd` to the command list
/// with `ed.pos` 0, and run `exec::exec_command(ed, true)` until the list
/// is used up. Stop on the first status that is not Ok and return it.
pub fn run_global(ed: &mut Editor, isglobal: bool, matching: bool) -> ArmResult {
    if isglobal {
        return Err(super::CommandError::Message("Cannot nest global commands"));
    }
    build_active(ed, matching)?;
    let list = if ed.opts.traditional && ed.rest() == "\n" {
        "p\n".to_string()
    } else {
        let first = ed.rest().to_string();
        let mut next = || ed.next_line();
        crate::io::extended::get_extended_line(&first, false, &mut next)?
    };
    ed.snapshot();
    while let Some(addr) = ed.active.next() {
        ed.buffer.set_current(addr).map_err(super::buf_err)?;
        ed.cmd.clone_from(&list);
        ed.pos = 0;
        while ed.pos < ed.cmd.len() {
            let status = super::exec::exec_command(ed, true);
            if status != super::Status::Ok {
                return Ok(status);
            }
        }
    }
    Ok(super::Status::Ok)
}

/// 'g' arm: [`run_global`] with `matching` true.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    run_global(ed, isglobal, true)
}

#[cfg(test)]
mod tests {
    use super::arm;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::Status;

    #[test]
    fn deletes_each_matching_line() {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(["foo", "bar", "foo"]);
        e.cmd = "/foo/d\n".into();
        assert_eq!(arm(&mut e, false).unwrap(), Status::Ok);
        assert_eq!(e.buffer.lines(), &["bar".to_string()]);
        assert!(e.buffer.has_snapshot());
    }
}
