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
    more(m, node, min, max, pos, 0, k)
}

fn more<'t>(
    m: &mut Matcher<'t>,
    node: &Node,
    min: u32,
    max: Option<u32>,
    pos: usize,
    count: u32,
    k: Cont<'_, 't>,
) -> bool {
    let room = max.is_none_or(|mx| count < mx);
    if room {
        let ok = m.run(node, pos, &mut |m, np| {
            if np == pos {
                // One empty iteration counts toward min and does not chain.
                // Once min is already met, refuse it so a trailing empty does
                // not replace the last non-empty capture.
                if count >= min.max(1) {
                    return false;
                }
                let need = min.max(count + 1);
                if max.is_some_and(|mx| need > mx) {
                    return false;
                }
                return k(m, np);
            }
            more(m, node, min, max, np, count + 1, k)
        });
        if ok {
            return true;
        }
    }
    count >= min && k(m, pos)
}

#[cfg(test)]
mod tests {
    use super::super::ast::Node::{Char, Concat, Empty, Group, Repeat};
    use super::super::ast::Node;
    use super::super::matcher::Matcher;

    fn rep(node: Node, min: u32, max: Option<u32>) -> Node {
        Repeat { node: Box::new(node), min, max }
    }

    fn end(node: &Node, text: &str) -> Option<(usize, Option<(usize, usize)>)> {
        let mut m = Matcher::new(text, false, false, 1);
        let ok = m.run(node, 0, &mut |mm, p| {
            mm.caps[0] = Some((0, p));
            true
        });
        ok.then(|| (m.caps[0].unwrap().1, m.caps[1]))
    }

    #[test]
    fn greedy_and_empty() {
        assert_eq!(end(&rep(Char('a'), 0, None), "aaa").unwrap().0, 3);
        assert_eq!(end(&rep(Char('a'), 0, None), "baaa").unwrap().0, 0);
        assert!(end(&rep(Char('a'), 1, None), "").is_none());
        assert_eq!(end(&rep(Char('a'), 0, Some(1)), "aaa").unwrap().0, 1);
        assert_eq!(end(&rep(Char('é'), 0, None), "éé").unwrap().0, "éé".len());
        let star_a = Concat(vec![rep(Char('a'), 0, None), Char('a')]);
        assert_eq!(end(&star_a, "aaa").unwrap().0, 3);
        let bounded = rep(Char('a'), 2, Some(3));
        assert_eq!(end(&bounded, "aaaa").unwrap().0, 3);
        assert_eq!(end(&bounded, "aa").unwrap().0, 2);
        let leave = Concat(vec![rep(Char('a'), 2, Some(3)), Char('a')]);
        assert_eq!(end(&leave, "aaa").unwrap().0, 3);
        assert_eq!(end(&leave, "aaaa").unwrap().0, 4);
        let g = rep(Group(Box::new(rep(Char('a'), 0, None)), 1), 0, None);
        assert_eq!(end(&g, "aaa").unwrap(), (3, Some((0, 3))));
        assert_eq!(end(&g, "b").unwrap(), (0, Some((0, 0))));
        assert_eq!(end(&g, "").unwrap(), (0, Some((0, 0))));
        let gb = Concat(vec![g, Char('b')]);
        assert_eq!(end(&gb, "b").unwrap(), (1, Some((0, 0))));
        assert_eq!(end(&gb, "aaab").unwrap(), (4, Some((0, 3))));
        let two = rep(Group(Box::new(rep(Char('a'), 0, None)), 1), 2, Some(2));
        assert_eq!(end(&two, "aaa").unwrap(), (3, Some((3, 3))));
        assert_eq!(end(&two, "a").unwrap(), (1, Some((1, 1))));
        let empty2 = rep(Group(Box::new(Empty), 1), 2, Some(2));
        assert_eq!(end(&empty2, "x").unwrap(), (0, Some((0, 0))));
        let nested = rep(rep(Char('a'), 0, None), 0, None);
        assert_eq!(end(&nested, "aaa").unwrap().0, 3);
        let star_then = Concat(vec![
            rep(Group(Box::new(rep(Char('a'), 0, None)), 1), 0, None),
            Char('a'),
        ]);
        assert_eq!(end(&star_then, "aaa").unwrap(), (3, Some((0, 2))));
    }
}
