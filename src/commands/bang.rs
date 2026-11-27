//! Shell escape and filter (the ! command).
//!
//! Ports the '!' arm of GNU ed 1.22.6 main_loop.c exec_command and main_loop.c command_shell.

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

/// '!': `io::shellcmd::get_shell_command`. With no address run it with
/// `io::shell::shell_system` (`Message("Can't create shell process")` on
/// failure) and print "!" and a newline unless `ed.opts.silent`. With an
/// address, filter the range: `range::set_addr_range2`; a `<` or `>` in
/// the command is `Message("Redirection not allowed")`; send the range
/// (encoded with `io::bytes::encode`) through `io::shell::shell_filter`
/// and decode the output with `io::bytes::decode`; unless `isglobal` take an undo
/// snapshot with `ed.snapshot()`; delete the range (with the cut buffer as delete does) and add
/// the output lines in its place, then if the current line is 0 and the
/// buffer is not empty make it 1. Print the byte count unless silent.
///
/// Clear `ed.bin.unterminated` when this changes the last line or adds
/// lines after it (in C that line node is then no longer the unterminated
/// last line).
pub fn arm(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let cmd = crate::io::shellcmd::get_shell_command(ed)?;
    let text = cmd.strip_prefix('!').unwrap_or(&cmd);
    if ed.addr_cnt == 0 {
        crate::io::shell::shell_system(text)
            .map_err(|_| super::CommandError::Message("Can't create shell process"))?;
        if !ed.opts.silent {
            println!("!");
        }
        return Ok(super::Status::Ok);
    }
    super::range::set_addr_range2(ed)?;
    if text.contains('<') || text.contains('>') {
        return Err(super::CommandError::Message("Redirection not allowed"));
    }
    let (from, to) = (ed.first, ed.second);
    let filtered = crate::io::shell::shell_filter(text, &range_bytes(ed, from, to))
        .map_err(|_| super::CommandError::Message("Can't create shell process"))?;
    let lines = output_lines(&filtered);
    if !ed.opts.silent {
        println!("{}", filtered.len());
    }
    if !isglobal {
        ed.snapshot();
    }
    let old_len = ed.buffer.len();
    super::yank::yank_lines(ed, from, to);
    if isglobal {
        ed.active.on_delete(from, to);
    }
    ed.buffer.delete(from, to).map_err(super::buf_err)?;
    let cur = ed.buffer.current();
    let at = cur - usize::from(cur >= from);
    let added = lines.len();
    ed.buffer.append(at, lines).map_err(super::buf_err)?;
    if ed.buffer.current() == 0 && !ed.buffer.is_empty() {
        ed.buffer.set_current(1).map_err(super::buf_err)?;
    }
    if isglobal {
        ed.active.on_insert(at, added);
    }
    if to == old_len {
        ed.bin.unterminated = false;
    }
    Ok(super::Status::Ok)
}

fn range_bytes(ed: &Editor, from: usize, to: usize) -> Vec<u8> {
    let encode: fn(&str) -> Vec<u8> = crate::io::bytes::encode;
    let mut raw = Vec::new();
    for addr in from..=to {
        if let Ok(line) = ed.buffer.line(addr) {
            raw.extend(encode(line));
            raw.push(b'\n');
        }
    }
    raw
}

fn output_lines(bytes: &[u8]) -> Vec<String> {
    let decode: fn(&[u8]) -> String = crate::io::bytes::decode;
    let text = decode(bytes);
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::arm;
    use crate::cli::Options;
    use crate::commands::editor::Editor;
    use crate::commands::{CommandError, Status};

    #[test]
    fn filters_a_range_and_rejects_redirection() {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load(["a", "b", "c"]);
        e.addr_cnt = 2;
        e.first = 1;
        e.second = 2;
        e.cmd = "cat\n".into();
        e.opts.silent = true;
        assert_eq!(arm(&mut e, false).unwrap(), Status::Ok);
        assert_eq!(e.buffer.lines(), ["a".to_string(), "b".into(), "c".into()]);
        e.pos = 0;
        e.cmd = "cat >x\n".into();
        e.addr_cnt = 1;
        e.first = 1;
        e.second = 1;
        assert_eq!(arm(&mut e, false), Err(CommandError::Message("Redirection not allowed")));
    }
}
