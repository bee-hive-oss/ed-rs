//! Substitute (the s command).
//!
//! Ports main_loop.c command_s (the 's' arm of exec_command).

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// 's': `range::set_addr_range2`, then `subst_repeat::parse_repeat`.
///
/// When that returns false, read a full command, as command_s does:
/// `RegexState::get_pattern_for_s`, remember the delimiter,
/// `regex::replace::extract_replacement`, then either a newline right after
/// (print with p) or the closing delimiter and
/// `subst_suffix::get_command_s_suffix`. Store `ed.s_pflags`, `ed.s_snum`,
/// and `ed.s_pmask` (the suffix print flags, or p when none). Compile with
/// `RegexState::set_subst_regex` only after the suffixes are read (they may
/// hold `I`).
///
/// For the `next_line` argument of extract_replacement, build a closure
/// from `ed.stdin` and `ed.linenum` with `io::stdin_line::get_stdin_line`
/// (not `Editor::next_line`), so it does not borrow all of `ed` while
/// `ed.regex` is borrowed.
///
/// Then for both forms (the repeat form leaves its newline unread, as
/// command_s does): set `ed.pflags` to `ed.s_pflags`, unless `isglobal` take an undo snapshot
/// (`ed.snapshot()`),
/// and run `subst_run::search_and_replace` on the range with `ed.s_snum`.
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("port main_loop.c command_s (the 's' arm of exec_command)")
}
