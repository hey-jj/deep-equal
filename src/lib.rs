//! Recursive deep equality for arbitrary JavaScript-shaped values.
//!
//! This crate answers one question: are two values deeply equal? It reproduces
//! the recursive equality algorithm behind Node's `assert.deepEqual` and
//! `assert.deepStrictEqual` as a standalone predicate that returns a boolean and
//! never panics on normal input.
//!
//! Two comparison modes exist.
//!
//! - Loose (the default) compares leaf primitives with coercive `==`. So
//!   `"3"` and `3` are loosely equal, `null` and `undefined` are loosely equal,
//!   and `+0` and `-0` are loosely equal.
//! - Strict compares leaf primitives with `Object.is`. So `NaN` equals `NaN`,
//!   `+0` and `-0` differ, and `"3"` and `3` differ.
//!
//! JavaScript erases type information at runtime, so the algorithm spends most
//! of its effort recovering it: typed array brands, `Map`/`Set` structure,
//! `Date` timestamps, and `RegExp` source and flags. Rust keeps that
//! information in the type system, so we model the value space with the
//! [`Value`] enum and branch on it.
//!
//! # Example
//!
//! ```
//! use deep_equal::{deep_equal, Options, Value};
//!
//! let a = Value::Object(vec![
//!     ("a".into(), Value::Num(2.0)),
//!     ("b".into(), Value::Str("4".into())),
//! ]);
//! let b = Value::Object(vec![
//!     ("a".into(), Value::Num(2.0)),
//!     ("b".into(), Value::Num(4.0)),
//! ]);
//!
//! // Loose: the string "4" coerces to the number 4.
//! assert!(deep_equal(&a, &b, Options::LOOSE));
//! // Strict: a string and a number are never equal.
//! assert!(!deep_equal(&a, &b, Options::STRICT));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod leaf;
mod value;

pub use value::{Options, TypedArrayKind, Value};

use std::borrow::Cow;

/// Compare two values for deep equality.
///
/// Returns `true` when `a` and `b` are deeply equivalent under the given
/// options. The comparison is symmetric: `deep_equal(a, b, o)` equals
/// `deep_equal(b, a, o)`.
///
/// A [`Value`] owns its children, so it is always a finite tree. Recursion over
/// it terminates without a cycle guard.
pub fn deep_equal(a: &Value, b: &Value, opts: Options) -> bool {
    internal_deep_equal(a, b, opts)
}

/// Compare two values with loose (coercive) leaf equality.
///
/// Shorthand for [`deep_equal`] with [`Options::LOOSE`].
pub fn deep_equal_loose(a: &Value, b: &Value) -> bool {
    deep_equal(a, b, Options::LOOSE)
}

/// Compare two values with strict (`Object.is`) leaf equality.
///
/// Shorthand for [`deep_equal`] with [`Options::STRICT`].
pub fn deep_equal_strict(a: &Value, b: &Value) -> bool {
    deep_equal(a, b, Options::STRICT)
}

/// Identity short-circuit comparison for the chosen mode.
///
/// Strict uses SameValue, so `NaN` short-circuits as equal. Loose uses `===`,
/// so `NaN` does not. This is the gate at the top of the recursion.
fn identity_eq(a: &Value, b: &Value, opts: Options) -> bool {
    if opts.strict {
        leaf::same_value(a, b)
    } else {
        leaf::strict_eq(a, b)
    }
}

/// Leaf comparison for the chosen mode.
///
/// Strict uses SameValue. Loose uses coercive `==`.
fn leaf_eq(a: &Value, b: &Value, opts: Options) -> bool {
    if opts.strict {
        leaf::same_value(a, b)
    } else {
        leaf::loose_eq(a, b)
    }
}

/// The top-level recursive entry.
fn internal_deep_equal(a: &Value, b: &Value, opts: Options) -> bool {
    // Identity short-circuit.
    if identity_eq(a, b, opts) {
        return true;
    }

    // The boxed-primitive brand check is vacuous in this model: a boxed
    // primitive has no representation distinct from its primitive, so there is
    // no brand to differ on. The check exists in JavaScript to make
    // `Object(3)` and `3` unequal, which this model does not express.

    // Leaf comparison: either operand falsy, or neither is an object.
    if leaf::is_falsy(a) || leaf::is_falsy(b) || (leaf::is_leaf(a) && leaf::is_leaf(b)) {
        return leaf_eq(a, b, opts);
    }

    obj_equiv(a, b, opts)
}

