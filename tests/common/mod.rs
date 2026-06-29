//! Shared test harness.
//!
//! Every case runs four ways: loose forward, loose reversed, strict forward,
//! strict reversed. Each row encodes the expected loose result and strict
//! result. Reversed checks enforce symmetry. This module provides the contract
//! so each case file stays small.

#![allow(dead_code)]

use deep_equal::{deep_equal, Options, TaKind, Value};

/// Assert a case in both modes and both directions.
///
/// `loose` and `strict` are the expected results. When `skip_reversed` is
/// false, the reversed pair is checked too, which proves symmetry.
pub fn check(name: &str, a: &Value, b: &Value, loose: bool, strict: bool, skip_reversed: bool) {
    assert_eq!(deep_equal(a, b, Options::LOOSE), loose, "loose {name}");
    assert_eq!(deep_equal(a, b, Options::STRICT), strict, "strict {name}");
    if !skip_reversed {
        assert_eq!(
            deep_equal(b, a, Options::LOOSE),
            loose,
            "loose reversed {name}"
        );
        assert_eq!(
            deep_equal(b, a, Options::STRICT),
            strict,
            "strict reversed {name}"
        );
    }
}

/// Convenience for a case checked in both directions.
pub fn check_both(name: &str, a: &Value, b: &Value, loose: bool, strict: bool) {
    check(name, a, b, loose, strict, false);
}

// Builders that keep the case files readable.

pub fn n(x: f64) -> Value {
    Value::Num(x)
}

pub fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}

pub fn b(x: bool) -> Value {
    Value::Bool(x)
}

pub fn arr(items: Vec<Value>) -> Value {
    Value::Array(items)
}

pub fn obj(entries: Vec<(&str, Value)>) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

pub fn map(entries: Vec<(Value, Value)>) -> Value {
    Value::Map(entries)
}

pub fn set(members: Vec<Value>) -> Value {
    Value::Set(members)
}

pub fn date(ms: f64) -> Value {
    Value::Date(ms)
}

pub fn regex(source: &str, flags: &str) -> Value {
    Value::Regex {
        source: source.to_string(),
        flags: flags.to_string(),
    }
}

pub fn ta(kind: TaKind, bytes: Vec<u8>) -> Value {
    Value::TypedArray { kind, bytes }
}

pub const NULL: Value = Value::Null;
pub const UNDEF: Value = Value::Undefined;
