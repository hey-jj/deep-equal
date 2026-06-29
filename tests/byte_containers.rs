//! Typed array, ArrayBuffer, and SharedArrayBuffer cases.
//!
//! These containers compare by byte contents. Typed arrays also compare by
//! brand: same bytes under different brands are not equal. ArrayBuffer and
//! SharedArrayBuffer compare byte length then contents.

mod common;

use common::*;
use deep_equal::{TaKind, Value};

#[test]
fn typed_array_same_contents() {
    check_both(
        "two Uint8Arrays with the same contents",
        &ta(TaKind::Uint8, vec![1, 2, 3]),
        &ta(TaKind::Uint8, vec![1, 2, 3]),
        true,
        true,
    );
}

#[test]
fn typed_array_different_contents() {
    check_both(
        "two Uint8Arrays with different contents",
        &ta(TaKind::Uint8, vec![1, 2, 3]),
        &ta(TaKind::Uint8, vec![1, 2, 4]),
        false,
        false,
    );
}

#[test]
fn typed_array_length_diff() {
    check_both(
        "typed arrays of different length",
        &ta(TaKind::Uint8, vec![1, 2, 3]),
        &ta(TaKind::Uint8, vec![1, 2]),
        false,
        false,
    );
}

#[test]
fn typed_array_kind_mismatch() {
    // Same bytes, different brand. The whichTypedArray brand check separates
    // them, so they are not equal.
    check_both(
        "Int8Array and Uint8Array with the same bytes",
        &ta(TaKind::Int8, vec![0; 10]),
        &ta(TaKind::Uint8, vec![0; 10]),
        false,
        false,
    );
}

#[test]
fn array_buffers_equal() {
    check_both(
        "similar ArrayBuffers",
        &Value::ArrayBuffer(vec![0; 8]),
        &Value::ArrayBuffer(vec![0; 8]),
        true,
        true,
    );
}

#[test]
fn array_buffers_different_contents() {
    check_both(
        "different ArrayBuffers",
        &Value::ArrayBuffer(vec![9; 8]),
        &Value::ArrayBuffer(vec![0; 8]),
        false,
        false,
    );
}

#[test]
fn array_buffers_length_diff() {
    // A real byteLength governs, so different lengths are unequal.
    check_both(
        "different-length ArrayBuffers",
        &Value::ArrayBuffer(vec![0; 4]),
        &Value::ArrayBuffer(vec![0; 6]),
        false,
        false,
    );
}

#[test]
fn shared_array_buffers_equal() {
    check_both(
        "similar SharedArrayBuffers",
        &Value::SharedArrayBuffer(vec![0; 8]),
        &Value::SharedArrayBuffer(vec![0; 8]),
        true,
        true,
    );
}

#[test]
fn shared_array_buffers_different_contents() {
    check_both(
        "different SharedArrayBuffers",
        &Value::SharedArrayBuffer(vec![9; 8]),
        &Value::SharedArrayBuffer(vec![0; 8]),
        false,
        false,
    );
}

#[test]
fn shared_array_buffers_length_diff() {
    check_both(
        "different-length SharedArrayBuffers",
        &Value::SharedArrayBuffer(vec![0; 4]),
        &Value::SharedArrayBuffer(vec![0; 6]),
        false,
        false,
    );
}

#[test]
fn array_buffer_vs_shared_are_distinct() {
    // Different brands, so an ArrayBuffer and a SharedArrayBuffer never match.
    check_both(
        "ArrayBuffer vs SharedArrayBuffer",
        &Value::ArrayBuffer(vec![0; 8]),
        &Value::SharedArrayBuffer(vec![0; 8]),
        false,
        false,
    );
}
