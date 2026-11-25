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
    let _ = ed;
    todo!("port the repeat form of main_loop.c command_s")
}
