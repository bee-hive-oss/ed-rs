//! Dispatcher arms for p, n, and l (the pflabel cases of main_loop.c
//! exec_command).

#![allow(dead_code)]

use super::editor::Editor;
use super::{ArmResult, CommandError, PFlags};

/// 'p': `range::set_addr_range2`, `args::get_command_suffix`, then print
/// the range. With no extra suffix call `print::print_lines`; with
/// suffixes call `print_any::print_lines` with p added. Set the current
/// line to the last line printed, and clear `ed.pflags` (GNU sets pflags
/// to 0 after printing).
///
/// print.rs, number.rs, and list.rs write Rust strings, which would send a
/// char from U+0080 to U+00FF as two UTF-8 bytes. Have them write into a
/// `Vec<u8>`, turn that back into a String, and write it to stdout with
/// `io::bytes::write_text`, so each char goes out as the one byte it
/// stands for.
pub fn arm_p(ed: &mut Editor, _isglobal: bool) -> ArmResult {
    show(ed, PFlags { p: true, ..PFlags::default() }, Kind::Plain)
}

/// 'n': as [`arm_p`] with `number::number_lines` and n added.
pub fn arm_n(ed: &mut Editor, _isglobal: bool) -> ArmResult {
    show(ed, PFlags { n: true, ..PFlags::default() }, Kind::Number)
}

/// 'l': as [`arm_p`] with `list::list_lines` and l added.
pub fn arm_l(ed: &mut Editor, _isglobal: bool) -> ArmResult {
    show(ed, PFlags { l: true, ..PFlags::default() }, Kind::List)
}

enum Kind { Plain, Number, List }

fn show(ed: &mut Editor, own: PFlags, kind: Kind) -> ArmResult {
    super::range::set_addr_range2(ed)?;
    super::args::get_command_suffix(ed)?;
    let flags = PFlags {
        l: ed.pflags.l || own.l,
        n: ed.pflags.n || own.n,
        p: ed.pflags.p || own.p,
    };
    let mixed = (flags.l && (flags.n || flags.p)) || (flags.n && flags.p);
    if mixed {
        super::print_any::print_lines(ed, ed.first, ed.second, flags)?;
    } else {
        let (from, to) = (ed.first, ed.second);
        let traditional = ed.opts.traditional;
        let cols = ed.window_columns();
        let mut buf = Vec::new();
        let cur = match kind {
            Kind::Plain => super::print::print_lines(ed.buffer.lines(), from, to, &mut buf)?,
            Kind::Number => super::number::number_lines(ed.buffer.lines(), from, to, &mut buf)?,
            Kind::List => super::list::list_lines(ed.buffer.lines(), from, to, traditional, cols, &mut buf)?,
        };
        let text = String::from_utf8(buf).map_err(|_| CommandError::Io(std::io::ErrorKind::InvalidData))?;
        crate::io::bytes::write_text(&mut std::io::stdout(), &text)
            .map_err(|err| CommandError::Io(err.kind()))?;
        ed.buffer.set_current(cur).map_err(|_| CommandError::InvalidAddress)?;
    }
    ed.pflags = PFlags::default();
    Ok(super::Status::Ok)
}
