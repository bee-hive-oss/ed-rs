//! Address extraction for the dispatcher (main_loop.c extract_addresses),
//! including `/re/` and `?re?` searches.
//!
//! The address grammar itself is `address::parse_addresses_with`. This file
//! plugs the regex search into it and maps its errors to GNU ed text.

#![allow(dead_code)]

use super::editor::Editor;
use super::CommandError;
use crate::address::{self, AddressError};

/// Ports main_loop.c `extract_addresses`.
///
/// Call `address::parse_addresses_with` on `ed.cmd[ed.pos..]` with
/// `&mut ed.buffer`, passing a search hook that calls
/// `regex::search::next_matching_node_addr` with `&mut ed.regex` and
/// `ed.opts.extended_regexp`. Map a search error into
/// `AddressError::Search` with its message. On success store the count and
/// the two addresses in `ed.addr_cnt`, `ed.first`, `ed.second`, move
/// `ed.pos` past what was parsed, and return the count. Map errors with
/// [`address_error`].
pub fn extract_addresses(ed: &mut Editor) -> Result<usize, CommandError> {
    let extended = ed.opts.extended_regexp;
    let start = ed.pos;
    let input = ed.cmd[start..].to_string();
    let parsed = {
        let Editor { buffer, regex, .. } = ed;
        address::parse_addresses_with(buffer, &input, &mut |buf, text| {
            crate::regex::search::next_matching_node_addr(buf, regex, text, extended)
                .map_err(|err| AddressError::Search(err.message()))
        })
    }
    .map_err(address_error)?;
    ed.addr_cnt = parsed.count;
    ed.first = parsed.first;
    ed.second = parsed.second;
    ed.pos = start + input.len() - parsed.rest.len();
    Ok(parsed.count)
}

/// GNU ed text for an address error: `InvalidAddress` as
/// `CommandError::InvalidAddress`, `InvalidMark` as
/// `Message("Invalid mark character")`, `NumberOutOfRange` as
/// `Message("Number out of range")`, and `Search(text)` as `Message(text)`.
pub fn address_error(err: AddressError) -> CommandError {
    match err {
        AddressError::InvalidAddress => CommandError::InvalidAddress,
        AddressError::InvalidMark => CommandError::Message("Invalid mark character"),
        AddressError::NumberOutOfRange => CommandError::Message("Number out of range"),
        AddressError::Search(text) => CommandError::Message(text),
        AddressError::RegexNotSupported => {
            CommandError::Message("Regular expression addresses are not implemented")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::extract_addresses;
    use super::super::editor::Editor;
    use crate::cli::Options;
    use crate::commands::CommandError::Message;

    fn ed(lines: &[&str], cmd: &str) -> Editor {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(lines.iter().copied());
        e.cmd = cmd.into();
        e
    }

    #[test]
    fn addresses_and_search() {
        let mut e = ed(&["foo", "bar", "baz"], "2,3p\n");
        assert_eq!(extract_addresses(&mut e).unwrap(), 2);
        assert_eq!((e.first, e.second), (2, 3));
        assert_eq!(&e.cmd[e.pos..], "p\n");
        let mut e = ed(&["foo", "bar"], "/foo/\n");
        assert_eq!(extract_addresses(&mut e).unwrap(), 1);
        assert_eq!(e.second, 1);
        let mut e = ed(&["foo"], "/no/\n");
        assert_eq!(extract_addresses(&mut e).unwrap_err(), Message("No match"));
        assert_eq!(extract_addresses(&mut ed(&[], "'\n")).unwrap_err(), Message("Invalid mark character"));
    }
}
