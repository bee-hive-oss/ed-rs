//! Start-up after option parsing (the end of main.c main, and
//! main_loop.c first_e_command).

#![allow(dead_code)]

use crate::cli::Options;

/// Build the editor and run the loop. Returns the process exit status.
///
/// Until its ticket lands this returns 1, the same as `run` did before, so
/// nothing that passes on main changes.
///
/// The ticket ports the operand part of main.c main: make an
/// `editor::Editor` from `opts` and a locked stdin. If `opts.file` is set,
/// check it with `io::getname::may_access_filename`; set the default
/// filename (unless it starts with `!`); read it with
/// `io::read::read_file` at address 0 with `&mut ed.bin` (first_e_command), which sets
/// `ed.read_only`. A failed read is exit 2 when stdin is not a terminal,
/// otherwise an initial error. Then apply `opts.start_line` (when it is
/// within the buffer) or `opts.start_regexp` (search from address 0 with
/// `regex::search::next_matching_node_addr`; on no match use line 1 for
/// `/` or `$` for `?`, print "<re>: <file>: No match found." to stderr
/// unless quiet, and return 1 when not interactive). Print an initial
/// error as a warning ("<file>: <errmsg>") to stderr. Finally return
/// `main_loop::main_loop(&mut ed, initial_error, opts.loose_exit)`.
pub fn start(opts: Options) -> i32 {
    let _ = opts;
    1
}
