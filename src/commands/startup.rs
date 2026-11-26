//! Start-up after option parsing (the end of main.c main, and
//! main_loop.c first_e_command).

#![allow(dead_code)]

use super::editor::Editor;
use crate::cli::{Options, StartRegexp};
use std::io::{self, BufRead, BufReader};
use std::os::fd::AsFd;

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
/// `ed.read_only`. A failed read is exit 2 when stdin is a regular file.
/// When it is not, "Cannot open input file" is not an initial error; a
/// read error is. Then apply `opts.start_line` (when it is
/// within the buffer) or `opts.start_regexp` (search from address 0 with
/// `regex::search::next_matching_node_addr`; on no match use line 1 for
/// `/` or `$` for `?`, print "<re>: <file>: No match found." to stderr
/// unless quiet, and return 1 when not interactive). Print an initial
/// error as a warning ("<file>: <errmsg>") to stderr. Finally return
/// `main_loop::main_loop(&mut ed, initial_error, opts.loose_exit)`.
pub fn start(opts: Options) -> i32 {
    let mut ed = Editor::new(opts, stdin_reader());
    let mut initial_error = false;
    if let Some(file) = ed.opts.file.clone() {
        match load_file(&mut ed, &file) {
            Boot::Exit(code) => return code,
            Boot::Initial => {
                initial_error = true;
                if !ed.opts.quiet {
                    eprintln!("{file}: {}", ed.errmsg);
                }
            }
            Boot::Ok => {}
        }
    }
    let loose = ed.opts.loose_exit;
    super::main_loop::main_loop(&mut ed, initial_error, loose)
}

enum Boot {
    Ok,
    Initial,
    Exit(i32),
}

fn interactive() -> bool {
    !stdin_is_regular_file()
}

/// True when stdin is a regular file. False for a pipe or terminal, and
/// false when the fd cannot be cloned or stated.
fn stdin_is_regular_file() -> bool {
    io::stdin()
        .as_fd()
        .try_clone_to_owned()
        .ok()
        .and_then(|fd| std::fs::File::from(fd).metadata().ok())
        .is_some_and(|meta| meta.is_file())
}

fn load_file(ed: &mut Editor, file: &str) -> Boot {
    if let Err(err) = crate::io::getname::may_access_filename(file, ed.opts.restricted, ed.opts.unsafe_names) {
        ed.errmsg = err.message().to_string();
        return if interactive() { Boot::Initial } else { Boot::Exit(2) };
    }
    if !file.starts_with('!') {
        ed.def_filename = file.to_string();
        ed.read_only = false;
    }
    let (strip, silent, extended, quiet) = (
        ed.opts.strip_trailing_cr,
        ed.opts.silent,
        ed.opts.extended_regexp,
        ed.opts.quiet,
    );
    let start_line = ed.opts.start_line;
    let start_re = ed.opts.start_regexp.clone();
    let read = {
        let Editor { buffer, bin, .. } = &mut *ed;
        crate::io::read::read_file(buffer, bin, file, 0, strip, silent, &mut io::stdout())
    };
    let out = match read {
        Ok(out) => out,
        Err(err) => {
            ed.errmsg = err.message().to_string();
            if !interactive() {
                return Boot::Exit(2);
            }
            // GNU read_file -1 is not an initial error; -2 is.
            return if err.message() == "Cannot open input file" {
                Boot::Ok
            } else {
                Boot::Initial
            };
        }
    };
    ed.read_only = out.read_only;
    if let Some(n) = start_line {
        let n = n as usize;
        if (1..=ed.buffer.len()).contains(&n) {
            let _ = ed.buffer.set_current(n);
        }
    } else if let Some(re) = start_re {
        return apply_re(ed, &re, file, extended, quiet);
    }
    Boot::Ok
}

fn apply_re(ed: &mut Editor, re: &StartRegexp, file: &str, extended: bool, quiet: bool) -> Boot {
    let _ = ed.buffer.set_current(0);
    let (forward, pat) = match re {
        StartRegexp::Forward(pat) => (true, pat.as_str()),
        StartRegexp::Backward(pat) => (false, pat.as_str()),
    };
    let text = if forward { format!("/{pat}") } else { format!("?{pat}") };
    let found = crate::regex::search::next_matching_node_addr(&ed.buffer, &mut ed.regex, &text, extended);
    if let Ok((addr, _)) = found {
        if (1..=ed.buffer.len()).contains(&addr) {
            let _ = ed.buffer.set_current(addr);
            return Boot::Ok;
        }
    }
    let fallback = if forward { 1.min(ed.buffer.len()) } else { ed.buffer.len() };
    let _ = ed.buffer.set_current(fallback);
    if !quiet {
        let shown = if forward { format!("+/{pat}") } else { format!("+?{pat}") };
        eprintln!("{shown}: {file}: No match found.");
    }
    if interactive() { Boot::Ok } else { Boot::Exit(1) }
}

/// Stdin for the editor. Capacity 1 so a shell command can read the next
/// script line (C `setvbuf` with `_IONBF`). Falls back to a normal reader
/// when the stdin fd cannot be cloned.
fn stdin_reader() -> Box<dyn BufRead> {
    match io::stdin().as_fd().try_clone_to_owned() {
        Ok(fd) => Box::new(BufReader::with_capacity(1, std::fs::File::from(fd))),
        Err(_) => Box::new(BufReader::new(io::stdin())),
    }
}
