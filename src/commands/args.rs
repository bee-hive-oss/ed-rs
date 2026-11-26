//! Small command-line readers shared by the arms (main_loop.c
//! skip_blanks, parse_int, get_command_suffix, unexpected_address,
//! unexpected_command_suffix).
//!
//! Each reads `ed.cmd` at `ed.pos` and moves `ed.pos` forward.

#![allow(dead_code)]

use super::editor::Editor;
use super::CommandError;

/// Ports main_loop.c `skip_blanks`: skip C `isspace` characters other than
/// newline.
pub fn skip_blanks(ed: &mut Editor) {
    while let Some(c) = ed.peek() {
        if c == '\n' || !is_c_space(c) {
            break;
        }
        ed.bump();
    }
}

/// Ports main_loop.c `parse_int` (strtol base 10, optional sign).
///
/// No digits is `Message("Invalid number")`. A value above `i32::MAX` or
/// below `-i32::MAX` is `Message("Number out of range")`. On success move
/// past the number and return it.
pub fn parse_int(ed: &mut Editor) -> Result<i32, CommandError> {
    let s = ed.rest();
    let mut i = 0;
    for (idx, c) in s.char_indices() {
        if is_c_space(c) {
            i = idx + c.len_utf8();
        } else {
            break;
        }
    }
    let b = s.as_bytes();
    let mut j = i;
    let mut sign: i64 = 1;
    if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
        if b[j] == b'-' {
            sign = -1;
        }
        j += 1;
    }
    if j >= b.len() || !b[j].is_ascii_digit() {
        return Err(CommandError::Message("Invalid number"));
    }
    let mut val: i64 = 0;
    while j < b.len() && b[j].is_ascii_digit() {
        val = val.saturating_mul(10).saturating_add((b[j] - b'0') as i64);
        j += 1;
    }
    if val > i64::from(i32::MAX) {
        return Err(CommandError::Message("Number out of range"));
    }
    ed.pos += j;
    Ok((val * sign) as i32)
}

/// Ports main_loop.c `get_command_suffix`.
///
/// Read any of `l`, `n`, `p` into `ed.pflags`, each at most once (a repeat
/// stops the scan). Then the next character must be a newline, which is
/// consumed; anything else is `InvalidCommandSuffix`.
pub fn get_command_suffix(ed: &mut Editor) -> Result<(), CommandError> {
    loop {
        match ed.peek() {
            Some('l') if !ed.pflags.l => ed.pflags.l = true,
            Some('n') if !ed.pflags.n => ed.pflags.n = true,
            Some('p') if !ed.pflags.p => ed.pflags.p = true,
            _ => break,
        }
        ed.bump();
    }
    if ed.bump() == Some('\n') {
        Ok(())
    } else {
        Err(CommandError::InvalidCommandSuffix)
    }
}

/// Ports main_loop.c `unexpected_address`: `UnexpectedAddress` when
/// `addr_cnt > 0`.
pub fn unexpected_address(addr_cnt: usize) -> Result<(), CommandError> {
    if addr_cnt > 0 {
        Err(CommandError::UnexpectedAddress)
    } else {
        Ok(())
    }
}

/// Ports main_loop.c `unexpected_command_suffix`: the next character must
/// be C `isspace` (newline included) or the error is
/// `UnexpectedCommandSuffix`. Does not move `ed.pos`.
pub fn unexpected_command_suffix(ed: &Editor) -> Result<(), CommandError> {
    match ed.peek() {
        Some(c) if is_c_space(c) => Ok(()),
        _ => Err(CommandError::UnexpectedCommandSuffix),
    }
}

fn is_c_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c')
}

#[cfg(test)]
mod tests {
    use super::{get_command_suffix, parse_int, skip_blanks, unexpected_address, unexpected_command_suffix};
    use super::super::editor::Editor;
    use crate::cli::Options;
    use crate::commands::CommandError::{InvalidCommandSuffix, Message, UnexpectedAddress};

    fn ed(cmd: &str) -> Editor {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.cmd = cmd.into();
        e
    }

    #[test]
    fn blanks_numbers_suffixes() {
        let mut e = ed(" \t\x0b12x\n");
        skip_blanks(&mut e);
        assert_eq!(parse_int(&mut e).unwrap(), 12);
        assert_eq!(&e.cmd[e.pos..], "x\n");
        assert!(unexpected_command_suffix(&e).is_err());
        let e = ed("\n");
        assert!(unexpected_command_suffix(&e).is_ok());
        assert_eq!(parse_int(&mut ed("x")).unwrap_err(), Message("Invalid number"));
        assert_eq!(parse_int(&mut ed("2147483648")).unwrap_err(), Message("Number out of range"));
        assert_eq!(parse_int(&mut ed("-2147483648")).unwrap_err(), Message("Number out of range"));
        assert_eq!(parse_int(&mut ed("-2147483647")).unwrap(), -2147483647);
        let mut e = ed("lp\n");
        get_command_suffix(&mut e).unwrap();
        assert!(e.pflags.l && e.pflags.p && !e.pflags.n);
        assert!(e.rest().is_empty());
        assert_eq!(get_command_suffix(&mut ed("pp\n")).unwrap_err(), InvalidCommandSuffix);
        assert!(unexpected_address(0).is_ok());
        assert_eq!(unexpected_address(1).unwrap_err(), UnexpectedAddress);
    }
}
