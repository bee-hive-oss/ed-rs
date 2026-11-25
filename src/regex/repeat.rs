//! Backtracking for `Node::Repeat`.

#![allow(dead_code)]

use super::ast::Node;
use super::matcher::{Cont, Matcher};

/// Match `node` between `min` and `max` times (None for no upper bound) at
/// `pos`, then call `k`.
///
/// Greedy: try more repeats first, then fewer. Call `Matcher::run` for each
/// repeat of `node`. Stop looping when a repeat consumes nothing, so `\(a*\)*`
/// cannot spin forever; that one empty repeat still counts toward `min`.
pub fn run_repeat<'t>(
    m: &mut Matcher<'t>,
    node: &Node,
    min: u32,
    max: Option<u32>,
    pos: usize,
    k: Cont<'_, 't>,
) -> bool {
    let _ = (m, node, min, max, pos, k);
    todo!("greedy bounded repeat with backtracking")
}
