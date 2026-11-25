//! Suffixes of a full s command (main_loop.c get_command_s_suffix).

#![allow(dead_code)]

use super::editor::Editor;
use super::{CommandError, PFlags};

/// What [`get_command_s_suffix`] read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SSuffix {
    pub pflags: PFlags,
    /// Occurrence to replace: 1 by default, n for a count, 0 for g.
    pub snum: i32,
    pub ignore_case: bool,
}

/// Ports main_loop.c `get_command_s_suffix`.
///
/// Read at `ed.pos`, in any order: a count starting with 1 to 9
/// (read with `args::parse_int`, must be > 0) or `g`, but only one of the two; `i` or
/// `I` once; and `l`, `n`, `p` once each. A second count or g, a count of
/// 0, or anything but a newline at the end is `InvalidCommandSuffix`. A
/// repeated `i`, `l`, `n` or `p` stops the scan (so the newline check then
/// fails). Consume the newline. Start from `pflags` empty, `snum` 1.
pub fn get_command_s_suffix(ed: &mut Editor) -> Result<SSuffix, CommandError> {
    let mut pflags = PFlags::default();
    let mut snum = 1;
    let mut ignore_case = false;
    let mut rep = false;
    let mut error = false;
    loop {
        match ed.peek() {
            Some('1'..='9') => {
                if rep {
                    error = true;
                    break;
                }
                match super::args::parse_int(ed) {
                    Ok(n) if n > 0 => {
                        rep = true;
                        snum = n;
                    }
                    _ => {
                        error = true;
                        break;
                    }
                }
            }
            Some('g') => {
                if rep {
                    break;
                }
                rep = true;
                snum = 0;
                ed.bump();
            }
            Some('i' | 'I') => {
                if ignore_case {
                    break;
                }
                ignore_case = true;
                ed.bump();
            }
            Some('l') => flag(&mut pflags.l, ed, &mut error),
            Some('n') => flag(&mut pflags.n, ed, &mut error),
            Some('p') => flag(&mut pflags.p, ed, &mut error),
            _ => break,
        }
        if error {
            break;
        }
    }
    if error || ed.bump() != Some('\n') {
        return Err(CommandError::InvalidCommandSuffix);
    }
    Ok(SSuffix { pflags, snum, ignore_case })
}

fn flag(slot: &mut bool, ed: &mut Editor, error: &mut bool) {
    if *slot {
        *error = true;
    } else {
        *slot = true;
        ed.bump();
    }
}

#[cfg(test)]
mod tests {
    use super::get_command_s_suffix;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::CommandError::InvalidCommandSuffix;
    use std::io::Cursor;

    fn ed(cmd: &str) -> Editor {
        let mut ed = Editor::new(Options::default(), Box::new(Cursor::new("")));
        ed.cmd = cmd.to_string();
        ed
    }

    #[test]
    fn suffixes() {
        let mut e = ed("gp\n");
        let s = get_command_s_suffix(&mut e).unwrap();
        assert_eq!(s.snum, 0);
        assert!(s.pflags.p);
        assert_eq!(e.pos, 3);
        let mut e = ed("2I\n");
        let s = get_command_s_suffix(&mut e).unwrap();
        assert_eq!(s.snum, 2);
        assert!(s.ignore_case);
        assert!(get_command_s_suffix(&mut ed("gg\n")).is_err());
        assert_eq!(get_command_s_suffix(&mut ed("p x\n")).unwrap_err(), InvalidCommandSuffix);
    }
}
