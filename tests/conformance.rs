//! Core structural and leaf cases.
//!
//! These cover the value-model cases for the core kinds:
//! object and array recursion, key order independence, loose coercion, scalar
//! leaves, the infinities, NaN, signed zero, regex source and flags, dates, and
//! cross-type inequality.

mod common;

use common::*;
use deep_equal::Value;

#[test]
fn equal_objects() {
    // two equal objects
    check_both(
        "two equal objects",
        &obj(vec![
            ("a", arr(vec![n(2.0), n(3.0)])),
            ("b", arr(vec![n(4.0)])),
        ]),
        &obj(vec![
            ("a", arr(vec![n(2.0), n(3.0)])),
            ("b", arr(vec![n(4.0)])),
        ]),
        true,
        true,
    );

    // two equal objects in different key order
    check_both(
        "two equal objects, different order",
        &obj(vec![
            ("a", arr(vec![n(2.0), n(3.0)])),
            ("b", arr(vec![n(4.0)])),
        ]),
        &obj(vec![
            ("b", arr(vec![n(4.0)])),
            ("a", arr(vec![n(2.0), n(3.0)])),
        ]),
        true,
        true,
    );

    // loosely equal, strictly inequal: "4" vs 4
    check_both(
        "loosely equal, strictly inequal objects",
        &obj(vec![("a", n(2.0)), ("b", s("4"))]),
        &obj(vec![("a", n(2.0)), ("b", n(4.0))]),
        true,
        false,
    );

    // different key set
    check_both(
        "two inequal objects",
        &obj(vec![("a", n(2.0)), ("b", n(4.0))]),
        &obj(vec![("a", n(2.0)), ("B", n(4.0))]),
        false,
        false,
    );

    // "-000" and false
    check_both("`false` and `\"-000\"`", &s("-000"), &b(false), true, false);
}

#[test]
fn not_equal() {
    check_both(
        "two inequal objects",
        &obj(vec![("x", n(5.0)), ("y", arr(vec![n(6.0)]))]),
        &obj(vec![("x", n(5.0)), ("y", n(6.0))]),
        false,
        false,
    );
}

#[test]
fn nested_nulls() {
    check(
        "same-length arrays of nulls",
        &arr(vec![NULL, NULL, NULL]),
        &arr(vec![NULL, NULL, NULL]),
        true,
        true,
        true,
    );
}

#[test]
fn strings_vs_numbers() {
    check_both(
        "objects with equivalent string/number values",
        &arr(vec![obj(vec![("a", n(3.0))]), obj(vec![("b", n(4.0))])]),
        &arr(vec![obj(vec![("a", s("3"))]), obj(vec![("b", s("4"))])]),
        true,
        false,
    );
}

#[test]
fn non_objects() {
    check("same numbers", &n(3.0), &n(3.0), true, true, true);
    check("same strings", &s("beep"), &s("beep"), true, true, true);
    check_both("numeric string and number", &s("3"), &n(3.0), true, false);
    check_both(
        "numeric string and array containing number",
        &s("3"),
        &arr(vec![n(3.0)]),
        false,
        false,
    );
    check_both(
        "number and array containing number",
        &n(3.0),
        &arr(vec![n(3.0)]),
        false,
        false,
    );
}

#[test]
fn infinities() {
    check(
        "inf and inf",
        &n(f64::INFINITY),
        &n(f64::INFINITY),
        true,
        true,
        true,
    );
    check(
        "neg inf and neg inf",
        &n(f64::NEG_INFINITY),
        &n(f64::NEG_INFINITY),
        true,
        true,
        true,
    );
    check_both(
        "inf and neg inf",
        &n(f64::INFINITY),
        &n(f64::NEG_INFINITY),
        false,
        false,
    );
}

#[test]
fn booleans() {
    check_both("trues", &b(true), &b(true), true, true);
    check_both("falses", &b(false), &b(false), true, true);
    check_both("true and false", &b(true), &b(false), false, false);
}

#[test]
fn booleans_and_arrays() {
    check_both("true and empty array", &b(true), &arr(vec![]), false, false);
    check_both(
        "false and empty array",
        &b(false),
        &arr(vec![]),
        false,
        false,
    );
}

#[test]
fn arrays_with_mixed_contents() {
    let mixed = || {
        arr(vec![
            UNDEF,
            NULL,
            n(-1.0),
            n(0.0),
            n(1.0),
            b(false),
            b(true),
            UNDEF,
            s(""),
            s("abc"),
            NULL,
            UNDEF,
        ])
    };
    check(
        "arrays with equal contents",
        &mixed(),
        &mixed(),
        true,
        true,
        true,
    );
}

