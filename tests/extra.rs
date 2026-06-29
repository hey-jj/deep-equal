//! Composite and edge cases beyond the per-kind blocks.
//!
//! Composite nesting, empty cross-type pairs, options defaulting, signed zero
//! and NaN buried deep, and Invalid Date.

mod common;

use common::*;
use deep_equal::{deep_equal, deep_equal_loose, deep_equal_strict, Options, Value};

#[test]
fn options_default_is_loose() {
    // Options::default() and Options::LOOSE behave the same.
    let a = obj(vec![("k", s("3"))]);
    let b = obj(vec![("k", n(3.0))]);
    assert_eq!(
        deep_equal(&a, &b, Options::default()),
        deep_equal(&a, &b, Options::LOOSE)
    );
    assert!(deep_equal(&a, &b, Options::default()));
    assert!(deep_equal_loose(&a, &b));
    assert!(!deep_equal_strict(&a, &b));
}

#[test]
fn composite_nesting() {
    // A Map inside an Object inside an Array round-trips equal when structurally
    // equal and unequal when a deep leaf differs.
    let build = |leaf: Value| {
        arr(vec![obj(vec![(
            "m",
            map(vec![(s("k"), set(vec![n(1.0), leaf]))]),
        )])])
    };
    check_both(
        "composite equal",
        &build(n(2.0)),
        &build(n(2.0)),
        true,
        true,
    );
    check_both(
        "composite leaf differs",
        &build(n(2.0)),
        &build(n(3.0)),
        false,
        false,
    );
}

#[test]
fn empty_cross_type_matrix() {
    let kinds = [
        ("object", obj(vec![])),
        ("array", arr(vec![])),
        ("map", map(vec![])),
        ("set", set(vec![])),
    ];
    for (i, (na, va)) in kinds.iter().enumerate() {
        for (j, (nb, vb)) in kinds.iter().enumerate() {
            let name = format!("{na} vs {nb}");
            if i == j {
                check_both(&name, va, vb, true, true);
            } else {
                check_both(&name, va, vb, false, false);
            }
        }
    }
}

#[test]
fn signed_zero_buried_deep() {
    let a = obj(vec![("a", obj(vec![("b", arr(vec![n(-0.0)]))]))]);
    let b = obj(vec![("a", obj(vec![("b", arr(vec![n(0.0)]))]))]);
    check_both("deep -0 vs +0", &a, &b, true, false);
}

#[test]
fn nan_buried_deep() {
    let a = obj(vec![("a", obj(vec![("b", arr(vec![n(f64::NAN)]))]))]);
    let b = obj(vec![("a", obj(vec![("b", arr(vec![n(f64::NAN)]))]))]);
    check_both("deep NaN vs NaN", &a, &b, false, true);
}

#[test]
fn invalid_date() {
    // Two Invalid Dates never match because getTime returns NaN.
    check_both(
        "two Invalid Dates",
        &Value::Date(f64::NAN),
        &Value::Date(f64::NAN),
        false,
        false,
    );
}

#[test]
fn strict_implies_loose() {
    // If two values are strictly equal, they are loosely equal. Check on a
    // curated set spanning the value kinds.
    let cases: Vec<Value> = vec![
        n(1.0),
        s("x"),
        b(true),
        NULL,
        UNDEF,
        arr(vec![n(1.0), s("y")]),
        obj(vec![("a", n(1.0))]),
        map(vec![(s("k"), n(2.0))]),
        set(vec![n(1.0), n(2.0)]),
        Value::Date(1000.0),
        regex("abc", "g"),
    ];
    for v in &cases {
        if deep_equal_strict(v, v) {
            assert!(deep_equal_loose(v, v), "strict implies loose for {v:?}");
        }
    }
}

#[test]
fn reflexive_on_clones() {
    let v = obj(vec![
        ("nums", arr(vec![n(1.0), n(-1.0), n(f64::INFINITY)])),
        ("nested", map(vec![(s("k"), set(vec![s("a"), s("b")]))])),
    ]);
    let clone = v.clone();
    assert!(deep_equal_loose(&v, &clone));
    assert!(deep_equal_strict(&v, &clone));
}
