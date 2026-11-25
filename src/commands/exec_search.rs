//! Address extraction for the dispatcher (main_loop.c extract_addresses),
//! including `/re/` and `?re?` searches.
//!
//! The address grammar itself is `address::parse_addresses_with`. This file
//! plugs the regex search into it and maps its errors to GNU ed text.

#![allow(dead_code)]

use super::editor::Editor;
use super::CommandError;
use crate::address::AddressError;

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
    let _ = ed;
    todo!("port main_loop.c extract_addresses")
}

/// GNU ed text for an address error: `InvalidAddress` as
/// `CommandError::InvalidAddress`, `InvalidMark` as
/// `Message("Invalid mark character")`, `NumberOutOfRange` as
/// `Message("Number out of range")`, and `Search(text)` as `Message(text)`.
pub fn address_error(err: AddressError) -> CommandError {
    let _ = err;
    todo!("map AddressError to GNU ed text")
}
