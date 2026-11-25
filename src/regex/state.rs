//! The last-used and last-substituted regexes (regex.c statics).

#![allow(dead_code)]

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
        let _ = (input, extended);
        todo!("port regex.c get_compiled_regex")
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
        let _ = input;
        todo!("port regex.c get_pattern_for_s")
    }
}
