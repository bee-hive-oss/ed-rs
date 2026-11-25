//! Editor state shared by the command loop and the command arms.
//!
//! GNU ed keeps this in file-level statics in main_loop.c, buffer.c and
//! regex.c. Here it is one struct that the loop passes to each arm. Types
//! and plain accessors only: no GNU ed logic lives in this file.

#![allow(dead_code)]

use super::active::ActiveList;
use super::{CommandError, PFlags};
use crate::buffer::Buffer;
use crate::buffer::BufferError;
use crate::cli::Options;
use crate::io::read::BinaryState;
use crate::regex::state::RegexState;
use std::io::BufRead;

pub struct Editor {
    pub buffer: Buffer,
    /// Cut buffer filled by y, d, c, j and read by x (C yank buffer).
    pub cut: Vec<String>,
    /// Lines a g, v, G or V command still has to visit (global.c).
    pub active: ActiveList,
    pub regex: RegexState,
    pub opts: Options,
    /// Default filename (C def_filename).
    pub def_filename: String,
    /// Last error message, shown by h and H (C errmsg).
    pub errmsg: String,
    pub prompt_str: String,
    pub prompt_on: bool,
    pub verbose: bool,
    pub read_only: bool,
    /// Binary-file state (C isbinary_ and unterminated_line).
    pub bin: BinaryState,
    /// `bin.unterminated` as it was at the last [`Editor::snapshot`], so
    /// undo brings it back with the lines (C keeps it on the line node).
    pub undo_unterminated: bool,
    /// Set after a warning about a modified buffer (C warned()).
    pub warned: bool,
    /// Last shell command, for `!!` (C shcmd).
    pub shell_cmd: String,
    /// Scroll size set by z (C user_lines). 0 means use the default 22.
    pub user_lines: usize,
    /// Current command buffer, always ending in a newline (C ibuf).
    pub cmd: String,
    /// Cursor into `cmd` (C ibufp), a byte offset.
    pub pos: usize,
    /// Address count and range of the command being run (C addr_cnt,
    /// first_addr, second_addr).
    pub addr_cnt: usize,
    pub first: usize,
    pub second: usize,
    /// Print suffixes collected by the running command.
    pub pflags: PFlags,
    /// Memory of the last s command (C static pflags, pmask, snum).
    pub s_pflags: PFlags,
    pub s_pmask: PFlags,
    pub s_snum: i32,
    /// Script line counter (C linenum).
    pub linenum: usize,
    pub stdin: Box<dyn BufRead>,
}

impl Editor {
    pub fn new(opts: Options, stdin: Box<dyn BufRead>) -> Self {
        let prompt_on = opts.prompt.is_some();
        let prompt_str = opts.prompt.clone().unwrap_or_else(|| "*".to_string());
        let verbose = opts.verbose;
        Editor {
            buffer: Buffer::new(),
            cut: Vec::new(),
            active: ActiveList::default(),
            regex: RegexState::default(),
            opts,
            def_filename: String::new(),
            errmsg: String::new(),
            prompt_str,
            prompt_on,
            verbose,
            read_only: false,
            bin: BinaryState::default(),
            undo_unterminated: false,
            warned: false,
            shell_cmd: String::new(),
            user_lines: 0,
            cmd: String::new(),
            pos: 0,
            addr_cnt: 0,
            first: 0,
            second: 0,
            pflags: PFlags::default(),
            s_pflags: PFlags::default(),
            s_pmask: PFlags {
                l: false,
                n: false,
                p: true,
            },
            s_snum: 1,
            linenum: 0,
            stdin,
        }
    }

    /// Take the undo snapshot before a change (C clear_undo_stack): the
    /// buffer's snapshot plus `bin.unterminated`.
    pub fn snapshot(&mut self) {
        self.buffer.snapshot();
        self.undo_unterminated = self.bin.unterminated;
    }

    /// Undo or redo (C undo): swap the buffer with its snapshot and swap
    /// `bin.unterminated` with the saved value.
    pub fn restore(&mut self) -> Result<(), BufferError> {
        self.buffer.restore()?;
        std::mem::swap(&mut self.bin.unterminated, &mut self.undo_unterminated);
        Ok(())
    }

    /// Unread part of the command buffer.
    pub fn rest(&self) -> &str {
        &self.cmd[self.pos..]
    }

    /// Next unread character, if any.
    pub fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    /// Take the next unread character and move past it.
    pub fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    /// Record `err` as the last error message and return Status::Err.
    pub fn fail(&mut self, err: impl Into<CommandError>) -> super::Status {
        self.errmsg = err.into().message().to_string();
        super::Status::Err
    }

    /// Read the next script line from stdin, with its trailing newline.
    /// None on end of input or a read error. Uses
    /// `io::stdin_line::get_stdin_line`.
    pub fn next_line(&mut self) -> Option<String> {
        crate::io::stdin_line::get_stdin_line(&mut *self.stdin, &mut self.linenum)
            .ok()
            .flatten()
    }

    /// Scroll size for z (C window_lines): `user_lines` when set, else 22.
    pub fn window_lines(&self) -> usize {
        if self.user_lines > 0 {
            self.user_lines
        } else {
            22
        }
    }

    /// Column width for list mode (C window_columns).
    pub fn window_columns(&self) -> usize {
        76
    }
}

/// Line source for a, i and c. In a global command it takes the next lines
/// of the command list in `cmd` starting at `pos`; otherwise it reads stdin.
/// Items have no trailing newline. Built from separate fields so the caller
/// can still borrow `Editor::buffer` while it runs.
pub struct InputLines<'a> {
    pub isglobal: bool,
    pub cmd: &'a str,
    pub pos: &'a mut usize,
    pub stdin: &'a mut dyn BufRead,
    pub linenum: &'a mut usize,
}

impl Iterator for InputLines<'_> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.isglobal {
            let rest = &self.cmd[*self.pos..];
            if rest.is_empty() {
                return None;
            }
            let len = rest.find('\n').map_or(rest.len(), |i| i + 1);
            let line = rest[..len].trim_end_matches('\n').to_string();
            *self.pos += len;
            Some(line)
        } else {
            let line = crate::io::stdin_line::get_stdin_line(self.stdin, self.linenum)
                .ok()
                .flatten()?;
            Some(line.trim_end_matches('\n').to_string())
        }
    }
}
