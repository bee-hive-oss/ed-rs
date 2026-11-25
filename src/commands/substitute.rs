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
    super::range::set_addr_range2(ed)?;
    if !super::subst_repeat::parse_repeat(ed)? {
        full_form(ed, isglobal)?;
    }
    ed.pflags = ed.s_pflags;
    if !isglobal {
        ed.snapshot();
    }
    let (from, to, snum) = (ed.first, ed.second, ed.s_snum);
    super::subst_run::search_and_replace(ed, from, to, snum, isglobal)?;
    Ok(super::Status::Ok)
}

fn full_form(ed: &mut Editor, isglobal: bool) -> Result<(), super::CommandError> {
    let input = ed.rest().to_string();
    let (pat, used) = ed.regex.get_pattern_for_s(&input)?;
    ed.pos += used;
    let delim = ed.peek().unwrap_or('\0');
    {
        let Editor { regex, cmd, pos, stdin, linenum, .. } = &mut *ed;
        let mut next = || {
            crate::io::stdin_line::get_stdin_line(stdin.as_mut(), linenum)
                .ok()
                .flatten()
        };
        crate::regex::replace::extract_replacement(regex, cmd, pos, isglobal, &mut next)?;
    }
    let extended = ed.opts.extended_regexp;
    if ed.peek() == Some('\n') {
        ed.bump();
        let p = super::PFlags { l: false, n: false, p: true };
        ed.s_pflags = p;
        ed.s_snum = 1;
        ed.s_pmask = p;
        ed.regex.set_subst_regex(&pat, false, extended)?;
    } else {
        if ed.peek() == Some(delim) {
            ed.bump();
        }
        let suffix = super::subst_suffix::get_command_s_suffix(ed)?;
        ed.s_pflags = suffix.pflags;
        ed.s_snum = suffix.snum;
        ed.s_pmask = if suffix.pflags.is_empty() {
            super::PFlags { l: false, n: false, p: true }
        } else {
            suffix.pflags
        };
        ed.regex.set_subst_regex(&pat, suffix.ignore_case, extended)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::arm;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::Status;

    #[test]
    fn replaces_on_the_current_line() {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(["ab", "a"]);
        e.buffer.set_current(1).unwrap();
        e.cmd = "/a/X/\n".into();
        assert_eq!(arm(&mut e, false).unwrap(), Status::Ok);
        assert_eq!(e.buffer.line(1).unwrap(), "Xb");
        assert_eq!(e.buffer.line(2).unwrap(), "a");
    }
}
