//! POSIX/GNU ed address and range parser.
//!
//! [`parse_addresses`] follows GNU ed 1.22.6 `extract_addresses`. It needs
//! `&mut Buffer` because `;` after a parsed address sets the current line
//! before the next address is read. That update is kept even if the rest of
//! the parse fails. Nothing else on the buffer changes: lines, marks, the
//! dirty flag, and the undo snapshot stay as they were.
//!
//! `/re/` and `?re?` go to a search hook when they would start an address.
//! [`parse_addresses_with`] takes that hook; [`parse_addresses`] passes one
//! that rejects them with [`AddressError::RegexNotSupported`].
//! After an address is already in progress, `/` and `?` are
//! [`AddressError::InvalidAddress`], matching GNU ed.
//!
//! The command seat has not called this parser yet. A binary crate treats
//! unused `pub` items as dead code, so the allow stays until that seat lands.

#![allow(dead_code)]

use crate::buffer::{Buffer, BufferError};
use std::fmt;

/// Failure while parsing an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
    /// The finished address is outside `0..=last`, or a mark is unset.
    InvalidAddress,
    /// `'x` where `x` is missing or not `a`–`z`.
    InvalidMark,
    /// A decimal literal does not fit in a signed 32-bit integer (GNU ed's `int`).
    NumberOutOfRange,
    /// `/re/` or `?re?` appeared where a base address is parsed.
    ///
    /// `src/regex.rs` does not exist yet as a real matcher. This is the hook
    /// that seat will replace.
    RegexNotSupported,
    /// The search hook failed, with GNU ed's text (for example "No match").
    Search(&'static str),
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidAddress => "Invalid address",
            Self::InvalidMark => "Invalid mark character",
            Self::NumberOutOfRange => "Number out of range",
            Self::RegexNotSupported => "Regular expression addresses are not implemented",
            Self::Search(text) => text,
        })
    }
}

impl std::error::Error for AddressError {}

/// Addresses parsed from the front of a command line.
///
/// `count` matches GNU ed 1.22.6 `extract_addresses`:
/// - `0`: no address was written. `first` and `second` are still both set to
///   the current line, and the count stays 0 so the command can apply its own
///   default (for example `w` uses `1,$` while `p` uses `.`).
/// - `1`: one address. `first == second` is that address.
/// - `2`: a range produced by `,`, `;`, or `%`.
///
/// `rest` is the unconsumed tail, starting at the command letter. Spaces and
/// tabs that trailed the address are consumed.
///
/// Out-of-range values are rejected when an address is finished (end of the
/// address, or a separator after a parsed address). Intermediate offsets may
/// leave `0..=last` (`1+100-100` is line 1). A reversed range (`5,4`) is
/// returned as count 2; the command decides whether that is an error.
/// Omitted addresses that invent line 1 (` , ` on an empty buffer is `1,0`)
/// are also returned rather than rejected here, because GNU checks them in
/// the command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAddresses<'a> {
    pub count: usize,
    pub first: usize,
    pub second: usize,
    pub rest: &'a str,
}

/// Parse ed addresses from `input` against `buffer`.
///
/// See [`ParsedAddresses`] for the count rules. `;` after a real address sets
/// the current line to that address before the second address is parsed.
/// `;` or `,` or `%` with the address before it omitted does not move `.`:
/// the omitted first address is already `.` (for `;`) or `1` (for `,` and `%`).
///
/// `%` is the same separator as `,`. With no address on either side it is
/// `1,$`. A following offset is a new address based on `.`, not an offset of
/// `$`, because the separator returns the parser to "start of address".
/// `%-1` with `.` equal to 1 is the range `1,0`, not `1,$-1`. When `.` is `$`,
/// the two readings coincide.
///
/// Whitespace: a leading run of blanks (space, tab, CR, VT, FF) is skipped.
/// Inside an address, only space and tab open a blank run. A `+` or `-`
/// binds a number only when a digit is the next byte; otherwise it means ±1.
/// A bare number after an address is a further positive offset, so `1 + 2`
/// is `1 + 1 + 2` (line 4 when those lines exist) and `1 +2` is `1+2`.
pub fn parse_addresses<'a>(
    buffer: &mut Buffer,
    input: &'a str,
) -> Result<ParsedAddresses<'a>, AddressError> {
    parse_addresses_with(buffer, input, &mut |_, _| {
        Err(AddressError::RegexNotSupported)
    })
}

