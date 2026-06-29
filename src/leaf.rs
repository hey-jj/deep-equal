//! Leaf comparison.
//!
//! Leaves are the non-container values: `undefined`, `null`, booleans, numbers,
//! and strings. The algorithm compares them two ways. Strict mode uses
//! `Object.is` (SameValue). Loose mode uses the ECMAScript Abstract Equality
//! Comparison, the `==` operator, restricted to these leaf kinds.

use crate::value::Value;

/// True when both values are leaves (non-container kinds).
///
/// Mirrors the JavaScript guard `typeof x !== 'object'`. `null` is a special
/// case there (`typeof null === 'object'`), but the algorithm reaches leaf
/// comparison through a separate falsy check, so for our purposes `null` and
/// `undefined` count as leaves.
pub fn is_leaf(v: &Value) -> bool {
    matches!(
        v,
        Value::Undefined | Value::Null | Value::Bool(_) | Value::Num(_) | Value::Str(_)
    )
}

/// True when a value is falsy in JavaScript.
///
/// Falsy values are `undefined`, `null`, `false`, `+0`/`-0`, `NaN`, and the
/// empty string. The algorithm's step 4 fires when either operand is falsy.
pub fn is_falsy(v: &Value) -> bool {
    match v {
        Value::Undefined | Value::Null => true,
        Value::Bool(b) => !b,
        Value::Num(n) => *n == 0.0 || n.is_nan(),
        Value::Str(s) => s.is_empty(),
        _ => false,
    }
}

/// SameValue equality, the semantics of `Object.is`.
///
/// `NaN` equals `NaN`. `+0` and `-0` are distinct. Everything else matches
/// `===`. This is the identity short-circuit and the leaf comparison in strict
/// mode.
pub fn same_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Num(x), Value::Num(y)) => {
            if x.is_nan() && y.is_nan() {
                return true;
            }
            // Distinguish +0 and -0 via the sign bit.
            x.to_bits() == y.to_bits()
        }
        _ => false,
    }
}

/// Strict equality, the semantics of `===`.
///
/// Same as `same_value` except `NaN !== NaN` and `+0 === -0`. Used for the
/// loose-mode identity short-circuit at the top of the algorithm.
pub fn strict_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => x == y,
        _ => same_value(a, b),
    }
}

/// Coerce a value to a number, the ECMAScript `ToNumber` for leaf kinds.
///
/// `undefined` becomes `NaN`. `null` becomes `0`. `false`/`true` become
/// `0`/`1`. A string parses with JavaScript number syntax, yielding `NaN` when
/// it does not parse.
pub fn to_number(v: &Value) -> f64 {
    match v {
        Value::Undefined => f64::NAN,
        Value::Null => 0.0,
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::Num(n) => *n,
        Value::Str(s) => string_to_number(s),
        _ => f64::NAN,
    }
}

/// Parse a string as a JavaScript number.
///
/// Implements the StringNumericLiteral grammar that `Number(string)` uses. An
/// empty or whitespace-only string is `0`. `"Infinity"`, hex/octal/binary
/// literals, and decimal forms parse. Anything else is `NaN`.
pub fn string_to_number(s: &str) -> f64 {
    let t = s.trim_matches(is_js_whitespace);
    if t.is_empty() {
        return 0.0;
    }

    match t {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }

    if let Some(rest) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return parse_radix(rest, 16);
    }
    if let Some(rest) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) {
        return parse_radix(rest, 8);
    }
    if let Some(rest) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        return parse_radix(rest, 2);
    }

    // Rust's f64 parser accepts the same decimal forms as JavaScript here, with
    // the exception of a leading `+`, which JavaScript allows.
    let decimal = t.strip_prefix('+').unwrap_or(t);
    decimal.parse::<f64>().unwrap_or(f64::NAN)
}

/// Parse the digits of a radix literal. Empty or out-of-range yields `NaN`.
fn parse_radix(digits: &str, radix: u32) -> f64 {
    if digits.is_empty() {
        return f64::NAN;
    }
    let mut acc: f64 = 0.0;
    for ch in digits.chars() {
        match ch.to_digit(radix) {
            Some(d) => acc = acc * f64::from(radix) + f64::from(d),
            None => return f64::NAN,
        }
    }
    acc
}

/// JavaScript whitespace and line terminators that `Number()` trims.
fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'
            | '\u{000A}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{000D}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// Loose equality of two leaves, the ECMAScript `==` operator.
///
/// Rules for the modeled kinds:
/// - `null == undefined` is true; either against any other kind is false.
/// - number vs number compares numerically, so `NaN != NaN` and `+0 == -0`.
/// - string vs string compares by characters.
/// - bool vs bool compares directly.
/// - number vs string, bool vs anything, etc. coerce both sides to numbers.
pub fn loose_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        // null and undefined are loosely equal to each other and nothing else.
        (Value::Null | Value::Undefined, Value::Null | Value::Undefined) => true,
        (Value::Null | Value::Undefined, _) | (_, Value::Null | Value::Undefined) => false,

        (Value::Num(x), Value::Num(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,

        // Any mixed pair of leaves coerces to numbers. NaN from a failed
        // coercion makes the comparison false.
        _ => {
            let x = to_number(a);
            let y = to_number(b);
            x == y
        }
    }
}