/// Structural comparison of two non-leaf values, matching `objEquiv`.
///
/// The first failing check returns false. Each gate mirrors a brand or shape
/// distinction the JavaScript algorithm enforces in the same order.
fn obj_equiv(a: &Value, b: &Value, opts: Options) -> bool {
    // Brand string (Object.prototype.toString). Differing brands are unequal.
    if brand(a) != brand(b) {
        return false;
    }

    // Array class must match.
    if is_array(a) != is_array(b) {
        return false;
    }

    // RegExp: source and canonical flags must match.
    if let (
        Value::Regex {
            source: sa,
            flags: fa,
        },
        Value::Regex {
            source: sb,
            flags: fb,
        },
    ) = (a, b)
    {
        return sa == sb && canonical_flags(fa) == canonical_flags(fb);
    }

    // Date: timestamps must match. NaN timestamps never match, so two Invalid
    // Dates are unequal, matching `getTime` returning NaN.
    if let (Value::Date(ta), Value::Date(tb)) = (a, b) {
        return ta == tb;
    }

    // Typed arrays: brand then length then strict element compare. Early
    // return, no key loop.
    if let (
        Value::TypedArray {
            kind: ka,
            bytes: ba,
        },
        Value::TypedArray {
            kind: kb,
            bytes: bb,
        },
    ) = (a, b)
    {
        return ka == kb && ba == bb;
    }

    // ArrayBuffer: byte length then byte contents. Early return.
    if let (Value::ArrayBuffer(ba), Value::ArrayBuffer(bb)) = (a, b) {
        return ba == bb;
    }

    // SharedArrayBuffer: byte length then byte contents. Early return.
    if let (Value::SharedArrayBuffer(ba), Value::SharedArrayBuffer(bb)) = (a, b) {
        return ba == bb;
    }

    // Own enumerable string keys: same count, same keys, equal values per key.
    // Arrays and plain objects flow through here. Their keys are the index
    // strings or property names.
    //
    // Each side is a sorted list of (key, value). The two lists walk in
    // lockstep, so each key pairs with its value directly and no per-key lookup
    // is needed.
    let ka = keys(a);
    let kb = keys(b);
    if ka.len() != kb.len() {
        return false;
    }
    for ((key_a, va), (key_b, vb)) in ka.iter().zip(kb.iter()) {
        if key_a != key_b {
            return false;
        }
        if !internal_deep_equal(va, vb, opts) {
            return false;
        }
    }

    // Collections get structural treatment after the key loop.
    match (a, b) {
        (Value::Set(_), Value::Set(_)) => set_equiv(a, b, opts),
        (Value::Map(_), Value::Map(_)) => map_equiv(a, b, opts),
        _ => true,
    }
}

/// The brand string an object reports through `Object.prototype.toString`.
///
/// Distinct brands make two values unequal at the gate. A `Set` and a `Map`
/// differ here. A `Date` and a `RegExp` differ here.
fn brand(v: &Value) -> &'static str {
    match v {
        Value::Undefined => "[object Undefined]",
        Value::Null => "[object Null]",
        Value::Bool(_) => "[object Boolean]",
        Value::Num(_) => "[object Number]",
        Value::Str(_) => "[object String]",
        Value::Array(_) => "[object Array]",
        Value::Object(_) => "[object Object]",
        Value::Map(_) => "[object Map]",
        Value::Set(_) => "[object Set]",
        Value::Date(_) => "[object Date]",
        Value::Regex { .. } => "[object RegExp]",
        // Typed arrays each report their own brand string.
        Value::TypedArray { kind, .. } => match kind {
            TypedArrayKind::Int8 => "[object Int8Array]",
            TypedArrayKind::Uint8 => "[object Uint8Array]",
            TypedArrayKind::Uint8Clamped => "[object Uint8ClampedArray]",
            TypedArrayKind::Int16 => "[object Int16Array]",
            TypedArrayKind::Uint16 => "[object Uint16Array]",
            TypedArrayKind::Int32 => "[object Int32Array]",
            TypedArrayKind::Uint32 => "[object Uint32Array]",
            TypedArrayKind::Float32 => "[object Float32Array]",
            TypedArrayKind::Float64 => "[object Float64Array]",
            TypedArrayKind::BigInt64 => "[object BigInt64Array]",
            TypedArrayKind::BigUint64 => "[object BigUint64Array]",
        },
        Value::ArrayBuffer(_) => "[object ArrayBuffer]",
        Value::SharedArrayBuffer(_) => "[object SharedArrayBuffer]",
    }
}

