//! Address search for `/re/` and `?re?` (regex.c next_matching_node_addr).

#![allow(dead_code)]

use super::state::RegexState;
use crate::buffer::Buffer;
use crate::commands::CommandError;

/// Ports regex.c `next_matching_node_addr`.
///
/// `input` starts at the `/` or `?`. Use `RegexState::get_compiled_regex`
/// to get the regex and the bytes used. Search from the current line in the
/// given direction (`/` forward, `?` backward), one line at a time, and wrap
/// around the end or start of the buffer. Address 0 is skipped. Stop after
/// one full lap back at the current line. Match with `Regex::is_match` on each
/// line's text.
///
/// Return the matching line number and the bytes used. No match is
/// `Message("No match")`.
pub fn next_matching_node_addr(
    buffer: &Buffer,
    state: &mut RegexState,
    input: &str,
    extended: bool,
) -> Result<(usize, usize), CommandError> {
    let _ = (buffer, state, input, extended);
    todo!("port regex.c next_matching_node_addr")
}
