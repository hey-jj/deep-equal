//! Value model.
//!
//! JavaScript values are dynamically typed. The equality algorithm branches on
//! the runtime kind of each operand: boxed primitive, arguments object, typed
//! array, `Map`, `Set`, `Date`, `RegExp`, prototype chain, and the loose `==`
//! coercion rules. To reproduce that behavior in Rust we model the value space
//! with an explicit enum. Each variant maps to a JavaScript value kind the
//! algorithm distinguishes.

/// A JavaScript runtime value.
///
/// The variants cover the leaf and container kinds the equality algorithm
/// treats differently. Numbers use `f64` so `NaN`, `+0`, `-0`, and the
/// infinities are all representable and distinguishable where the algorithm
/// cares.
#[derive(Debug, Clone)]
pub enum Value {
    /// JavaScript `undefined`.
    Undefined,
    /// JavaScript `null`.
    Null,
    /// A boolean.
    Bool(bool),
    /// A number. Holds `NaN`, `+0`/`-0`, and `±Infinity`.
    Num(f64),
    /// A string. Compared as an opaque sequence of characters.
    Str(String),
    /// An ordered, indexed array. Indices become string keys `"0"`, `"1"`, etc.
    Array(Vec<Value>),
    /// A plain object as an insertion-ordered list of own enumerable string
    /// keys. Key order does not affect equality.
    Object(Vec<(String, Value)>),
    /// A `Map` with arbitrary value keys. Entry order does not affect equality.
    Map(Vec<(Value, Value)>),
    /// A `Set` of members. Member order does not affect equality.
    Set(Vec<Value>),
    /// A `Date` holding epoch milliseconds. `NaN` represents an Invalid Date.
    Date(f64),
    /// A regular expression. Equality compares `source` and canonical `flags`.
    Regex {
        /// The pattern text.
        source: String,
        /// The flag string in canonical sorted order, for example `gi`.
        flags: String,
    },
    /// A typed array. The `kind` tag is load-bearing: an `Int8Array` and a
    /// `Uint8Array` with identical bytes are not equal because the brand check
    /// separates them.
    TypedArray {
        /// Which typed array brand this is.
        kind: TaKind,
        /// The raw bytes.
        bytes: Vec<u8>,
    },
    /// An `ArrayBuffer`. Equality compares byte length then byte contents.
    ArrayBuffer(Vec<u8>),
    /// A `SharedArrayBuffer`. Equality compares byte length then byte contents.
    SharedArrayBuffer(Vec<u8>),
}

/// The brand of a typed array.
///
/// JavaScript's `whichTypedArray` returns the constructor name. Two typed
/// arrays with different brands are never equal, even with identical bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaKind {
    /// `Int8Array`.
    Int8,
    /// `Uint8Array`.
    Uint8,
    /// `Uint8ClampedArray`.
    Uint8Clamped,
    /// `Int16Array`.
    Int16,
    /// `Uint16Array`.
    Uint16,
    /// `Int32Array`.
    Int32,
    /// `Uint32Array`.
    Uint32,
    /// `Float32Array`.
    Float32,
    /// `Float64Array`.
    Float64,
    /// `BigInt64Array`.
    BigInt64,
    /// `BigUint64Array`.
    BigUint64,
}

/// Comparison options.
///
/// Only the `strict` flag exists. The default is loose comparison, matching the
/// default of `assert.deepEqual`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// When true, compare leaf values with `Object.is` semantics and compare
    /// object prototypes. When false (the default), compare leaves with
    /// coercive `==`.
    pub strict: bool,
}

impl Options {
    /// Loose options. Leaf comparison uses coercive `==`.
    pub const LOOSE: Options = Options { strict: false };

    /// Strict options. Leaf comparison uses `Object.is` and prototypes are
    /// compared.
    pub const STRICT: Options = Options { strict: true };
}