/// Search hook for `/re/` and `?re?`. It gets the buffer and the input
/// starting at the `/` or `?`, and returns the matching line and how many
/// bytes it used (pattern, closing delimiter, and any `I` suffix).
pub type SearchHook<'h> = &'h mut dyn FnMut(&Buffer, &str) -> Result<(usize, usize), AddressError>;

/// [`parse_addresses`] with a search hook for `/re/` and `?re?` (GNU ed
/// 1.22.6 extract_addresses calling next_matching_node_addr).
pub fn parse_addresses_with<'a>(
    buffer: &mut Buffer,
    input: &'a str,
    search: SearchHook<'_>,
) -> Result<ParsedAddresses<'a>, AddressError> {
    let bytes = input.as_bytes();
    let mut idx = skip_blanks(bytes, 0);
    // `first_mode` means the next token starts an address. After a base
    // address it is false and further numbers or ± are offsets.
    // `-1` is "not set yet". A computed address of -1 is rejected by the
    // range check before it can be confused with "not set".
    let mut first_mode = true;
    let mut first_addr: i64 = -1;
    let mut second_addr: i64 = -1;

    loop {
        if idx >= bytes.len() {
            return finish(buffer, input, idx, first_mode, first_addr, second_addr);
        }
        let ch = bytes[idx];
        if ch.is_ascii_digit() {
            let (number, next) = parse_number(bytes, idx)?;
            idx = next;
            if first_mode {
                first_mode = false;
                second_addr = number;
            } else {
                second_addr = second_addr
                    .checked_add(number)
                    .ok_or(AddressError::NumberOutOfRange)?;
            }
            continue;
        }
        match ch {
            b' ' | b'\t' => {
                idx += 1;
                idx = skip_blanks(bytes, idx);
            }
            b'+' | b'-' => {
                if first_mode {
                    first_mode = false;
                    second_addr = buffer.current() as i64;
                }
                let signed = idx + 1 < bytes.len() && bytes[idx + 1].is_ascii_digit();
                if signed {
                    let (number, next) = parse_number(bytes, idx)?;
                    idx = next;
                    second_addr = second_addr
                        .checked_add(number)
                        .ok_or(AddressError::NumberOutOfRange)?;
                } else {
                    idx += 1;
                    second_addr = if ch == b'+' {
                        second_addr.checked_add(1)
                    } else {
                        second_addr.checked_sub(1)
                    }
                    .ok_or(AddressError::NumberOutOfRange)?;
                }
            }
            b'.' | b'$' => {
                if !first_mode {
                    return Err(AddressError::InvalidAddress);
                }
                first_mode = false;
                idx += 1;
                second_addr = if ch == b'.' {
                    buffer.current() as i64
                } else {
                    buffer.len() as i64
                };
            }
            b'/' | b'?' => {
                if !first_mode {
                    return Err(AddressError::InvalidAddress);
                }
                let (addr, used) = search(buffer, &input[idx..])?;
                first_mode = false;
                idx += used;
                second_addr = addr as i64;
            }
            b'\'' => {
                if !first_mode {
                    return Err(AddressError::InvalidAddress);
                }
                first_mode = false;
                idx += 1;
                let mark = bytes.get(idx).copied().unwrap_or(0);
                if idx < bytes.len() {
                    idx += 1;
                }
                second_addr = lookup_mark(buffer, mark as char)?;
            }
            b'%' | b',' | b';' => {
                if first_mode {
                    // The address before the separator was omitted.
                    if first_addr < 0 {
                        first_addr = if ch == b';' {
                            buffer.current() as i64
                        } else {
                            1
                        };
                        second_addr = buffer.len() as i64;
                    } else {
                        first_addr = second_addr;
                    }
                } else {
                    if second_addr < 0 || second_addr > buffer.len() as i64 {
                        return Err(AddressError::InvalidAddress);
                    }
                    // Only a parsed address followed by `;` moves `.`.
                    if ch == b';' {
                        buffer
                            .set_current(second_addr as usize)
                            .map_err(|_| AddressError::InvalidAddress)?;
                    }
                    first_addr = second_addr;
                    first_mode = true;
                }
                idx += 1;
            }
            _ => {
                return finish(buffer, input, idx, first_mode, first_addr, second_addr);
            }
        }
    }
}

