//! Command parser, dispatcher, and small command stubs.
//!
//! `run` hands off to `startup::start`, which still returns 1 until its
//! ticket lands, so the CLI tests that already pass on main stay passing.
//! The command loop (`main_loop`), the dispatcher (`exec`), and one arm file
//! per command are stubs that their own tickets fill in.
//!
//! Shared result types live here so each command file can fill in a body
//! without also inventing error types. Status values match GNU ed 1.22.4
//! main_loop.c: enum Status { QUIT = -1, ERR = -2, EMOD = -3, FATAL = -4 }.

#![allow(dead_code)]

use crate::cli::Options;

pub mod active;
pub mod append;
pub mod args;
pub mod bang;
pub mod change;
pub mod copy;
pub mod delete;
pub mod edit;
pub mod edit_e;
pub mod editor;
pub mod equal;
pub mod exec;
pub mod exec_edit;
pub mod exec_misc;
pub mod exec_print;
pub mod exec_search;
pub mod filename;
pub mod global;
pub mod global_interactive;
pub mod global_interactive_v;
pub mod global_v;
pub mod help;
pub mod help_verbose;
pub mod insert;
pub mod join;
pub mod list;
pub mod main_loop;
pub mod mark;
pub mod move_lines;
pub mod null;
pub mod number;
pub mod print;
pub mod print_any;
pub mod prompt;
pub mod put;
pub mod quit;
pub mod range;
pub mod read;
pub mod scroll;
pub mod startup;
pub mod subst_repeat;
pub mod subst_run;
pub mod subst_suffix;
pub mod substitute;
pub mod undo;
pub mod write;
pub mod write_append;
pub mod yank;

/// Enter the editor. The real start-up lives in `startup::start`, which
/// still returns 1 until its ticket lands, so behavior on main is unchanged.
pub fn run(opts: Options) -> i32 {
    startup::start(opts)
}

/// What a command arm returns: a status for the loop, or a recoverable
/// error whose text goes to `Editor::errmsg`.
pub type ArmResult = Result<Status, CommandError>;

/// Process / command status from GNU ed 1.22.4 main_loop.c.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    /// Command succeeded (C return 0).
    Ok = 0,
    /// Quit the editor (C QUIT).
    Quit = -1,
    /// Command failed; stay in the loop (C ERR).
    Err = -2,
    /// Quit or edit refused because the buffer is modified and the user
    /// has not been warned yet (C EMOD).
    Emod = -3,
    /// Fatal error (C FATAL).
    Fatal = -4,
}

/// Recoverable command failure, with the GNU ed 1.22.4 diagnostic text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    InvalidAddress,
    UnexpectedAddress,
    UnexpectedCommandSuffix,
    InvalidCommandSuffix,
    InvalidRedirection,
    NoCurrentFilename,
    FilenameTooLong,
    UnknownCommand,
    /// Writing command output failed. GNU ed 1.22.4 does not check stdout
    /// writes in print_lines, so this text is ed-rs's own, not GNU's.
    Io(std::io::ErrorKind),
    /// Any other GNU ed 1.22.4 diagnostic, with its exact text, for example
    /// "No match" or "Nothing to put".
    Message(&'static str),
}

impl CommandError {
    /// Exact GNU ed 1.22.4 set_error_msg / constant text for this error.
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidAddress => "Invalid address",
            Self::UnexpectedAddress => "Unexpected address",
            Self::UnexpectedCommandSuffix => "Unexpected command suffix",
            Self::InvalidCommandSuffix => "Invalid command suffix",
            Self::InvalidRedirection => "Invalid redirection",
            Self::NoCurrentFilename => "No current filename",
            Self::FilenameTooLong => "Filename too long",
            Self::UnknownCommand => "Unknown command",
            Self::Io(_) => "Cannot write output",
            Self::Message(text) => text,
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for CommandError {}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.kind())
    }
}

impl From<crate::regex::RegexError> for CommandError {
    fn from(err: crate::regex::RegexError) -> Self {
        Self::Message(err.message())
    }
}

/// Map a buffer error to the matching GNU ed diagnostic.
pub fn buf_err(err: crate::buffer::BufferError) -> CommandError {
    use crate::buffer::BufferError;
    match err {
        BufferError::InvalidAddress => CommandError::InvalidAddress,
        BufferError::InvalidMark => CommandError::Message("Invalid mark character"),
        BufferError::InvalidDestination => CommandError::Message("Invalid destination"),
        BufferError::NothingToUndo => CommandError::Message("Nothing to undo"),
    }
}

/// Print suffixes l, n, p (GNU ed Pflags pf_l, pf_n, pf_p).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PFlags {
    pub l: bool,
    pub n: bool,
    pub p: bool,
}

impl PFlags {
    /// True when no suffix is set.
    pub fn is_empty(&self) -> bool {
        !(self.l || self.n || self.p)
    }

    /// Bitwise xor, like `pflags ^= pmask` in command_s.
    pub fn xor(self, other: PFlags) -> PFlags {
        PFlags {
            l: self.l ^ other.l,
            n: self.n ^ other.n,
            p: self.p ^ other.p,
        }
    }
}
