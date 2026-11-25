//! The substitution regex (regex.c set_subst_regex and
//! replace_subst_re_by_search_re).
//!
//! These are methods on [`RegexState`], kept in their own file so state.rs
//! stays small.

#![allow(dead_code)]

use super::state::RegexState;
use super::Regex;
use crate::commands::CommandError;
use std::rc::Rc;

impl RegexState {
    /// Ports regex.c `set_subst_regex` and `compile_regex`.
    ///
    /// An empty `pat` with `icase` is `Message("Suffix 'I' not allowed on
    /// empty regexp")`. An empty `pat` reuses `last` (or fails
    /// with `Message("No previous pattern")`). Otherwise compile with
    /// `Regex::compile`, store it in `last`, and make it the `subst` regex.
    pub fn set_subst_regex(
        &mut self,
        pat: &str,
        icase: bool,
        extended: bool,
    ) -> Result<(), CommandError> {
        if pat.is_empty() {
            if icase {
                return Err(CommandError::Message("Suffix 'I' not allowed on empty regexp"));
            }
            let Some(last) = self.last.clone() else {
                return Err(CommandError::Message("No previous pattern"));
            };
            self.subst = Some(last);
            return Ok(());
        }
        let re = Rc::new(
            Regex::compile(pat, icase, extended).map_err(|e| CommandError::Message(e.message()))?,
        );
        self.last = Some(Rc::clone(&re));
        self.subst = Some(re);
        Ok(())
    }

    /// Ports regex.c `replace_subst_re_by_search_re`: make `subst` the same
    /// regex as `last`, or `Message("No previous pattern")`.
    pub fn replace_subst_re_by_search_re(&mut self) -> Result<(), CommandError> {
        let Some(last) = self.last.clone() else {
            return Err(CommandError::Message("No previous pattern"));
        };
        self.subst = Some(last);
        Ok(())
    }
}