/// True when the value is an array, matching `Array.isArray`.
fn is_array(v: &Value) -> bool {
    matches!(v, Value::Array(_))
}

/// The own enumerable string keys of an object or array, sorted by key with
/// each key paired to its value.
///
/// Plain objects report their property names. Arrays report their index
/// strings, `"0"`, `"1"`, and so on. Other kinds have no enumerable string
/// keys here, so they compare structurally through their dedicated paths.
///
/// Object keys borrow from the entry. Array index strings are owned because
/// they are built on the fly. Duplicate object keys collapse to the last value,
/// matching how JavaScript builds an object from a property list. Objects are
/// sorted so two objects with the same keys in different orders compare equal in
/// a single lockstep walk.
fn keys(v: &Value) -> Vec<(Cow<'_, str>, &Value)> {
    match v {
        Value::Object(entries) => {
            // Sort by key first. A stable sort keeps insertion order within a
            // run of equal keys, so the final entry in each run is the last
            // write. Then dedup adjacent keys keeping that last entry.
            let mut pairs: Vec<(Cow<'_, str>, &Value)> = entries
                .iter()
                .map(|(k, val)| (Cow::Borrowed(k.as_str()), val))
                .collect();
            pairs.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
            let mut out: Vec<(Cow<'_, str>, &Value)> = Vec::with_capacity(pairs.len());
            for (key, val) in pairs {
                match out.last_mut() {
                    Some(last) if last.0 == key => last.1 = val,
                    _ => out.push((key, val)),
                }
            }
            out
        }
        Value::Array(items) => items
            .iter()
            .enumerate()
            .map(|(i, val)| (Cow::Owned(i.to_string()), val))
            .collect(),
        _ => Vec::new(),
    }
}

/// Normalize a regex flag string to canonical sorted order.
///
/// `regexp.prototype.flags` returns flags in a fixed order. Comparing
/// normalized forms makes `gi` and `ig` equal.
fn canonical_flags(flags: &str) -> String {
    let mut chars: Vec<char> = flags.chars().collect();
    chars.sort_unstable();
    chars.into_iter().collect()
}

/// Set comparison.
///
/// Sets compare by membership, order independent. Primitive members use direct
/// containment with loose coercion in loose mode. Object members need deep
/// matching, always done loosely. The algorithm drops the strict flag when it
/// matches object members across sets, so this path mirrors that.
fn set_equiv(a: &Value, b: &Value, opts: Options) -> bool {
    let (raw_a, raw_b) = match (a, b) {
        (Value::Set(ea), Value::Set(eb)) => (ea, eb),
        _ => return false,
    };
    // A JavaScript Set holds distinct members. Construction collapses repeated
    // primitives by SameValueZero. Object members stay distinct because a Set
    // keys them by reference, which this model does not express, so each is
    // kept. Compare the deduplicated sizes, not the raw vector lengths.
    let dedup_a = dedup_set(raw_a);
    let dedup_b = dedup_set(raw_b);
    let ea = dedup_a.as_slice();
    let eb = dedup_b.as_slice();
    if ea.len() != eb.len() {
        return false;
    }

    // Leftover holds a-members still needing a deep match against b. We track
    // them by index into `ea`. A used index is removed so it is not matched
    // twice.
    let mut leftover: Vec<usize> = Vec::new();

    for (i, val) in ea.iter().enumerate() {
        if !leaf::is_falsy(val) && !leaf::is_leaf(val) {
            // Object member: always needs deep matching.
            leftover.push(i);
        } else if !set_has(eb, val) {
            // Primitive not directly present in b.
            if opts.strict {
                return false;
            }
            if !set_might_have_loose_prim(ea, eb, val) {
                return false;
            }
            leftover.push(i);
        }
    }

    if leftover.is_empty() {
        return true;
    }

    for val in eb.iter() {
        if !leaf::is_falsy(val) && !leaf::is_leaf(val) {
            // Object member from b: find a deep match in leftover. The match is
            // forced loose, matching the strict-flag drop noted above.
            if !set_take_equal(ea, &mut leftover, val) {
                return false;
            }
        } else if !opts.strict && !set_has(ea, val) && !set_take_equal(ea, &mut leftover, val) {
            return false;
        }
    }

    leftover.is_empty()
}

/// Whether a set contains a primitive member, using SameValueZero containment.
/// JavaScript `Set.prototype.has` treats `NaN` as present and `+0`/`-0` as the
/// same. Strictness does not change containment here.
fn set_has(members: &[Value], val: &Value) -> bool {
    members.iter().any(|m| set_same(m, val))
}

/// Membership equality for `Set.prototype.has`: SameValueZero. `NaN` matches
/// `NaN`, `+0` matches `-0`, otherwise `===`.
fn set_same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => (x.is_nan() && y.is_nan()) || x == y,
        _ => leaf::same_value(a, b),
    }
}