fn finish<'a>(
    buffer: &Buffer,
    input: &'a str,
    idx: usize,
    first_mode: bool,
    mut first_addr: i64,
    mut second_addr: i64,
) -> Result<ParsedAddresses<'a>, AddressError> {
    if !first_mode && (second_addr < 0 || second_addr > buffer.len() as i64) {
        return Err(AddressError::InvalidAddress);
    }
    let addr_cnt = if second_addr >= 0 {
        if first_addr >= 0 {
            2
        } else {
            1
        }
    } else {
        0
    };
    if addr_cnt == 0 {
        second_addr = buffer.current() as i64;
    }
    if addr_cnt <= 1 {
        first_addr = second_addr;
    }
    Ok(ParsedAddresses {
        count: addr_cnt,
        first: first_addr as usize,
        second: second_addr as usize,
        rest: &input[idx..],
    })
}

fn lookup_mark(buffer: &Buffer, mark: char) -> Result<i64, AddressError> {
    match buffer.get_mark(mark) {
        Ok(line) => Ok(line as i64),
        Err(BufferError::InvalidMark) => Err(AddressError::InvalidMark),
        Err(_) => Err(AddressError::InvalidAddress),
    }
}

/// C `isspace` in the C locale, excluding newline. Newline ends a command line
/// and is left for the caller.
fn is_blank(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | 0x0b | 0x0c)
}

fn skip_blanks(bytes: &[u8], mut idx: usize) -> usize {
    while idx < bytes.len() && is_blank(bytes[idx]) {
        idx += 1;
    }
    idx
}

