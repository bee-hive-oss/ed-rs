//! The command dispatcher (main_loop.c exec_command).
//!
//! Parse addresses, read the command letter, and hand off to the arm for
//! that command. Each arm lives in its own file and returns an
//! [`ArmResult`](super::ArmResult).

#![allow(dead_code)]

use super::editor::Editor;
use super::{CommandError, PFlags, Status};

/// Ports main_loop.c `exec_command`.
///
/// Clear `ed.pflags`. Call `exec_search::extract_addresses`, then
/// `args::skip_blanks`, then take one character and call its arm:
/// `a i d` in exec_edit.rs, `p n l` in exec_print.rs, `q Q = f #` in
/// exec_misc.rs, and one file each for `c e E g v G V h H j k m P r s t u w
/// W x y z !` and the null command (a newline). Any other character is
/// `UnknownCommand`. After an arm that returned `Status::Ok`, if
/// `ed.pflags` is not empty, print the current line with
/// `print_any::print_lines`.
///
/// On an error, store its text with `Editor::fail` and return
/// `Status::Err`. Otherwise return the arm's status.
pub fn exec_command(ed: &mut Editor, isglobal: bool) -> Status {
    ed.pflags = PFlags::default();
    if let Err(err) = super::exec_search::extract_addresses(ed) {
        return ed.fail(err);
    }
    super::args::skip_blanks(ed);
    let Some(ch) = ed.bump() else {
        return ed.fail(CommandError::UnknownCommand);
    };
    let result = dispatch(ed, isglobal, ch);
    match result {
        Ok(Status::Ok) if !ed.pflags.is_empty() => {
            let cur = ed.buffer.current();
            let flags = ed.pflags;
            ed.pflags = PFlags::default();
            match super::print_any::print_lines(ed, cur, cur, flags) {
                Ok(()) => Status::Ok,
                Err(err) => ed.fail(err),
            }
        }
        Ok(status) => status,
        Err(err) => ed.fail(err),
    }
}

fn dispatch(ed: &mut Editor, isglobal: bool, ch: char) -> super::ArmResult {
    match ch {
        'a' => super::exec_edit::arm_a(ed, isglobal),
        'c' => super::change::arm(ed, isglobal),
        'd' => super::exec_edit::arm_d(ed, isglobal),
        'e' => super::edit_e::arm(ed, isglobal),
        'E' => super::edit::edit_file(ed, isglobal),
        'f' => super::exec_misc::arm_f(ed),
        'g' => super::global::arm(ed, isglobal),
        'G' => super::global_interactive::arm(ed, isglobal),
        'h' => super::help::arm(ed, isglobal),
        'H' => super::help_verbose::arm(ed, isglobal),
        'i' => super::exec_edit::arm_i(ed, isglobal),
        'j' => super::join::arm(ed, isglobal),
        'k' => super::mark::arm(ed, isglobal),
        'l' => super::exec_print::arm_l(ed, isglobal),
        'm' => super::move_lines::arm(ed, isglobal),
        'n' => super::exec_print::arm_n(ed, isglobal),
        'p' => super::exec_print::arm_p(ed, isglobal),
        'P' => super::prompt::arm(ed, isglobal),
        'q' => super::exec_misc::arm_quit(ed, false),
        'Q' => super::exec_misc::arm_quit(ed, true),
        'r' => super::read::arm(ed, isglobal),
        's' => super::substitute::arm(ed, isglobal),
        't' => super::copy::arm(ed, isglobal),
        'u' => super::undo::arm(ed, isglobal),
        'v' => super::global_v::arm(ed, isglobal),
        'V' => super::global_interactive_v::arm(ed, isglobal),
        'w' => super::write::arm(ed, isglobal),
        'W' => super::write_append::arm(ed, isglobal),
        'x' => super::put::arm(ed, isglobal),
        'y' => super::yank::arm(ed, isglobal),
        'z' => super::scroll::arm(ed, isglobal),
        '!' => super::bang::arm(ed, isglobal),
        '=' => super::exec_misc::arm_equal(ed),
        '\n' => super::null::arm(ed, isglobal),
        '#' => super::exec_misc::arm_comment(ed),
        _ => Err(CommandError::UnknownCommand),
    }
}