/// Collapse repeated primitive members of a set, matching how a JavaScript Set
/// drops duplicate primitives on construction.
///
/// Primitives compare by SameValueZero, so `1` appears once and `NaN` appears
/// once. Object members are kept as is. A real Set keys them by reference, which
/// this model has no way to express, so two structurally equal objects stay as
/// two members.
fn dedup_set(members: &[Value]) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::with_capacity(members.len());
    for m in members {
        let already_kept = leaf::is_leaf(m)
            && out
                .iter()
                .any(|kept| leaf::is_leaf(kept) && set_same(kept, m));
        if already_kept {
            continue;
        }
        out.push(m.clone());
    }
    out
}

/// Find a deep-equal match for `val` among the leftover a-members and consume
/// it. Matching is loose because the strict flag is dropped on this path.
fn set_take_equal(ea: &[Value], leftover: &mut Vec<usize>, val: &Value) -> bool {
    if let Some(pos) = leftover
        .iter()
        .position(|&i| internal_deep_equal(&ea[i], val, Options::LOOSE))
    {
        leftover.remove(pos);
        true
    } else {
        false
    }
}

/// The loose-primitive table, matching `findLooseMatchingPrimitives`.
///
/// A primitive either yields a direct yes/no for whether a loose match might
/// exist, or it signals to retry with a nullish alternate key. `null` and
/// `undefined` are loosely equal, so each is retried as the other.
enum LoosePrim {
    /// A direct answer: a loose match cannot exist beyond what is already
    /// known. Carries the result the caller returns.
    Direct(bool),
    /// Retry with this nullish alternate key. `undefined` retries as `null`,
    /// `null` retries as `undefined`.
    Alt(Value),
}

/// Classify a primitive for loose matching.
fn find_loose_matching_primitives(prim: &Value) -> LoosePrim {
    match prim {
        // undefined retries as null.
        Value::Undefined => LoosePrim::Alt(Value::Null),
        // null retries as undefined.
        Value::Null => LoosePrim::Alt(Value::Undefined),
        // Strings and numbers match loosely only when they coerce to a non-NaN
        // number. The test is `+prim === +prim`.
        Value::Str(_) | Value::Num(_) => LoosePrim::Direct(!leaf::to_number(prim).is_nan()),
        // Booleans coerce cleanly.
        Value::Bool(_) => LoosePrim::Direct(true),
        // Anything else has no loose match.
        _ => LoosePrim::Direct(false),
    }
}

/// Whether a set might hold a loose match for a primitive in b but not a,
/// matching `setMightHaveLoosePrim`.
fn set_might_have_loose_prim(a: &[Value], b: &[Value], prim: &Value) -> bool {
    match find_loose_matching_primitives(prim) {
        LoosePrim::Direct(v) => v,
        LoosePrim::Alt(alt) => set_has(b, &alt) && !set_has(a, &alt),
    }
}