/// Parse a signed decimal at `idx`. The value must fit in `-(2^31-1)..=2^31-1`,
/// which is GNU ed's `strtol` check against `INT_MAX` (`-INT_MIN` is rejected).
fn parse_number(bytes: &[u8], idx: usize) -> Result<(i64, usize), AddressError> {
    let mut pos = idx;
    let mut sign = 1i64;
    if pos < bytes.len() && (bytes[pos] == b'+' || bytes[pos] == b'-') {
        if bytes[pos] == b'-' {
            sign = -1;
        }
        pos += 1;
    }
    if pos >= bytes.len() || !bytes[pos].is_ascii_digit() {
        return Err(AddressError::InvalidAddress);
    }
    let mut value: i64 = 0;
    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
        let digit = i64::from(bytes[pos] - b'0');
        value = value
            .checked_mul(10)
            .and_then(|acc| acc.checked_add(digit))
            .ok_or(AddressError::NumberOutOfRange)?;
        pos += 1;
    }
    let signed = value
        .checked_mul(sign)
        .ok_or(AddressError::NumberOutOfRange)?;
    let int_max = i64::from(i32::MAX);
    if signed > int_max || signed < -int_max {
        return Err(AddressError::NumberOutOfRange);
    }
    Ok((signed, pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf_with(lines: &[&str], current: usize) -> Buffer {
        let mut buf = Buffer::new();
        buf.load(lines.iter().copied());
        buf.set_current(current).unwrap();
        buf
    }

    fn parsed<'a>(buf: &mut Buffer, input: &'a str) -> ParsedAddresses<'a> {
        parse_addresses(buf, input).unwrap()
    }

    fn assert_parse(
        buf: &mut Buffer,
        input: &str,
        count: usize,
        first: usize,
        second: usize,
        rest: &str,
        current: usize,
    ) {
        let got = parsed(buf, input);
        assert_eq!(got.count, count, "count for {input:?}");
        assert_eq!(got.first, first, "first for {input:?}");
        assert_eq!(got.second, second, "second for {input:?}");
        assert_eq!(got.rest, rest, "rest for {input:?}");
        assert_eq!(buf.current(), current, "current after {input:?}");
    }

    #[test]
    fn omitted_one_and_two_addresses() {
        // No address: count stays 0, but both slots are the current line.
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 2);
        assert_parse(&mut buf, "", 0, 2, 2, "", 2);
        assert_parse(&mut buf, "p", 0, 2, 2, "p", 2);
        assert_parse(&mut buf, "   p", 0, 2, 2, "p", 2);

        // One address: first == second, count 1. `.` and `$` and numbers.
        assert_parse(&mut buf, ".", 1, 2, 2, "", 2);
        assert_parse(&mut buf, "$", 1, 5, 5, "", 2);
        assert_parse(&mut buf, "3", 1, 3, 3, "", 2);
        assert_parse(&mut buf, "3p", 1, 3, 3, "p", 2);
        assert_parse(&mut buf, "0002n", 1, 2, 2, "n", 2);
        assert_parse(&mut buf, "0", 1, 0, 0, "", 2);
        assert_parse(&mut buf, "  4\tp", 1, 4, 4, "p", 2);

        // Two addresses from an explicit pair.
        assert_parse(&mut buf, "2,4p", 2, 2, 4, "p", 2);
        assert_parse(&mut buf, "2 , 4 n", 2, 2, 4, "n", 2);
    }

    #[test]
    fn relative_and_chained_offsets() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 2);
        assert_parse(&mut buf, "+", 1, 3, 3, "", 2);
        assert_parse(&mut buf, "-", 1, 1, 1, "", 2);
        assert_parse(&mut buf, "++", 1, 4, 4, "", 2);
        assert_parse(&mut buf, "+2", 1, 4, 4, "", 2);
        assert_parse(&mut buf, "-1", 1, 1, 1, "", 2);
        assert_parse(&mut buf, ".+2", 1, 4, 4, "", 2);
        assert_parse(&mut buf, "$-1", 1, 4, 4, "", 2);
        assert_parse(&mut buf, "1+3", 1, 4, 4, "", 2);
        assert_parse(&mut buf, ".+2-1", 1, 3, 3, "", 2);
    }

    #[test]
    fn twelve_plus_three_is_out_of_range_on_short_buffer() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 1);
        // The arithmetic itself is 15; the finished address is past `$`.
        assert_eq!(
            parse_addresses(&mut buf, "12+3"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(buf.current(), 1);
    }

    /// A sign binds a number only when the next byte is a digit.
    /// Otherwise the sign is ±1, and a later number is its own positive offset.
    /// Spaces and tabs between those tokens are skipped.
    #[test]
    fn sign_binding_and_implicit_positive_offset() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 1);
        // `1 + 2` is 1, then bare +1, then +2 → 4. Not 3.
        assert_parse(&mut buf, "1 + 2=", 1, 4, 4, "=", 1);
        assert_parse(&mut buf, "1+ 2=", 1, 4, 4, "=", 1);
        assert_parse(&mut buf, "1 +2=", 1, 3, 3, "=", 1);
        assert_parse(&mut buf, "3 - 2=", 1, 4, 4, "=", 1);
        assert_parse(&mut buf, "3 -2=", 1, 1, 1, "=", 1);
        assert_parse(&mut buf, ". +2=", 1, 3, 3, "=", 1);
        assert_parse(&mut buf, ". + 2=", 1, 4, 4, "=", 1);
        assert_parse(&mut buf, ". 2=", 1, 3, 3, "=", 1);
        assert_parse(&mut buf, "1 2=", 1, 3, 3, "=", 1);
        assert_parse(&mut buf, "+ +", 1, 3, 3, "", 1);
        assert_parse(&mut buf, "1++", 1, 3, 3, "", 1);
        assert_parse(&mut buf, "++2", 1, 4, 4, "", 1);
        assert_parse(&mut buf, "+0", 1, 1, 1, "", 1);
        assert_parse(&mut buf, "2+0", 1, 2, 2, "", 1);

        // Out of range only when the address is finished, not at each step.
        assert_parse(&mut buf, "1+100-100", 1, 1, 1, "", 1);
        assert_parse(&mut buf, ".+100-98", 1, 3, 3, "", 1);
        let mut top = buf_with(&["a", "b", "c"], 3);
        assert_parse(&mut top, "++--", 1, 3, 3, "", 3);
    }

    #[test]
    fn comma_semicolon_percent_defaults() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 2);
        // `,` with no first address is `1,$`. Current line stays put.
        assert_parse(&mut buf, ",", 2, 1, 5, "", 2);
        assert_parse(&mut buf, ",p", 2, 1, 5, "p", 2);
        assert_parse(&mut buf, ",3", 2, 1, 3, "", 2);
        // `;` with no first address is `.,$` and does not write `.` again.
        assert_parse(&mut buf, ";", 2, 2, 5, "", 2);
        assert_parse(&mut buf, ";3", 2, 2, 3, "", 2);
        // `%` is the same default as `,` when no address was given: `1,$`.
        assert_parse(&mut buf, "%", 2, 1, 5, "", 2);
        assert_parse(&mut buf, "%p", 2, 1, 5, "p", 2);
        assert_parse(&mut buf, "%n", 2, 1, 5, "n", 2);

        // A separator after a real address with nothing after it is still two
        // addresses, both equal to that address. It is not "through `$`".
        // `;` is what moves `.` to 3; `,` does not.
        assert_parse(&mut buf, "3,", 2, 3, 3, "", 2);
        assert_parse(&mut buf, "3;", 2, 3, 3, "", 3);
        buf.set_current(2).unwrap();

        // Extra separators keep the last pair. An omitted slot does not
        // become `$` when an address was already parsed (`1,,3` stays `1,3`).
        assert_parse(&mut buf, "1,2,4", 2, 2, 4, "", 2);
        assert_parse(&mut buf, "1,,3", 2, 1, 3, "", 2);
        assert_parse(&mut buf, ",,", 2, 5, 5, "", 2);
        assert_parse(&mut buf, ";;", 2, 5, 5, "", 2);
        assert_parse(&mut buf, "1,2,3", 2, 2, 3, "", 2);

        // `%` is the comma separator, so `2%5` is `2,5` and `2%` is `2,2`.
        assert_parse(&mut buf, "2%5", 2, 2, 5, "", 2);
        assert_parse(&mut buf, "2%", 2, 2, 2, "", 2);
        assert_parse(&mut buf, "%%", 2, 5, 5, "", 2);
    }

    /// After a separator the parser is back at "start of address", so `+1`
    /// is `.+1` using the current line (which `;` may just have changed),
    /// not an offset of the left-hand address.
    #[test]
    fn semicolon_updates_dot_before_the_second_address() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 1);
        // `3;` sets `.` to 3, then `+1` is that current line plus one.
        assert_parse(&mut buf, "3;+1p", 2, 3, 4, "p", 3);
        assert_eq!(buf.current(), 3);

        buf.set_current(1).unwrap();
        assert_parse(&mut buf, "3;.", 2, 3, 3, "", 3);
        buf.set_current(1).unwrap();
        assert_parse(&mut buf, "4;-1", 2, 4, 3, "", 4);
        buf.set_current(1).unwrap();
        assert_parse(&mut buf, ".;+2", 2, 1, 3, "", 1);

        // Comma does not move `.`, so `3,+1` uses the old current line.
        buf.set_current(1).unwrap();
        assert_parse(&mut buf, "3,+1", 2, 3, 2, "", 1);

        // Chained semicolons: each parsed address followed by `;` updates `.`
        // before the next address. `1;2;3` leaves `.` on 2 and the range 2,3.
        buf.set_current(1).unwrap();
        assert_parse(&mut buf, "1;2;3", 2, 2, 3, "", 2);

        // The range after `;` can be a fresh pair. `.` stays on the earlier address.
        buf.set_current(1).unwrap();
        assert_parse(&mut buf, "4;1,2p", 2, 1, 2, "p", 4);

        // Omitted `;` does not assign `.`. From line 2, `;+1` is `2,3`.
        buf.set_current(2).unwrap();
        assert_parse(&mut buf, ";+1", 2, 2, 3, "", 2);
        assert_parse(&mut buf, ",+1", 2, 1, 3, "", 2);
        assert_parse(&mut buf, ";;", 2, 5, 5, "", 2);

        // `%` follows the comma rule. With `.` == 1, `%-1` is `1,0`, not `1,$`.
        buf.set_current(1).unwrap();
        assert_parse(&mut buf, "%-1", 2, 1, 0, "", 1);
        assert_parse(&mut buf, "%+1", 2, 1, 2, "", 1);
        // When `.` is `$`, `%-1` reads the same as `1,$-1`.
        buf.set_current(5).unwrap();
        assert_parse(&mut buf, "%-1", 2, 1, 4, "", 5);
    }

    #[test]
    fn semicolon_keeps_dot_when_the_second_address_fails() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 1);
        buf.set_mark(2, 'a').unwrap();
        buf.snapshot();
        let modified = buf.is_modified();

        assert_eq!(
            parse_addresses(&mut buf, "3;99p"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(buf.current(), 3);
        // Lines, marks, dirty flag, and the snapshot are untouched.
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.get_mark('a').unwrap(), 2);
        assert_eq!(buf.is_modified(), modified);
        assert!(buf.has_snapshot());

        buf.set_current(1).unwrap();
        assert_eq!(
            parse_addresses(&mut buf, "4;'zp"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(buf.current(), 4);

        // The address before `;` is checked first, so a bad one does not move `.`.
        buf.set_current(1).unwrap();
        assert_eq!(
            parse_addresses(&mut buf, "99;1"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(buf.current(), 1);

        // Comma never moves `.`, including when the second address is bad.
        assert_eq!(
            parse_addresses(&mut buf, "4,99"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(buf.current(), 1);
    }

    #[test]
    fn marks_and_invalid_marks() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 1);
        buf.set_mark(2, 'a').unwrap();
        buf.set_mark(5, 'z').unwrap();
        assert_parse(&mut buf, "'a", 1, 2, 2, "", 1);
        assert_parse(&mut buf, "'ap", 1, 2, 2, "p", 1);
        assert_parse(&mut buf, "'a+1", 1, 3, 3, "", 1);
        assert_parse(&mut buf, "'z", 1, 5, 5, "", 1);
        // The mark letter is one byte. Anything after it is the command tail.
        assert_parse(&mut buf, "'ab", 1, 2, 2, "b", 1);

        assert_eq!(
            parse_addresses(&mut buf, "'bp"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(
            parse_addresses(&mut buf, "'A"),
            Err(AddressError::InvalidMark)
        );
        assert_eq!(
            parse_addresses(&mut buf, "'"),
            Err(AddressError::InvalidMark)
        );
        assert_eq!(
            parse_addresses(&mut buf, "'1"),
            Err(AddressError::InvalidMark)
        );
        // A dot after a finished address is not a new base address.
        assert_eq!(
            parse_addresses(&mut buf, "1 ."),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(buf.current(), 1);
    }

    #[test]
    fn invalid_addresses_and_regex_hook() {
        let mut buf = buf_with(&["a", "b", "c", "d", "e"], 1);
        assert_eq!(
            parse_addresses(&mut buf, "99"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(
            parse_addresses(&mut buf, "$+"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(
            parse_addresses(&mut buf, "$+1"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(
            parse_addresses(&mut buf, "--"),
            Err(AddressError::InvalidAddress)
        );
        // Reversed ranges parse. The command rejects them later.
        assert_parse(&mut buf, "5,4", 2, 5, 4, "", 1);
        assert_parse(&mut buf, "0,1", 2, 0, 1, "", 1);
        assert_parse(&mut buf, "0,0", 2, 0, 0, "", 1);

        // Literals outside signed 32-bit INT_MAX are "Number out of range".
        // INT_MAX itself is a legal literal and then fails the line check.
        assert_eq!(
            parse_addresses(&mut buf, "2147483647"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(
            parse_addresses(&mut buf, "2147483648"),
            Err(AddressError::NumberOutOfRange)
        );
        assert_eq!(
            parse_addresses(&mut buf, "999999999999999999999"),
            Err(AddressError::NumberOutOfRange)
        );
        assert_eq!(
            parse_addresses(&mut buf, "+999999999999999999999"),
            Err(AddressError::NumberOutOfRange)
        );

        assert_eq!(
            parse_addresses(&mut buf, "/a/"),
            Err(AddressError::RegexNotSupported)
        );
        assert_eq!(
            parse_addresses(&mut buf, "?a?"),
            Err(AddressError::RegexNotSupported)
        );
        // After a base address, `/` is not a search; GNU calls it an invalid address.
        assert_eq!(
            parse_addresses(&mut buf, "1/a/"),
            Err(AddressError::InvalidAddress)
        );
        // A search after `;` still reports the regex hook, and `.` stays moved.
        buf.set_current(1).unwrap();
        assert_eq!(
            parse_addresses(&mut buf, "3;/a/"),
            Err(AddressError::RegexNotSupported)
        );
        assert_eq!(buf.current(), 3);
    }

    #[test]
    fn empty_buffer_addresses() {
        let mut buf = Buffer::new();
        assert_eq!(buf.current(), 0);
        assert_parse(&mut buf, "", 0, 0, 0, "", 0);
        assert_parse(&mut buf, ".", 1, 0, 0, "", 0);
        assert_parse(&mut buf, "$", 1, 0, 0, "", 0);
        assert_parse(&mut buf, "0", 1, 0, 0, "", 0);
        // `1` is a finished address past `$`.
        assert_eq!(
            parse_addresses(&mut buf, "1"),
            Err(AddressError::InvalidAddress)
        );
        // Omitted `,` / `%` invent line 1 even though it does not exist.
        // The command, not the parser, rejects `1,0`.
        assert_parse(&mut buf, ",", 2, 1, 0, "", 0);
        assert_parse(&mut buf, "%", 2, 1, 0, "", 0);
        // Omitted `;` is `.,$` which is `0,0` on an empty buffer.
        assert_parse(&mut buf, ";", 2, 0, 0, "", 0);
        assert_eq!(
            parse_addresses(&mut buf, "+"),
            Err(AddressError::InvalidAddress)
        );
        assert_eq!(
            parse_addresses(&mut buf, "-"),
            Err(AddressError::InvalidAddress)
        );
    }

    #[test]
    fn trailing_vertical_tab_is_not_an_internal_blank() {
        let mut buf = buf_with(&["a", "b"], 1);
        // VT in the middle is not space or tab, so it ends the address.
        assert_parse(&mut buf, "1\x0bp", 1, 1, 1, "\x0bp", 1);
        // A space opens a blank run, and that run does consume VT.
        assert_parse(&mut buf, "1 \x0bp", 1, 1, 1, "p", 1);
    }
}