#[test]
fn null_and_undefined() {
    check_both("null and undefined", &NULL, &UNDEF, true, false);
    check_both(
        "[null] and [undefined]",
        &arr(vec![NULL]),
        &arr(vec![UNDEF]),
        true,
        false,
    );
}

#[test]
fn nans() {
    // The module's own behavior: loose NaN vs NaN is false, strict is true.
    check_both("two NaNs", &n(f64::NAN), &n(f64::NAN), false, true);
    check_both(
        "two objects with a NaN value",
        &obj(vec![("a", n(f64::NAN))]),
        &obj(vec![("a", n(f64::NAN))]),
        false,
        true,
    );
    check_both("NaN and 1", &n(f64::NAN), &n(1.0), false, false);
}

#[test]
fn zeroes() {
    check_both("0 and -0", &n(0.0), &n(-0.0), true, false);
    check_both(
        "objects with a same-keyed 0/-0 value",
        &obj(vec![("a", n(0.0))]),
        &obj(vec![("a", n(-0.0))]),
        true,
        false,
    );
}

#[test]
fn object_literals_prototype_key() {
    check_both(
        "loosely equal, strictly inequal prototype properties",
        &obj(vec![("prototype", n(2.0))]),
        &obj(vec![("prototype", s("2"))]),
        true,
        false,
    );
}

#[test]
fn arrays_and_objects() {
    check_both(
        "empty array and empty object",
        &arr(vec![]),
        &obj(vec![]),
        false,
        false,
    );
    check_both(
        "empty array and empty arraylike object",
        &arr(vec![]),
        &obj(vec![("length", n(0.0))]),
        false,
        false,
    );
    check_both(
        "array and similar object",
        &arr(vec![n(1.0)]),
        &obj(vec![("0", n(1.0))]),
        false,
        false,
    );
}

#[test]
fn object_and_null() {
    check_both("null and an object", &obj(vec![]), &NULL, false, false);
}

#[test]
fn regexes_vs_dates() {
    check_both(
        "Date and RegExp",
        &date(1387585278000.0),
        &regex("abc", ""),
        false,
        false,
    );
}

#[test]
fn regexen() {
    check_both(
        "two different regexes",
        &regex("abc", ""),
        &regex("xyz", ""),
        false,
        false,
    );
    check(
        "two abc regexes",
        &regex("abc", ""),
        &regex("abc", ""),
        true,
        true,
        false,
    );
    check(
        "two xyz regexes",
        &regex("xyz", ""),
        &regex("xyz", ""),
        true,
        true,
        false,
    );
    check_both(
        "same source, different flags",
        &regex("abc", "g"),
        &regex("abc", "i"),
        false,
        false,
    );
    // Flag order does not matter: gi and ig are the same set.
    check_both(
        "flag order independence",
        &regex("abc", "gi"),
        &regex("abc", "ig"),
        true,
        true,
    );
}

#[test]
fn dates() {
    check_both(
        "two Dates with the same timestamp",
        &date(1387585278000.0),
        &date(1387585278000.0),
        true,
        true,
    );
    check_both(
        "two inequal Dates",
        &date(946684800000.0),
        &date(978307200000.0),
        false,
        false,
    );
    // Two Invalid Dates (NaN timestamp) never match, because getTime is NaN.
    check_both(
        "two Invalid Dates",
        &date(f64::NAN),
        &date(f64::NAN),
        false,
        false,
    );
}

#[test]
fn deep_structures_terminate() {
    // Cycle behavior with true self-references lives in cycles.rs. Here we
    // confirm deep finite nesting compares structurally and does not blow up.
    let a = nested(64);
    let b = nested(64);
    check_both("deeply nested equal objects", &a, &b, true, true);

    let diverge = obj(vec![("a", n(1.0)), ("b", n(1.0))]);
    let nested_b = obj(vec![("a", n(1.0)), ("b", nested(4))]);
    check_both(
        "deep object vs leaf at the same key",
        &nested_b,
        &diverge,
        false,
        false,
    );
}

/// Build a finite nested object of the given depth.
fn nested(depth: usize) -> Value {
    if depth == 0 {
        obj(vec![("a", n(1.0))])
    } else {
        obj(vec![("a", n(1.0)), ("b", nested(depth - 1))])
    }
}
