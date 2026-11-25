//! The read-and-run loop (main_loop.c main_loop).

#![allow(dead_code)]

use super::editor::Editor;

/// Ports main_loop.c `main_loop` without the signal handling.
///
/// Loop: flush stdout and stderr; after a failed command with
/// `ed.verbose` print `ed.errmsg`; when `ed.prompt_on` print
/// `ed.prompt_str`; read a line with `io::stdin_line::get_stdin_line`
/// into `ed.cmd` (pos 0). A read error returns 2. At end of input act like
/// q (EMOD when modified and not already warned, which sets exit status 2
/// unless `loose`). Otherwise run `exec::exec_command(ed, false)`.
///
/// After Ok: if `ed.read_only` and modified, clear it and print
/// "<def_filename>: warning: read-only file" to stderr. After Quit return
/// the exit status. After any failure print "?" and a newline, set the
/// exit status to 1 unless `loose` (or already set), set `ed.warned` to
/// whether it was EMOD and if so set `ed.errmsg` to "Warning: buffer
/// modified". If stdin is not a terminal (C interactive() false; use
/// `std::io::IsTerminal`), print "script, line N: errmsg" when verbose and
/// return (1 for Fatal, else the exit status). On Fatal in a terminal
/// print errmsg when verbose and return 1.
///
/// `initial_error` starts the exit status at 1 (a bad file on the command
/// line).
pub fn main_loop(ed: &mut Editor, initial_error: bool, loose: bool) -> i32 {
    let _ = (ed, initial_error, loose);
    todo!("port main_loop.c main_loop")
}
