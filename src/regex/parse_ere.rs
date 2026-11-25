//! Parser for POSIX extended regular expressions (the -E flag).

#![allow(dead_code)]

use super::ast::Node::{Alt, Any, Backref, Bol, Char, Concat, Empty, Eol, Group, Repeat, Set};
use super::ast::{Node, Program};
use super::class::parse_bracket;
use super::interval::parse_interval;
use super::RegexError::{self, *};

/// Parse an ERE into a [`Program`].
///
/// Syntax to support: literals; `.`; `[...]` through
/// `class::parse_bracket`; `*`, `+`, `?`; `{m,n}` through
/// `interval::parse_interval`; `( )` groups numbered by open paren; `|`
/// alternation; `^` and `$` always anchors; `\1` to `\9` back references;
/// and a backslash before any other character is that character.
///
/// Errors: TrailingBackslash, UnmatchedParen, UnmatchedCloseParen,
/// BadBackref, BadRepeat (a repeat operator with nothing before it), and
/// whatever the bracket and interval parsers return.
pub fn parse_ere(pattern: &str) -> Result<Program, RegexError> {
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

fn quantifier(ch: char) -> bool { matches!(ch, '*' | '+' | '?' | '{') }

fn can_repeat(node: &Node) -> bool { !matches!(node, Bol | Eol | Empty) }

impl P {
    fn alt(&mut self, depth: usize) -> Result<Node, RegexError> {
        let mut arms = Vec::new();
        loop {
            arms.push(self.branch(depth)?);
            if self.i >= self.c.len() || self.c[self.i] != '|' { break; }
            self.i += 1;
        }
        Ok(pack(arms, true))
    }

    fn branch(&mut self, depth: usize) -> Result<Node, RegexError> {
        let mut nodes = Vec::new();
        while self.i < self.c.len() {
            let ch = self.c[self.i];
            if ch == '|' || (depth > 0 && ch == ')') { break; }
            if quantifier(ch) {
                let Some(prev) = nodes.pop() else { return Err(BadRepeat) };
                if !can_repeat(&prev) { return Err(BadRepeat); }
                nodes.push(self.quant(prev)?);
                continue;
            }
            if ch == '^' { nodes.push(Bol); self.i += 1; continue; }
            if ch == '$' { nodes.push(Eol); self.i += 1; continue; }
            nodes.push(self.atom(depth)?);
        }
        Ok(pack(nodes, false))
    }

    fn quant(&mut self, node: Node) -> Result<Node, RegexError> {
        let ch = self.c[self.i];
        self.i += 1;
        if ch == '*' { return Ok(rep(node, 0, None)); }
        if ch == '+' { return Ok(rep(node, 1, None)); }
        if ch == '?' { return Ok(rep(node, 0, Some(1))); }
        let (min, max, next) = parse_interval(&self.c, self.i, true)?;
        self.i = next;
        Ok(rep(node, min, max))
    }

    fn atom(&mut self, depth: usize) -> Result<Node, RegexError> {
        let ch = self.c[self.i];
        if ch == '(' { return self.group(depth); }
        if ch == '.' { self.i += 1; return Ok(Any); }
        if ch == '[' {
            let (set, next) = parse_bracket(&self.c, self.i + 1)?;
            self.i = next;
            return Ok(Set(set));
        }
        if ch == '\\' {
            let Some(n) = self.c.get(self.i + 1).copied() else { return Err(TrailingBackslash) };
            self.i += 2;
            if let Some(k) = backref_num(n) {
                if !self.done[k] { return Err(BadBackref); }
                return Ok(Backref(k));
            }
            return Ok(Char(n));
        }
        self.i += 1;
        Ok(Char(ch))
    }

    fn group(&mut self, depth: usize) -> Result<Node, RegexError> {
        self.i += 1;
        self.nsub += 1;
        let n = self.nsub;
        let inner = self.alt(depth + 1)?;
        if self.i >= self.c.len() || self.c[self.i] != ')' { return Err(UnmatchedParen); }
        self.i += 1;
        if n < 10 { self.done[n] = true; }
        Ok(Group(Box::new(inner), n))
    }
}

fn backref_num(ch: char) -> Option<usize> {
    match ch {
        '1'..='9' => Some((ch as u8 - b'0') as usize),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_ere, Alt, BadBackref, BadRepeat, Concat, Group, Repeat, TrailingBackslash, UnmatchedParen};

    fn root(pat: &str) -> super::Node {
        parse_ere(pat).unwrap_or_else(|e| panic!("{pat}: {e:?}")).root
    }

    #[test]
    fn syntax_and_errors() {
        assert!(matches!(root("a|b"), Alt(_)));
        assert!(matches!(root("a+"), Repeat { min: 1, max: None, .. }));
        assert!(matches!(root("a?"), Repeat { min: 0, max: Some(1), .. }));
        assert!(matches!(root("a*"), Repeat { min: 0, max: None, .. }));
        assert!(matches!(root("(a)*"), Repeat { .. }));
        assert!(matches!(root("()"), Group(_, 1)));
        assert!(matches!(root("^a$"), Concat(_)));
        assert_eq!(parse_ere("(a)\\1").unwrap().nsub, 1);
        assert_eq!(parse_ere("*").unwrap_err(), BadRepeat);
        assert_eq!(parse_ere("a|+").unwrap_err(), BadRepeat);
        assert_eq!(parse_ere("(").unwrap_err(), UnmatchedParen);
        assert_eq!(parse_ere("\\").unwrap_err(), TrailingBackslash);
        assert_eq!(parse_ere("\\1").unwrap_err(), BadBackref);
    }
}
