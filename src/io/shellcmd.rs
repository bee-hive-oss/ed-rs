//! Read a shell command operand (main_loop.c get_shell_command).

#![allow(dead_code)]

use crate::commands::editor::Editor;
use crate::commands::CommandError;
use std::io::{self, Write};

/// Ports main_loop.c `get_shell_command`.
///
/// `ed.pos` is just after the `!`. In restricted mode fail with
/// `Message("Shell access restricted")`. Join continuation lines with
/// `io::extended::get_extended_line(.., true, ..)` (use `ed.next_line`).
/// Build the command text with a leading `!` marker. A `!` as the first
/// character is replaced by the previous command in `ed.shell_cmd`
/// (`Message("No previous command")` when it is empty, or when
/// `ed.opts.traditional` is set and the previous command is just `!`). A `%`
/// is replaced by `ed.def_filename` (`NoCurrentFilename` when empty);
/// `\%` is a literal `%`; other backslash pairs are kept. Stop at the
/// newline and consume it. Store the result in `ed.shell_cmd`. If a `!` or
/// `%` was replaced, print the final command (without the marker) and a
/// newline to stdout.
///
/// Return the command including its leading `!`.
pub fn get_shell_command(ed: &mut Editor) -> Result<String, CommandError> {
    if ed.opts.restricted {
        return Err(CommandError::Message("Shell access restricted"));
    }
    let first = ed.rest().to_string();
    let joined = {
        let mut next = || ed.next_line();
        crate::io::extended::get_extended_line(&first, true, &mut next)?
    };
    ed.cmd = joined;
    ed.pos = 0;

    let mut buf = String::new();
    let mut replacement = false;
    if ed.peek() != Some('!') {
        buf.push('!');
    } else if ed.shell_cmd.is_empty()
        || (ed.opts.traditional && ed.shell_cmd.as_bytes().get(1).copied().unwrap_or(0) == 0)
    {
        return Err(CommandError::Message("No previous command"));
    } else {
        buf.push_str(&ed.shell_cmd);
        ed.bump();
        replacement = true;
    }
    while let Some(ch) = ed.peek() {
        if ch == '\n' {
            break;
        }
        if ch == '%' {
            if ed.def_filename.is_empty() {
                return Err(CommandError::NoCurrentFilename);
            }
            buf.push_str(&ed.def_filename);
            ed.bump();
            replacement = true;
            continue;
        }
        ed.bump();
        if ch != '\\' {
            buf.push(ch);
            continue;
        }
        let Some(next) = ed.bump() else {
            buf.push('\\');
            break;
        };
        if next != '%' {
            buf.push('\\');
        }
        buf.push(next);
    }
    if ed.peek() == Some('\n') {
        ed.bump();
    }
    ed.shell_cmd = buf;
    if replacement {
        let shown = ed.shell_cmd.strip_prefix('!').unwrap_or(&ed.shell_cmd);
        let mut out = io::stdout();
        let _ = writeln!(out, "{shown}");
        let _ = out.flush();
    }
    Ok(ed.shell_cmd.clone())
}

#[cfg(test)]
mod tests {
    use super::get_shell_command;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::CommandError;

    fn ed(cmd: &str, stdin: &str) -> Editor {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new(stdin.to_string())));
        e.cmd = cmd.into();
        e
    }

    #[test]
    fn builds_marks_and_replaces() {
        let mut e = ed("ls\n", "");
        assert_eq!(get_shell_command(&mut e).unwrap(), "!ls");
        assert_eq!(e.shell_cmd, "!ls");
        assert_eq!(e.peek(), None);

        let mut e = ed("echo \\%\n", "");
        assert_eq!(get_shell_command(&mut e).unwrap(), "!echo %");

        let mut e = ed("!\n", "");
        e.shell_cmd = "!ls".into();
        assert_eq!(get_shell_command(&mut e).unwrap(), "!ls");

        let mut e = ed("cat %\n", "");
        e.def_filename = "foo".into();
        assert_eq!(get_shell_command(&mut e).unwrap(), "!cat foo");
    }

    #[test]
    fn rejects_restricted_missing_history_and_filename() {
        let mut e = ed("ls\n", "");
        e.opts.restricted = true;
        assert_eq!(
            get_shell_command(&mut e),
            Err(CommandError::Message("Shell access restricted"))
        );
        let mut e = ed("!\n", "");
        assert_eq!(
            get_shell_command(&mut e),
            Err(CommandError::Message("No previous command"))
        );
        let mut e = ed("!\n", "");
        e.opts.traditional = true;
        e.shell_cmd = "!".into();
        assert_eq!(
            get_shell_command(&mut e),
            Err(CommandError::Message("No previous command"))
        );
        let mut e = ed("%\n", "");
        assert_eq!(get_shell_command(&mut e), Err(CommandError::NoCurrentFilename));
    }
}
