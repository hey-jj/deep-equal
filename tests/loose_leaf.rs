//! Loose and strict leaf coercion.
//!
//! Loose comparison reproduces the ECMAScript `==` operator over leaf kinds.
//! Strict comparison reproduces `Object.is`. These cases pin the exact
//! coercions the suite relies on.

mod common;

use common::*;
use deep_equal::{deep_equal_loose, deep_equal_strict, Value};

fn loose(a: &Value, b: &Value) -> bool {
    deep_equal_loose(a, b)
}

fn strict(a: &Value, b: &Value) -> bool {
    deep_equal_strict(a, b)
}

#[test]
fn numeric_string_and_number() {
    assert!(loose(&s("3"), &n(3.0)));
    assert!(!strict(&s("3"), &n(3.0)));
}

#[test]
fn minus_zeros_string_and_false() {
    // "-000" coerces to -0, false coerces to 0, and -0 == 0.
    assert!(loose(&s("-000"), &b(false)));
    assert!(!strict(&s("-000"), &b(false)));
}

#[test]
fn plus_zeros_string_and_two() {
    // "+000" coerces to 0, which does not equal 2.
    assert!(!loose(&s("+000"), &n(2.0)));
    assert!(!strict(&s("+000"), &n(2.0)));
}

#[test]
fn null_and_undefined_cross() {
    assert!(loose(&NULL, &UNDEF));
    assert!(!strict(&NULL, &UNDEF));
    // null and undefined are loosely equal to nothing else.
    assert!(!loose(&NULL, &n(0.0)));
    assert!(!loose(&UNDEF, &n(0.0)));
    assert!(!loose(&NULL, &s("")));
    assert!(!loose(&NULL, &b(false)));
}

#[test]
fn nan_pairs() {
    // Loose NaN vs NaN is false; strict is true.
    assert!(!loose(&n(f64::NAN), &n(f64::NAN)));
    assert!(strict(&n(f64::NAN), &n(f64::NAN)));
}

#[test]
fn signed_zero() {
    assert!(loose(&n(0.0), &n(-0.0)));
    assert!(!strict(&n(0.0), &n(-0.0)));
}

#[test]
fn boolean_coercions() {
    assert!(loose(&b(true), &n(1.0)));
    assert!(!strict(&b(true), &n(1.0)));
    assert!(loose(&b(false), &s("")));
    assert!(loose(&b(false), &n(0.0)));
    assert!(loose(&b(true), &s("1")));
}

#[test]
fn same_strings_and_numbers() {
    assert!(loose(&s("beep"), &s("beep")));
    assert!(strict(&s("beep"), &s("beep")));
    assert!(!loose(&n(2.0), &s("4")));
    assert!(!strict(&n(2.0), &s("4")));
}

#[test]
fn whitespace_and_radix_strings() {
    // Number(" 3 ") is 3, Number("0x10") is 16, Number("") is 0.
    assert!(loose(&s("  3  "), &n(3.0)));
    assert!(loose(&s("0x10"), &n(16.0)));
    assert!(loose(&s(""), &n(0.0)));
    // A non-numeric string coerces to NaN and matches nothing numerically.
    assert!(!loose(&s("abc"), &n(0.0)));
}

#[test]
fn infinity_strings() {
    assert!(loose(&s("Infinity"), &n(f64::INFINITY)));
    assert!(loose(&s("-Infinity"), &n(f64::NEG_INFINITY)));
}
