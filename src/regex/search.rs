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
    let forward = input.starts_with('/');
    let (re, used) = state.get_compiled_regex(input, extended)?;
    let start = buffer.current();
    let last = buffer.len();
    let mut addr = start;
    loop {
        addr = if forward {
            if addr >= last { 0 } else { addr + 1 }
        } else if addr == 0 {
            last
        } else {
            addr - 1
        };
        if addr != 0 && buffer.line(addr).is_ok_and(|text| re.is_match(text)) {
            return Ok((addr, used));
        }
        if addr == start {
            break;
        }
    }
    Err(CommandError::Message("No match"))
}

#[cfg(test)]
mod tests {
    use super::next_matching_node_addr;
    use super::super::state::RegexState;
    use crate::buffer::Buffer;
    use crate::commands::CommandError::Message;

    #[test]
    fn wraps_either_way() {
        let mut buf = Buffer::new();
        buf.load(["foo", "bar", "baz"]);
        buf.set_current(1).unwrap();
        let mut st = RegexState::default();
        assert_eq!(next_matching_node_addr(&buf, &mut st, "/bar/", false).unwrap(), (2, 5));
        assert_eq!(next_matching_node_addr(&buf, &mut st, "/foo/", false).unwrap(), (1, 5));
        buf.set_current(2).unwrap();
        assert_eq!(next_matching_node_addr(&buf, &mut st, "/bar/", false).unwrap(), (2, 5));
        assert_eq!(next_matching_node_addr(&buf, &mut st, "/no/", false).unwrap_err(), Message("No match"));
        buf.set_current(1).unwrap();
        assert_eq!(next_matching_node_addr(&buf, &mut st, "?baz?", false).unwrap(), (3, 5));
        let empty = Buffer::new();
        assert_eq!(next_matching_node_addr(&empty, &mut st, "/foo/", false).unwrap_err(), Message("No match"));
    }
}
