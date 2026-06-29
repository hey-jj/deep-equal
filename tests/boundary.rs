//! Parity boundary.
//!
//! The deep-equality algorithm spends most of its code recovering JavaScript type
//! information that Rust keeps in the type system: boxed primitives, arguments
//! objects, Symbol.toStringTag spoofing, prototype-chain comparison, and
//! duck-typed fakes. None of these have a representation in the value model, so
//! they are out of scope. This file records that boundary and asserts the
//! chosen behavior for the cases that do have a value-model answer.

mod common;

use common::*;
use deep_equal::Value;

// Cases below have no value-model analog and are intentionally not modeled.
// They are listed so the boundary is explicit:
//
// - Boxed primitives: Object(3) vs 3. The model has no boxed wrapper, so a
//   number is always just a number.
// - Arguments objects vs arrays. The model has no arguments brand.
// - Symbol.toStringTag spoofing. The model derives the brand from the variant,
//   so it cannot be faked.
// - Prototype-chain comparison in strict mode. The model has no prototype.
// - Duck-typed fakes: fake buffer, fake array, fake Date, fake RegExp, fake
//   Error. The model uses real variants, so there is nothing to fake.
// - WeakMap and WeakSet. Their contents are unenumerable, so the algorithm
//   reports same-brand instances as equal. The model does not include them.
// - Functions and getters. The model has no callable or accessor kind.
// - Symbols as Set members or Map keys. The model has no symbol kind.

#[test]
fn brand_distinguishes_container_kinds() {
    // The brand check that backs Symbol.toStringTag in JavaScript is derived
    // from the variant here, so distinct kinds never compare equal.
    let pairs: [(Value, Value); 3] = [
        (set(vec![]), map(vec![])),
        (Value::Date(0.0), regex("abc", "")),
        (arr(vec![]), obj(vec![])),
    ];
    for (a, b) in &pairs {
        assert!(!deep_equal::deep_equal_loose(a, b));
        assert!(!deep_equal::deep_equal_strict(a, b));
    }
}

#[test]
fn array_vs_arraylike_object() {
    // [1] vs { "0": 1 } is the value-model analog of array-vs-arraylike. The
    // array class differs, so they are not equal.
    check_both(
        "array and arraylike object",
        &arr(vec![n(1.0)]),
        &obj(vec![("0", n(1.0))]),
        false,
        false,
    );
}
