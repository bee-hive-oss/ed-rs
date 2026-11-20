//! Command parser, dispatcher, and small command stubs.
//!
//! `run` still returns 1 and does not call the stub modules, so the CLI
//! tests that already pass on main stay passing. A later command-loop
//! ticket will wire these functions in.
//!
//! Shared result types live here so each command file can fill in a body
//! without also inventing error types. Status values match GNU ed 1.22.4
//! main_loop.c: enum Status { QUIT = -1, ERR = -2, EMOD = -3, FATAL = -4 }.

#![allow(dead_code)]

use crate::cli::Options;

pub mod append;
pub mod delete;
pub mod equal;
pub mod filename;
pub mod print;
pub mod quit;

pub fn run(_opts: Options) -> i32 {
    1
}

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
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for CommandError {}
