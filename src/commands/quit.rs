//! Quit the editor (the q and Q commands).
//!
//! Ports the q and Q arms of GNU ed 1.22.4 main_loop.c exec_command.
//! The P command shares that switch arm in C but is not part of this
//! function: P only toggles the prompt and does not quit.
//!
//! Address and suffix checks stay in the dispatcher (q and Q reject an
//! address with "Unexpected address").

#![allow(dead_code)]

use super::Status;

/// Decide whether q or Q should quit, or refuse because the buffer is
/// dirty.
///
/// Matches `return ( c == 'q' && modified() && !warned() ) ? EMOD : QUIT`.
/// `force` is true for Q (always Quit) and false for q. The command loop,
/// not this function, prints the warning and sets the warned flag after
/// Emod, so a second q can quit.
pub fn quit(force: bool, modified: bool, warned: bool) -> Status {
    if !force && modified && !warned {
        Status::Emod
    } else {
        Status::Quit
    }
}
