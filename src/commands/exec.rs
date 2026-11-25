//! The command dispatcher (main_loop.c exec_command).
//!
//! Parse addresses, read the command letter, and hand off to the arm for
//! that command. Each arm lives in its own file and returns an
//! [`ArmResult`](super::ArmResult).

#![allow(dead_code)]

use super::editor::Editor;
use super::Status;

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
    let _ = (ed, isglobal);
    todo!("port main_loop.c exec_command")
}
