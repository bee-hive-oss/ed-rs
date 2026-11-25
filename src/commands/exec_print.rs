//! Dispatcher arms for p, n, and l (the pflabel cases of main_loop.c
//! exec_command).

#![allow(dead_code)]

use super::editor::Editor;
use super::ArmResult;

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
pub fn arm_p(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("p arm of main_loop.c exec_command")
}

/// 'n': as [`arm_p`] with `number::number_lines` and n added.
pub fn arm_n(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("n arm of main_loop.c exec_command")
}

/// 'l': as [`arm_p`] with `list::list_lines` and l added.
pub fn arm_l(ed: &mut Editor, isglobal: bool) -> ArmResult {
    let _ = (ed, isglobal);
    todo!("l arm of main_loop.c exec_command")
}
