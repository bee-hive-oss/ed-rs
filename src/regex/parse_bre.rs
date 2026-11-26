//! Parser for POSIX basic regular expressions, with the GNU extensions
//! glibc gives `regcomp` without REG_EXTENDED.

#![allow(dead_code)]

use super::ast::Node::{Alt, Any, Backref, Bol, Char, Concat, Empty, Eol, Group, Repeat, Set};
use super::ast::{Node, Program};
use super::bre_escape::{classify_escape, BreEscape};
use super::class::parse_bracket;
use super::interval::parse_interval;
use super::RegexError::{self, *};

/// Parse a BRE into a [`Program`].
///
/// Syntax to support: literals; `.`; `[...]` through
/// `class::parse_bracket`; `*`; `\{m,n\}` through `interval::parse_interval`;
/// `\(` `\)` groups numbered by open paren; `\1` to `\9` back references;
/// `^` as an anchor only at the start of the pattern or after `\(` or `\|`,
/// and `$` only at the end or before `\)` or `\|` (elsewhere they are
/// literals); `*` first in the pattern or right after `\(` or `^` is a
/// literal; and the GNU operators `\+`, `\?`, `\|`. A backslash before any
/// other character is that character. `Node::Empty` stands for an empty
/// branch. Classify every backslash pair with
/// `bre_escape::classify_escape`.
///
/// Errors: TrailingBackslash, UnmatchedParen, UnmatchedCloseParen,
/// BadBackref (a number above the groups closed so far), BadRepeat, and
/// whatever the bracket and interval parsers return.
pub fn parse_bre(pattern: &str) -> Result<Program, RegexError> {
    let mut p = P { c: pattern.chars().collect(), i: 0, nsub: 0, done: [false; 10] };
    Ok(Program { root: p.alt(0)?, nsub: p.nsub })
}

struct P { c: Vec<char>, i: usize, nsub: usize, done: [bool; 10] }

fn pack(mut xs: Vec<Node>, alt: bool) -> Node {
    if xs.len() <= 1 { return xs.pop().unwrap_or(Empty); }
    if alt { Alt(xs) } else { Concat(xs) }
}

fn rep(node: Node, min: u32, max: Option<u32>) -> Node {
    Repeat { node: Box::new(node), min, max }
}

impl P {
    // Pass n to classify_escape only when group n is already closed.
    fn closed_for(&self, pos: usize) -> usize {
        let Some(d @ '1'..='9') = self.c.get(pos).copied() else { return 0 };
        let n = (d as u8 - b'0') as usize;
        usize::from(self.done[n]) * n
    }

    fn peek(&self) -> Result<Option<BreEscape>, RegexError> {
        if self.i >= self.c.len() || self.c[self.i] != '\\' { return Ok(None); }
        let n = self.closed_for(self.i + 1);
        Ok(Some(classify_escape(&self.c, self.i + 1, n)?))
    }

    fn eat2(&mut self, node: Node) -> Node { self.i += 2; node }
    fn alt(&mut self, depth: usize) -> Result<Node, RegexError> {
        let mut arms = Vec::new();
        loop {
            arms.push(self.branch(depth)?);
            if !matches!(self.peek()?, Some(BreEscape::Alt)) { break; }
            self.i += 2;
        }
        Ok(pack(arms, true))
    }

    fn branch(&mut self, depth: usize) -> Result<Node, RegexError> {
        let mut nodes = Vec::new();
        let mut after_bol = false;
        while self.i < self.c.len() {
            if self.c[self.i] == '$' && self.dollar(depth)? {
                nodes.push(Eol); self.i += 1; after_bol = false; continue;
            }
            let pk = self.peek()?;
            let stop = matches!(pk, Some(BreEscape::Alt))
                || (depth > 0 && matches!(pk, Some(BreEscape::GroupClose)));
            if stop { break; }
            if nodes.is_empty() && self.c[self.i] == '^' {
                nodes.push(Bol); self.i += 1; after_bol = true; continue;
            }
            let lit = nodes.is_empty() || after_bol;
            let atom = self.atom(depth, lit)?;
            nodes.push(self.quants(atom)?);
            after_bol = false;
        }
        Ok(pack(nodes, false))
    }
    fn dollar(&self, depth: usize) -> Result<bool, RegexError> {
        let j = self.i + 1;
        if j >= self.c.len() { return Ok(true); }
        if self.c[j] != '\\' { return Ok(false); }
        let esc = classify_escape(&self.c, j + 1, self.closed_for(j + 1))?;
        Ok(matches!(esc, BreEscape::Alt) || (depth > 0 && matches!(esc, BreEscape::GroupClose)))
    }

    fn atom(&mut self, depth: usize, star_lit: bool) -> Result<Node, RegexError> {
        let ch = self.c[self.i];
        if ch == '*' { self.i += 1; return if star_lit { Ok(Char('*')) } else { Err(BadRepeat) }; }
        if ch == '.' { self.i += 1; return Ok(Any); }
        if ch == '[' {
            let (set, next) = parse_bracket(&self.c, self.i + 1)?;
            self.i = next; return Ok(Set(set));
        }
        if ch != '\\' { self.i += 1; return Ok(Char(ch)); }
        Ok(match self.peek()? {
            Some(BreEscape::GroupOpen) => self.group(depth)?,
            Some(BreEscape::GroupClose) => return Err(UnmatchedCloseParen),
            Some(BreEscape::Plus) => self.eat2(Char('+')),
            Some(BreEscape::Question) => self.eat2(Char('?')),
            Some(BreEscape::Interval) => return Err(BadRepeat),
            Some(BreEscape::Backref(n)) => self.eat2(Backref(n)),
            Some(BreEscape::Literal(c)) => self.eat2(Char(c)),
            Some(BreEscape::Alt) | None => return Err(BadPattern),
        })
    }

    fn group(&mut self, depth: usize) -> Result<Node, RegexError> {
        self.i += 2; self.nsub += 1;
        let n = self.nsub;
        let inner = self.alt(depth + 1)?;
        if !matches!(self.peek()?, Some(BreEscape::GroupClose)) { return Err(UnmatchedParen); }
        self.i += 2;
        if n < 10 { self.done[n] = true; }
        Ok(Group(Box::new(inner), n))
    }

    fn quants(&mut self, mut node: Node) -> Result<Node, RegexError> {
        loop {
            if self.i >= self.c.len() { return Ok(node); }
            let repeated = matches!(node, Repeat { .. });
            if self.c[self.i] == '*' {
                if repeated { return Err(BadRepeat); }
                self.i += 1; node = rep(node, 0, None); continue;
            }
            node = match self.peek()? {
                Some(BreEscape::Plus) => self.eat2(rep(node, 1, None)),
                Some(BreEscape::Question) => self.eat2(rep(node, 0, Some(1))),
                Some(BreEscape::Interval) if !repeated => {
                    let (min, max, next) = parse_interval(&self.c, self.i + 2, false)?;
                    self.i = next; rep(node, min, max)
                }
                Some(BreEscape::Interval) => return Err(BadRepeat),
                _ => return Ok(node),
            };
        }
    }
}
