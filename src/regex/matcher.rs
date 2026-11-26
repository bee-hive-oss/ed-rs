//! Backtracking matcher: everything except `Node::Repeat`, which lives in
//! `repeat.rs`.
//!
//! Matching uses continuations. [`Matcher::run`] tries one node at `pos`
//! and, for each way it can match, calls the continuation with the position
//! after the match. If the continuation returns true the whole match
//! succeeded and `run` returns true. If it returns false, `run` backtracks
//! and tries the next way. Captures set on the way are undone on failure.

#![allow(dead_code)]

use super::ast::{Captures, ClassItem, ClassSet, NamedClass, Node};
use super::repeat::run_repeat;

/// Matching state for one `exec` attempt.
pub struct Matcher<'t> {
    pub text: &'t str,
    pub icase: bool,
    pub notbol: bool,
    /// Slot 0 is filled by the caller. Slots 1.. are groups.
    pub caps: Captures,
}

/// Continuation: called with the matcher and the position after a match.
pub type Cont<'c, 't> = &'c mut dyn FnMut(&mut Matcher<'t>, usize) -> bool;

impl<'t> Matcher<'t> {
    pub fn new(text: &'t str, icase: bool, notbol: bool, nsub: usize) -> Self {
        Matcher { text, icase, notbol, caps: vec![None; nsub + 1] }
    }

    /// Match `node` at byte offset `pos`, then call `k`.
    ///
    /// Char and Set consume one whole character (use `str::chars` on
    /// `&self.text[pos..]`). Compare case-insensitively when `icase`, ASCII
    /// letters only (C locale). Any
    /// matches one character. Bol matches at 0 unless `notbol`, Eol at the
    /// end of the text. Empty matches without consuming. Concat chains its
    /// parts, Alt tries each branch in order, Group records its span in
    /// `caps[n]` when its inner node finishes and clears it again on
    /// backtrack, and Backref matches the text of an earlier group (and
    /// fails if that group did not take part). Repeat calls
    /// `repeat::run_repeat`.
    pub fn run(&mut self, node: &Node, pos: usize, k: Cont<'_, 't>) -> bool {
        match node {
            Node::Empty => k(self, pos),
            Node::Char(c) => {
                let (c, icase) = (*c, self.icase);
                self.one(pos, move |ch| fold(ch, c, icase), k)
            }
            Node::Any => self.one(pos, |_| true, k),
            Node::Set(set) => {
                let icase = self.icase;
                self.one(pos, |ch| in_set(set, ch, icase), k)
            }
            Node::Bol => (pos == 0 && !self.notbol) && k(self, pos),
            Node::Eol => pos == self.text.len() && k(self, pos),
            Node::Concat(xs) => self.cat(xs, 0, pos, k),
            Node::Alt(xs) => {
                for br in xs {
                    if self.run(br, pos, k) { return true; }
                }
                false
            }
            Node::Group(inner, n) => {
                let n = *n;
                let saved = self.caps[n];
                let ok = self.run(inner, pos, &mut |m, end| {
                    m.caps[n] = Some((pos, end));
                    let yes = k(m, end);
                    if !yes { m.caps[n] = saved; }
                    yes
                });
                if !ok { self.caps[n] = saved; }
                ok
            }
            Node::Backref(n) => {
                let Some((a, b)) = self.caps[*n] else { return false };
                let mut i = pos;
                for c in self.text[a..b].chars() {
                    let Some((ch, np)) = self.bump(i) else { return false };
                    if !fold(ch, c, self.icase) { return false; }
                    i = np;
                }
                k(self, i)
            }
            Node::Repeat { node, min, max } => run_repeat(self, node, *min, *max, pos, k),
        }
    }

    fn one(&mut self, pos: usize, pred: impl Fn(char) -> bool, k: Cont<'_, 't>) -> bool {
        match self.bump(pos) {
            Some((ch, np)) if pred(ch) => k(self, np),
            _ => false,
        }
    }

    fn bump(&self, pos: usize) -> Option<(char, usize)> {
        let ch = self.text.get(pos..)?.chars().next()?;
        Some((ch, pos + ch.len_utf8()))
    }

    fn cat(&mut self, xs: &[Node], i: usize, pos: usize, k: Cont<'_, 't>) -> bool {
        if i >= xs.len() { return k(self, pos); }
        self.run(&xs[i], pos, &mut |m, np| m.cat(xs, i + 1, np, k))
    }
}

fn fold(a: char, b: char, icase: bool) -> bool {
    a == b || (icase && a.is_ascii_alphabetic() && a.eq_ignore_ascii_case(&b))
}

fn in_set(set: &ClassSet, ch: char, icase: bool) -> bool {
    let hit = set.items.iter().any(|item| match item {
        ClassItem::Ch(c) => fold(ch, *c, icase),
        ClassItem::Range(a, b) => in_range(ch, *a, *b, icase),
        ClassItem::Named(n) => named(*n, ch),
    });
    hit ^ set.negate
}

fn in_range(ch: char, a: char, b: char, icase: bool) -> bool {
    (ch >= a && ch <= b)
        || (icase && ch.is_ascii_alphabetic()
            && ((ch.to_ascii_lowercase() >= a && ch.to_ascii_lowercase() <= b)
                || (ch.to_ascii_uppercase() >= a && ch.to_ascii_uppercase() <= b)))
}

fn named(n: NamedClass, ch: char) -> bool {
    if !ch.is_ascii() { return false; }
    match n {
        NamedClass::Alnum => ch.is_ascii_alphanumeric(),
        NamedClass::Alpha => ch.is_ascii_alphabetic(),
        NamedClass::Blank => matches!(ch, ' ' | '\t'),
        NamedClass::Cntrl => ch.is_ascii_control(),
        NamedClass::Digit => ch.is_ascii_digit(),
        NamedClass::Graph => ch.is_ascii_graphic(),
        NamedClass::Lower => ch.is_ascii_lowercase(),
        NamedClass::Print => !ch.is_ascii_control(),
        NamedClass::Punct => ch.is_ascii_punctuation(),
        NamedClass::Space => ch.is_ascii_whitespace(),
        NamedClass::Upper => ch.is_ascii_uppercase(),
        NamedClass::Xdigit => ch.is_ascii_hexdigit(),
    }
}
