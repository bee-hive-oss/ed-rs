//! The read-and-run loop (main_loop.c main_loop).

#![allow(dead_code)]

use super::editor::Editor;
use super::Status;
use std::io::{self, IsTerminal, Write};

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
    let mut err_status = i32::from(initial_error);
    let mut status = err_status;
    loop {
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();
        if status < 0 && ed.verbose {
            println!("{}", ed.errmsg);
            let _ = io::stdout().flush();
        }
        if ed.prompt_on {
            print!("{}", ed.prompt_str);
            let _ = io::stdout().flush();
        }
        let line = match crate::io::stdin_line::get_stdin_line(&mut *ed.stdin, &mut ed.linenum) {
            Err(_) => return 2,
            Ok(line) => line,
        };
        status = match line {
            Some(line) => {
                ed.cmd = line;
                ed.pos = 0;
                super::exec::exec_command(ed, false) as i32
            }
            None => end_of_input(ed, loose, &mut err_status),
        };
        if status == Status::Ok as i32 {
            note_readonly(ed);
            continue;
        }
        if status == Status::Quit as i32 {
            return err_status;
        }
        println!("?");
        if !loose && err_status == 0 {
            err_status = 1;
        }
        ed.warned = status == Status::Emod as i32;
        if ed.warned {
            ed.errmsg = "Warning: buffer modified".to_string();
        }
        if !io::stdin().is_terminal() {
            if ed.verbose {
                println!("script, line {}: {}", ed.linenum, ed.errmsg);
            }
            return if status == Status::Fatal as i32 { 1 } else { err_status };
        }
        if status == Status::Fatal as i32 {
            if ed.verbose {
                println!("{}", ed.errmsg);
            }
            return 1;
        }
    }
}

fn end_of_input(ed: &Editor, loose: bool, err_status: &mut i32) -> i32 {
    let status = super::quit::quit(false, ed.buffer.is_modified(), ed.warned);
    if status == Status::Emod && !loose {
        *err_status = 2;
    }
    status as i32
}

fn note_readonly(ed: &mut Editor) {
    if ed.read_only && ed.buffer.is_modified() {
        ed.read_only = false;
        eprintln!("{}: warning: read-only file", ed.def_filename);
    }
}