/// Map comparison, matching `mapEquiv`.
///
/// Maps compare by entries, order independent. Primitive keys look up directly
/// with loose coercion in loose mode. Object keys need deep matching, which
/// honors the strict flag.
fn map_equiv(a: &Value, b: &Value, opts: Options) -> bool {
    let (raw_a, raw_b) = match (a, b) {
        (Value::Map(ea), Value::Map(eb)) => (ea, eb),
        _ => return false,
    };
    // A JavaScript Map holds one entry per key. Setting a key twice keeps the
    // last value. Construction collapses repeated primitive keys by
    // SameValueZero. Object keys stay distinct because a Map keys them by
    // reference, which this model does not express. Compare deduplicated sizes,
    // not raw vector lengths.
    let dedup_a = dedup_map(raw_a);
    let dedup_b = dedup_map(raw_b);
    let ea = dedup_a.as_slice();
    let eb = dedup_b.as_slice();
    if ea.len() != eb.len() {
        return false;
    }

    // Leftover holds a-entries with object keys, tracked by index.
    let mut leftover: Vec<usize> = Vec::new();

    for (i, (key, item1)) in ea.iter().enumerate() {
        if !leaf::is_falsy(key) && !leaf::is_leaf(key) {
            leftover.push(i);
        } else {
            let item2 = map_get(eb, key);
            let matched = match item2 {
                Some(v) => internal_deep_equal(item1, v, opts),
                None => false,
            };
            if !matched {
                if opts.strict {
                    return false;
                }
                if !map_might_have_loose_prim(ea, eb, key, item1) {
                    return false;
                }
                leftover.push(i);
            }
        }
    }

    if leftover.is_empty() {
        return true;
    }

    for (key, item2) in eb.iter() {
        if !leaf::is_falsy(key) && !leaf::is_leaf(key) {
            // Object key from b: honor the strict flag.
            if !map_take_equal_entry(ea, &mut leftover, key, item2, opts) {
                return false;
            }
        } else if !opts.strict {
            let direct = match map_get(ea, key) {
                Some(v) => internal_deep_equal(v, item2, opts),
                None => false,
            };
            if !direct && !map_take_equal_entry(ea, &mut leftover, key, item2, Options::LOOSE) {
                return false;
            }
        }
    }

    leftover.is_empty()
}

/// Collapse repeated primitive keys of a map, keeping the last value, matching
/// how a JavaScript Map builds from an entry list.
///
/// Primitive keys compare by SameValueZero. A later assignment to the same key
/// overwrites the value. Object keys are kept as is because a Map keys them by
/// reference, which this model has no way to express, so two structurally equal
/// objects stay as two entries.
fn dedup_map(entries: &[(Value, Value)]) -> Vec<(Value, Value)> {
    let mut out: Vec<(Value, Value)> = Vec::with_capacity(entries.len());
    for (key, val) in entries {
        if leaf::is_leaf(key) {
            if let Some(slot) = out
                .iter_mut()
                .find(|(k, _)| leaf::is_leaf(k) && set_same(k, key))
            {
                slot.1 = val.clone();
                continue;
            }
        }
        out.push((key.clone(), val.clone()));
    }
    out
}

/// Look up a map value by key using SameValueZero key matching, matching
/// `Map.prototype.get`.
fn map_get<'a>(entries: &'a [(Value, Value)], key: &Value) -> Option<&'a Value> {
    entries
        .iter()
        .find(|(k, _)| set_same(k, key))
        .map(|(_, v)| v)
}

/// Whether a map has a key using SameValueZero matching.
fn map_has(entries: &[(Value, Value)], key: &Value) -> bool {
    entries.iter().any(|(k, _)| set_same(k, key))
}

/// Whether a map might hold a loose match for a primitive key, matching
/// `mapMightHaveLoosePrim`.
fn map_might_have_loose_prim(
    a: &[(Value, Value)],
    b: &[(Value, Value)],
    prim: &Value,
    item: &Value,
) -> bool {
    match find_loose_matching_primitives(prim) {
        LoosePrim::Direct(v) => v,
        LoosePrim::Alt(alt) => {
            // Look up the entry under the nullish alternate key.
            let cur_b = map_get(b, &alt);
            let ok_b = match cur_b {
                Some(v) => internal_deep_equal(item, v, Options::LOOSE),
                None => false,
            };
            if !ok_b {
                return false;
            }
            let cur = cur_b.expect("cur_b is Some after ok_b");
            !map_has(a, &alt) && internal_deep_equal(item, cur, Options::LOOSE)
        }
    }
}

/// Find and consume a leftover object-keyed entry whose key and value both
/// deep-match, matching `mapHasEqualEntry`.
fn map_take_equal_entry(
    ea: &[(Value, Value)],
    leftover: &mut Vec<usize>,
    key: &Value,
    item2: &Value,
    opts: Options,
) -> bool {
    if let Some(pos) = leftover.iter().position(|&i| {
        let (k2, v2) = &ea[i];
        internal_deep_equal(key, k2, opts) && internal_deep_equal(item2, v2, opts)
    }) {
        leftover.remove(pos);
        true
    } else {
        false
    }
}
