//! Default ranges and the third address (main_loop.c set_addr_range,
//! set_addr_range2, set_second_addr, get_third_addr).
//!
//! These read and write `ed.first`, `ed.second`, and `ed.addr_cnt`, which
//! `exec::exec_command` fills from the address parser.

#![allow(dead_code)]

use super::editor::Editor;
use super::exec_search;
use super::CommandError;

/// Ports main_loop.c `set_addr_range`. When `ed.addr_cnt == 0` use `n,m`.
/// Then `first < 1`, `first > second`, or `second > $` is `InvalidAddress`.
pub fn set_addr_range(ed: &mut Editor, n: usize, m: usize) -> Result<(), CommandError> {
    if ed.addr_cnt == 0 {
        ed.first = n;
        ed.second = m;
    }
    if ed.first < 1 || ed.first > ed.second || ed.second > ed.buffer.len() {
        Err(CommandError::InvalidAddress)
    } else {
        Ok(())
    }
}

/// Ports main_loop.c `set_addr_range2`: [`set_addr_range`] with `.,.`.
pub fn set_addr_range2(ed: &mut Editor) -> Result<(), CommandError> {
    let dot = ed.buffer.current();
    set_addr_range(ed, dot, dot)
}

/// Ports main_loop.c `set_second_addr`. When `ed.addr_cnt == 0` use
/// `addr`. Then `second < 1` or `second > $` is `InvalidAddress`.
pub fn set_second_addr(ed: &mut Editor, addr: usize) -> Result<(), CommandError> {
    if ed.addr_cnt == 0 {
        ed.second = addr;
    }
    if ed.second < 1 || ed.second > ed.buffer.len() {
        Err(CommandError::InvalidAddress)
    } else {
        Ok(())
    }
}

/// Ports main_loop.c `get_third_addr` for m and t.
///
/// Parse one more address at `ed.pos` with
/// `exec_search::extract_addresses`, then put `first`, `second`, and
/// `addr_cnt` back as they were. With `ed.opts.traditional` and no address
/// written, fail with `Message("Destination expected")`. A result outside
/// `0..=$` is `InvalidAddress`. Return the address.
pub fn get_third_addr(ed: &mut Editor) -> Result<usize, CommandError> {
    let old = (ed.first, ed.second, ed.addr_cnt);
    let parsed = exec_search::extract_addresses(ed);
    let cnt = match parsed {
        Ok(cnt) => cnt,
        Err(err) => {
            ed.first = old.0;
            ed.second = old.1;
            ed.addr_cnt = old.2;
            return Err(err);
        }
    };
    let addr = ed.second;
    ed.first = old.0;
    ed.second = old.1;
    ed.addr_cnt = old.2;
    if ed.opts.traditional && cnt == 0 {
        return Err(CommandError::Message("Destination expected"));
    }
    if addr > ed.buffer.len() {
        return Err(CommandError::InvalidAddress);
    }
    Ok(addr)
}

#[cfg(test)]
mod tests {
    use super::{get_third_addr, set_addr_range, set_addr_range2, set_second_addr};
    use super::super::editor::Editor;
    use crate::cli::Options;
    use crate::commands::CommandError::{InvalidAddress, Message};

    fn ed(n: usize, cmd: &str) -> Editor {
        let mut e = Editor::new(Options::default(), Box::new(std::io::Cursor::new("")));
        e.buffer.load((1..=n).map(|i| i.to_string()));
        e.buffer.set_current(1).unwrap();
        e.cmd = cmd.into();
        e
    }

    #[test]
    fn defaults_and_third() {
        let mut e = ed(3, "");
        set_addr_range(&mut e, 1, 3).unwrap();
        assert_eq!((e.first, e.second), (1, 3));
        e.addr_cnt = 2;
        e.first = 3;
        e.second = 1;
        assert_eq!(set_addr_range(&mut e, 1, 3).unwrap_err(), InvalidAddress);
        e.addr_cnt = 0;
        set_addr_range2(&mut e).unwrap();
        assert_eq!((e.first, e.second), (1, 1));
        e.addr_cnt = 0;
        set_second_addr(&mut e, 2).unwrap();
        assert_eq!(e.second, 2);
        e.addr_cnt = 2;
        e.first = 1;
        e.second = 2;
        e.pos = 0;
        e.cmd = "0\n".into();
        assert_eq!(get_third_addr(&mut e).unwrap(), 0);
        assert_eq!((e.first, e.second, e.addr_cnt), (1, 2, 2));
        e.opts.traditional = true;
        e.pos = 0;
        e.cmd = "p\n".into();
        assert_eq!(get_third_addr(&mut e).unwrap_err(), Message("Destination expected"));
    }
}
