//! Read the replacement part of an s command (regex.c extract_replacement).

#![allow(dead_code)]

use super::state::RegexState;
use crate::commands::CommandError;

/// Ports regex.c `extract_replacement`.
///
/// `cmd[*pos..]` starts at the delimiter that follows the pattern, for
/// example the second `/` in `s/a/b/`. Read the replacement up to the next
/// unescaped delimiter, or up to the newline when the last delimiter is
/// omitted (in a global command the newline only ends it when it is also the
/// end of the command list, because newlines inside it are unescaped). On
/// return `*pos` is at that closing delimiter or newline.
///
/// A replacement that is just `%` (followed by the delimiter or the
/// newline) means "repeat the previous replacement": keep the stored one, or
/// `Message("No previous substitution")` if there is none. An empty
/// replacement is stored as an empty string. A backslash keeps the next
/// character as it is in the stored text (`subst::line_replace` interprets
/// `\&`, `\1`..`\9`, and `&` later). When the backslash is followed by a
/// newline and `isglobal` is false, call `next_line` for more text,
/// continue with it, and make `cmd` that new line with `*pos` moved into it.
/// A missing delimiter right at the start (a newline) is
/// `Message("Missing pattern delimiter")`.
///
/// The replacement is stored in `state.replacement`.
pub fn extract_replacement(
    state: &mut RegexState,
    cmd: &mut String,
    pos: &mut usize,
    isglobal: bool,
    next_line: &mut dyn FnMut() -> Option<String>,
) -> Result<(), CommandError> {
    let Some(delim) = cmd[*pos..].chars().next() else {
        return Err(CommandError::Message("Missing pattern delimiter"));
    };
    if delim == '\n' {
        return Err(CommandError::Message("Missing pattern delimiter"));
    }
    *pos += delim.len_utf8();
    if percent(cmd, *pos, delim, isglobal) {
        *pos += 1;
        if state.replacement.is_none() {
            return Err(CommandError::Message("No previous substitution"));
        }
        return Ok(());
    }
    let mut buf = String::new();
    while let Some(ch) = cmd[*pos..].chars().next() {
        if ch == delim || (ch == '\n' && (!isglobal || cmd.len() == *pos + 1)) {
            break;
        }
        *pos += ch.len_utf8();
        buf.push(ch);
        if ch == '\\' {
            let Some(ch2) = cmd[*pos..].chars().next() else { break };
            *pos += ch2.len_utf8();
            buf.push(ch2);
            if ch2 == '\n' && !isglobal {
                let Some(line) = next_line() else {
                    return Err(CommandError::Message("Unexpected end-of-file"));
                };
                *cmd = line;
                *pos = 0;
            }
        }
    }
    state.replacement = Some(buf);
    Ok(())
}

fn percent(cmd: &str, pos: usize, delim: char, isglobal: bool) -> bool {
    if !cmd[pos..].starts_with('%') {
        return false;
    }
    let n = pos + 1;
    let b = cmd.as_bytes();
    let nxt = b.get(n).copied().unwrap_or(0);
    nxt == delim as u8 || (nxt == b'\n' && (!isglobal || b.get(n + 1).copied().unwrap_or(0) == 0))
}

#[cfg(test)]
mod tests {
    use super::extract_replacement;
    use super::super::state::RegexState;
    use crate::commands::CommandError::Message;

    fn take(src: &str, global: bool, more: Option<&str>) -> (RegexState, String, usize) {
        let mut st = RegexState::default();
        let mut cmd = src.to_string();
        let mut pos = 0;
        let mut extra = more.map(str::to_string);
        extract_replacement(&mut st, &mut cmd, &mut pos, global, &mut || extra.take()).unwrap();
        (st, cmd, pos)
    }

    #[test]
    fn reads_replacement() {
        let (st, _, pos) = take("/Z/", false, None);
        assert_eq!(st.replacement.as_deref(), Some("Z"));
        assert_eq!(pos, 2);
        let mut st = RegexState { replacement: Some("Z".into()), ..RegexState::default() };
        let mut cmd = "/%/".to_string();
        let mut pos = 0;
        extract_replacement(&mut st, &mut cmd, &mut pos, false, &mut || None).unwrap();
        assert_eq!(st.replacement.as_deref(), Some("Z"));
        let mut cmd = "/a\\/b/".to_string();
        let mut pos = 0;
        extract_replacement(&mut st, &mut cmd, &mut pos, false, &mut || None).unwrap();
        assert_eq!(st.replacement.as_deref(), Some("a\\/b"));
        let mut cmd = "/a\\\n".to_string();
        let mut pos = 0;
        extract_replacement(&mut st, &mut cmd, &mut pos, false, &mut || Some("c/\n".into())).unwrap();
        assert_eq!(st.replacement.as_deref(), Some("a\\\nc"));
        assert_eq!(pos, 1);
        assert_eq!(extract_replacement(&mut st, &mut "\n".into(), &mut 0, false, &mut || None).unwrap_err(), Message("Missing pattern delimiter"));
        st.replacement = None;
        assert_eq!(extract_replacement(&mut st, &mut "/%/".into(), &mut 0, false, &mut || None).unwrap_err(), Message("No previous substitution"));
    }
}
