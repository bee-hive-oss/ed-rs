//! Read a file name operand (main_loop.c get_filename, and main.c
//! may_access_filename).

#![allow(dead_code)]

use crate::commands::editor::Editor;
use crate::commands::CommandError;

/// Ports main.c `may_access_filename`.
///
/// Errors, in the C order: an empty name or one ending in `/` is
/// `Message("Invalid filename")`. In restricted mode, a name starting with
/// `!` is `Message("Shell access restricted")` and `..` or any name with a
/// `/` is `Message("Directory access restricted")`. A newline in the name is
/// `Message("Newline character not allowed in file names")`. Unless
/// `unsafe_names`, a byte in 7..=13, 27 or 127 is
/// `Message("Control characters not allowed in file names")`.
pub fn may_access_filename(
    name: &str,
    restricted: bool,
    unsafe_names: bool,
) -> Result<(), CommandError> {
    if name.is_empty() || name.ends_with('/') {
        return Err(CommandError::Message("Invalid filename"));
    }
    if restricted {
        if name.starts_with('!') {
            return Err(CommandError::Message("Shell access restricted"));
        }
        if name == ".." || name.contains('/') {
            return Err(CommandError::Message("Directory access restricted"));
        }
    }
    if name.contains('\n') {
        return Err(CommandError::Message("Newline character not allowed in file names"));
    }
    if !unsafe_names && name.chars().any(forbidden_control) {
        return Err(CommandError::Message("Control characters not allowed in file names"));
    }
    Ok(())
}

/// Ports main_loop.c `get_filename`.
///
/// The name stays in ed-rs's one-char-per-byte form; callers open it with
/// `io::bytes::to_path`.
///
/// Skip blanks at `ed.pos`. If the line ends there: with
/// `traditional_f_command` false and an empty `ed.def_filename` fail with
/// `NoCurrentFilename`; otherwise consume the newline and return an empty
/// string. Otherwise join continuation lines with
/// `io::extended::get_extended_line(.., true, ..)` (use `ed.next_line`),
/// and update `ed.cmd`. If the operand starts with `!`, consume it and
/// return `shellcmd::get_shell_command(ed)`. A leading `~/` becomes the
/// HOME directory when HOME is set and not empty. A name longer than
/// 4096 bytes (path_max) is `Message("Filename too long")`. Read up to the
/// newline, consume it, then check the name with [`may_access_filename`]
/// using `ed.opts.restricted` and `ed.opts.unsafe_names`.
pub fn get_filename(
    ed: &mut Editor,
    traditional_f_command: bool,
) -> Result<String, CommandError> {
    crate::commands::args::skip_blanks(ed);
    if ed.peek().is_none() || ed.peek() == Some('\n') {
        if !traditional_f_command && ed.def_filename.is_empty() {
            return Err(CommandError::NoCurrentFilename);
        }
        if ed.peek() == Some('\n') {
            ed.bump();
        }
        return Ok(String::new());
    }
    let first = ed.rest().to_string();
    let joined = {
        let mut next = || ed.next_line();
        crate::io::extended::get_extended_line(&first, true, &mut next)?
    };
    let tail = first.split_once('\n').map_or("", |(_, rest)| rest);
    ed.cmd = format!("{joined}{tail}");
    ed.pos = 0;
    if ed.peek() == Some('!') {
        ed.bump();
        return crate::io::shellcmd::get_shell_command(ed);
    }
    let mut name = String::new();
    if ed.rest().starts_with("~/") {
        if let Some(home) = home_dir() {
            name.push_str(&home);
            ed.bump();
        }
    }
    while let Some(ch) = ed.peek() {
        if ch == '\n' {
            break;
        }
        name.push(ch);
        ed.bump();
    }
    if name.chars().count() > 4096 {
        return Err(CommandError::Message("Filename too long"));
    }
    if ed.peek() == Some('\n') {
        ed.bump();
    }
    may_access_filename(&name, ed.opts.restricted, ed.opts.unsafe_names)?;
    Ok(name)
}

fn home_dir() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    (!home.is_empty()).then_some(home)
}

fn forbidden_control(c: char) -> bool {
    let b = c as u32;
    (7..=13).contains(&b) || b == 27 || b == 127
}

#[cfg(test)]
mod tests {
    use super::{get_filename, may_access_filename};
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::CommandError;

    fn ed(cmd: &str) -> Editor {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.cmd = cmd.into();
        e
    }

    #[test]
    fn empty_or_trailing_slash_is_invalid_filename() {
        for name in ["", "FILE/"] {
            assert_eq!(
                may_access_filename(name, false, false),
                Err(CommandError::Message("Invalid filename")),
            );
        }
    }

    #[test]
    fn access_rules_and_names() {
        assert!(may_access_filename("", false, false).is_err());
        assert!(may_access_filename("a/", false, false).is_err());
        assert!(may_access_filename("a", false, false).is_ok());
        assert!(may_access_filename("!x", true, false).is_err());
        assert!(may_access_filename("a/b", true, false).is_err());
        assert!(may_access_filename("a\n", false, false).is_err());
        assert!(may_access_filename("a\u{7}", false, false).is_err());
        assert!(may_access_filename("a\u{7}", false, true).is_ok());
        assert_eq!(get_filename(&mut ed("  f\n"), false).unwrap(), "f");
        assert_eq!(get_filename(&mut ed("\n"), false), Err(CommandError::NoCurrentFilename));
        let mut e = ed("\n");
        e.def_filename = "old".into();
        assert_eq!(get_filename(&mut e, false).unwrap(), "");
        assert_eq!(get_filename(&mut ed("\n"), true).unwrap(), "");
        assert_eq!(get_filename(&mut ed("!ls\n"), false).unwrap(), "!ls");
        let mut e = ed("f\nw\n");
        assert_eq!(get_filename(&mut e, false).unwrap(), "f");
        assert_eq!(e.rest(), "w\n");
    }
}
