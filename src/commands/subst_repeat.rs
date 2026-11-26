//! The repeat form of the s command (the first half of main_loop.c
//! command_s).

#![allow(dead_code)]

use super::editor::Editor;
use super::CommandError;

/// Ports the `do ... while( sflags && **ibufpp != '\n' )` loop of
/// main_loop.c command_s and the `if( sflags )` block after it.
///
/// Read at `ed.pos`: a count (with `args::parse_int`, must be > 0, sets
/// `ed.s_snum`), `g` (flips `ed.s_snum` between 0 and 1, C `snum =
/// !snum`), `p`, `r`, each at most once, a count and g sharing one slot.
/// A newline right away also counts as the repeat form. A repeat, a bad
/// count, or any other character once a flag was read is
/// `InvalidCommandSuffix`. Any other first character means a full s
/// command follows: return `Ok(false)` without moving `ed.pos`.
///
/// For the repeat form: no `ed.regex.subst` is `Message("No previous
/// substitution")`; with `r` call
/// `RegexState::replace_subst_re_by_search_re`; with `p` set `ed.s_pflags`
/// to `ed.s_pflags.xor(ed.s_pmask)`. Leave `ed.pos` on the newline and
/// return `Ok(true)`.
pub fn parse_repeat(ed: &mut Editor) -> Result<bool, CommandError> {
    const G: u8 = 0x01;
    const P: u8 = 0x02;
    const R: u8 = 0x04;
    let mut sflags = 0u8;
    loop {
        if !step(ed, &mut sflags)? {
            break;
        }
        if ed.peek() == Some('\n') {
            break;
        }
    }
    if sflags == 0 {
        return Ok(false);
    }
    if ed.regex.subst.is_none() {
        return Err(CommandError::Message("No previous substitution"));
    }
    if sflags & R != 0 {
        ed.regex.replace_subst_re_by_search_re()?;
    }
    if sflags & P != 0 {
        ed.s_pflags = ed.s_pflags.xor(ed.s_pmask);
    }
    Ok(true)
}

/// One flag. `Ok(false)` ends the scan with `sflags` still 0 (not a repeat).
fn step(ed: &mut Editor, sflags: &mut u8) -> Result<bool, CommandError> {
    const G: u8 = 0x01;
    const P: u8 = 0x02;
    const R: u8 = 0x04;
    const NONE: u8 = 0x08;
    match ed.peek() {
        Some('1'..='9') => {
            if *sflags & G != 0 {
                return Err(CommandError::InvalidCommandSuffix);
            }
            let n = super::args::parse_int(ed).map_err(|_| CommandError::InvalidCommandSuffix)?;
            if n <= 0 {
                return Err(CommandError::InvalidCommandSuffix);
            }
            *sflags |= G;
            ed.s_snum = n;
        }
        Some('\n') => *sflags |= NONE,
        Some('g') => bit(sflags, G, ed)?,
        Some('p') => bit(sflags, P, ed)?,
        Some('r') => bit(sflags, R, ed)?,
        _ => {
            if *sflags != 0 {
                return Err(CommandError::InvalidCommandSuffix);
            }
            return Ok(false);
        }
    }
    Ok(true)
}

fn bit(sflags: &mut u8, flag: u8, ed: &mut Editor) -> Result<(), CommandError> {
    if *sflags & flag != 0 {
        return Err(CommandError::InvalidCommandSuffix);
    }
    *sflags |= flag;
    if flag == 0x01 {
        ed.s_snum = i32::from(ed.s_snum == 0);
    }
    ed.bump();
    Ok(())
}
