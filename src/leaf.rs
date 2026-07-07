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

    // JavaScript allows a leading `+` that Rust's parser rejects, so strip it.
    // Rust's parser also accepts `inf` and `infinity` in any case as infinity,
    // but `Number()` reads only the exact "Infinity" handled above, so reject
    // the word forms before parsing the decimal.
    let decimal = t.strip_prefix('+').unwrap_or(t);
    let unsigned = decimal.strip_prefix('-').unwrap_or(decimal);
    if unsigned.eq_ignore_ascii_case("inf") || unsigned.eq_ignore_ascii_case("infinity") {
        return f64::NAN;
    }
    decimal.parse::<f64>().unwrap_or(f64::NAN)
}

/// Parse the digits of a radix literal. Empty or out-of-range yields `NaN`.
fn parse_radix(digits: &str, radix: u32) -> f64 {
    if digits.is_empty() {
        return f64::NAN;
    }
    let bit_width = radix.trailing_zeros();
    let mut bits: Vec<bool> = Vec::new();
    for ch in digits.chars() {
        match ch.to_digit(radix) {
            Some(d) => {
                for shift in (0..bit_width).rev() {
                    let bit = (d & (1 << shift)) != 0;
                    if bit || !bits.is_empty() {
                        bits.push(bit);
                    }
                }
            }
            None => return f64::NAN,
        }
    }
    integer_bits_to_f64(&bits)
}

/// Convert exact positive integer bits to a number with one final rounding.
fn integer_bits_to_f64(bits: &[bool]) -> f64 {
    if bits.is_empty() {
        return 0.0;
    }

    if bits.len() <= f64::MANTISSA_DIGITS as usize {
        return bits
            .iter()
            .fold(0_u64, |acc, bit| (acc << 1) | u64::from(*bit)) as f64;
    }

    let mut exponent = bits.len() as i32 - 1;
    let mut significand = bits[..f64::MANTISSA_DIGITS as usize]
        .iter()
        .fold(0_u64, |acc, bit| (acc << 1) | u64::from(*bit));
    let guard = bits[f64::MANTISSA_DIGITS as usize];
    let sticky = bits[f64::MANTISSA_DIGITS as usize + 1..]
        .iter()
        .any(|bit| *bit);
    if guard && (sticky || significand % 2 == 1) {
        significand += 1;
        if significand == (1_u64 << f64::MANTISSA_DIGITS) {
            significand >>= 1;
            exponent += 1;
        }
    }

    if exponent > 1023 {
        return f64::INFINITY;
    }

    let biased = (exponent + 1023) as u64;
    let fraction = significand - (1_u64 << (f64::MANTISSA_DIGITS - 1));
    f64::from_bits((biased << 52) | fraction)
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
