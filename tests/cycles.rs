//! Recursion depth and termination.
//!
//! A `Value` owns its children, so it is always a finite tree. A self-reference
//! cannot be constructed. These tests confirm the comparator terminates on deep
//! finite structures and decides them correctly.

mod common;

use common::*;
use deep_equal::Value;

/// Build a finite chain of nested objects of the given depth.
fn chain(depth: usize) -> Value {
    let mut v = obj(vec![("a", n(1.0))]);
    for _ in 0..depth {
        v = obj(vec![("a", n(1.0)), ("b", v)]);
    }
    v
}

#[test]
fn deep_equal_chains_terminate() {
    let a = chain(1000);
    let b = chain(1000);
    check_both("deep equal chains", &a, &b, true, true);
}

#[test]
fn deep_chains_diverge_at_leaf() {
    // One side keeps recursing, the other holds a leaf where the cycle would
    // close. They are not equal.
    let recursing = chain(500);
    let leaf_at_end = obj(vec![("a", n(1.0)), ("b", n(1.0))]);
    check_both("chain vs leaf", &recursing, &leaf_at_end, false, false);
}

#[test]
fn deep_chains_diverge_at_depth() {
    let a = chain(64);
    // Same shape but a different leaf deep inside.
    let mut b = obj(vec![("a", n(2.0))]);
    for _ in 0..64 {
        b = obj(vec![("a", n(1.0)), ("b", b)]);
    }
    check_both("chains differ deep inside", &a, &b, false, false);
}
