//! The last-used and last-substituted regexes (regex.c statics).

#![allow(dead_code)]

use super::pattern::extract_pattern;
use super::Regex;
use crate::commands::CommandError;
use std::rc::Rc;

/// Regex memory shared by addresses, g, v, G, V and s.
///
/// The two s-only methods, `set_subst_regex` and
/// `replace_subst_re_by_search_re`, live in `subst_re.rs`.
#[derive(Debug, Default)]
pub struct RegexState {
    /// Last regex used by a search or a global command (C last_regexp).
    pub last: Option<Rc<Regex>>,
    /// Regex of the last substitution (C subst_regexp).
    pub subst: Option<Rc<Regex>>,
    /// Replacement text of the last substitution (C rbuf).
    pub replacement: Option<String>,
}

impl RegexState {
    /// Ports regex.c `get_compiled_regex` (with `compile_regex`).
    ///
    /// `input` starts at the opening delimiter, for example `/abc/I` or
    /// `?abc?`. A delimiter that is a space, newline, or end is
    /// `Message("Invalid pattern delimiter")`. An empty pattern (`//`) returns
    /// the last regex, or `Message("No previous pattern")`, and if it is
    /// followed by the closing delimiter and then `I` the error is
    /// `Message("Suffix 'I' not allowed on empty regexp")`. Otherwise
    /// use `pattern::extract_pattern`, skip the closing delimiter, accept a
    /// trailing `I` as ignore-case, compile with `Regex::compile` (extended
    /// when `extended`), store it in `last`, and return it.
    ///
    /// Also return the number of bytes of `input` used, so the caller can
    /// advance past the pattern, the closing delimiter, and any `I`.
    pub fn get_compiled_regex(
        &mut self,
        input: &str,
        extended: bool,
    ) -> Result<(Rc<Regex>, usize), CommandError> {
        let Some(delim) = input.chars().next() else {
            return Err(CommandError::Message("Invalid pattern delimiter"));
        };
        if delim == ' ' || delim == '\n' || delim == '\0' {
            return Err(CommandError::Message("Invalid pattern delimiter"));
        }
        let mut used = delim.len_utf8();
        let rest = &input[used..];
        let next = rest.chars().next();
        if next.is_none() || next == Some(delim) || next == Some('\n') || next == Some('\0') {
            let Some(last) = self.last.clone() else {
                return Err(CommandError::Message("No previous pattern"));
            };
            if next == Some(delim) {
                used += delim.len_utf8();
                if input[used..].starts_with('I') {
                    return Err(CommandError::Message("Suffix 'I' not allowed on empty regexp"));
                }
            }
            return Ok((last, used));
        }
        let (pat, rel) = extract_pattern(rest, delim)?;
        used += rel;
        let mut icase = false;
        if input[used..].starts_with(delim) {
            used += delim.len_utf8();
            if input[used..].starts_with('I') {
                icase = true;
                used += 1;
            }
        }
        let re = Rc::new(compile(&pat, icase, extended)?);
        self.last = Some(Rc::clone(&re));
        Ok((re, used))
    }

    /// Ports regex.c `get_pattern_for_s`.
    ///
    /// `input` starts at the opening delimiter of an s command. Return the
    /// pattern text and the bytes used, stopping on the closing delimiter
    /// (not consumed). A space or newline delimiter is `Message("Invalid
    /// pattern delimiter")`. An empty pattern needs a `last` regex
    /// (`Message("No previous pattern")`) and returns an empty string. A
    /// missing closing delimiter is `Message("Missing pattern delimiter")`.
    pub fn get_pattern_for_s(&self, input: &str) -> Result<(String, usize), CommandError> {
        let Some(delim) = input.chars().next() else {
            return Err(CommandError::Message("Invalid pattern delimiter"));
        };
        if delim == ' ' || delim == '\n' {
            return Err(CommandError::Message("Invalid pattern delimiter"));
        }
        let rest = &input[delim.len_utf8()..];
        if rest.starts_with(delim) {
            if self.last.is_none() {
                return Err(CommandError::Message("No previous pattern"));
            }
            return Ok((String::new(), delim.len_utf8()));
        }
        let (pat, rel) = extract_pattern(rest, delim)?;
        let used = delim.len_utf8() + rel;
        if !input[used..].starts_with(delim) {
            return Err(CommandError::Message("Missing pattern delimiter"));
        }
        Ok((pat, used))
    }
}

fn compile(pat: &str, icase: bool, extended: bool) -> Result<Regex, CommandError> {
    Regex::compile(pat, icase, extended).map_err(|e| CommandError::Message(e.message()))
}

#[cfg(test)]
mod tests {
    use super::RegexState;
    use crate::commands::CommandError::Message;

    #[test]
    fn last_and_s_pattern() {
        let mut s = RegexState::default();
        assert_eq!(s.get_compiled_regex(" ", false).unwrap_err(), Message("Invalid pattern delimiter"));
        assert_eq!(s.get_pattern_for_s("//").unwrap_err(), Message("No previous pattern"));
        let (re, n) = s.get_compiled_regex("/a\\+/I", false).unwrap();
        assert_eq!(n, 6);
        assert!(re.icase);
        assert_eq!(re.exec("XA", false).unwrap()[0], Some((1, 2)));
        assert_eq!(s.get_compiled_regex("//I", false).unwrap_err(), Message("Suffix 'I' not allowed on empty regexp"));
        assert_eq!(s.get_compiled_regex("//x", false).unwrap().1, 2);
        assert_eq!(s.get_pattern_for_s(" /a/").unwrap_err(), Message("Invalid pattern delimiter"));
        assert_eq!(s.get_pattern_for_s("/b").unwrap_err(), Message("Missing pattern delimiter"));
        assert_eq!(s.get_pattern_for_s("//").unwrap(), (String::new(), 1));
        assert_eq!(s.get_pattern_for_s("/b[/]/").unwrap().0, "b[/]");
        assert_eq!(s.get_compiled_regex("?b?", false).unwrap().1, 3);
    }
}
