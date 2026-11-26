//! Syntax tree shared by the regex parsers and the matcher. Types only.

#![allow(dead_code)]

/// POSIX bracket class name, as in `[[:alpha:]]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedClass {
    Alnum,
    Alpha,
    Blank,
    Cntrl,
    Digit,
    Graph,
    Lower,
    Print,
    Punct,
    Space,
    Upper,
    Xdigit,
}

/// One item inside a bracket expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassItem {
    Ch(char),
    /// Inclusive range `a-z`.
    Range(char, char),
    Named(NamedClass),
}

/// A parsed bracket expression. `negate` is true for `[^...]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassSet {
    pub negate: bool,
    pub items: Vec<ClassItem>,
}

/// One node of a parsed pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// Matches the empty string.
    Empty,
    Char(char),
    /// `.`
    Any,
    Set(ClassSet),
    /// `^` as an anchor.
    Bol,
    /// `$` as an anchor.
    Eol,
    /// A capture group. The number is 1-based, in order of the open paren.
    Group(Box<Node>, usize),
    /// `\1` to `\9`.
    Backref(usize),
    /// `*`, `\{m,n\}`, `+`, `?`. `max` None means no upper bound.
    Repeat {
        node: Box<Node>,
        min: u32,
        max: Option<u32>,
    },
    Concat(Vec<Node>),
    /// Alternatives, tried left to right.
    Alt(Vec<Node>),
}

/// A parsed pattern and its group count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub root: Node,
    pub nsub: usize,
}

/// Capture offsets. Slot 0 is the whole match. Offsets are byte offsets.
pub type Captures = Vec<Option<(usize, usize)>>;
