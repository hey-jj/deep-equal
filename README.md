# deep-equal

Recursive deep equality for arbitrary JavaScript-shaped values.

This crate answers one question: are two values deeply equal? It reproduces the
recursive equality algorithm behind Node's `assert.deepEqual` and
`assert.deepStrictEqual` as a standalone predicate. It returns a boolean and
never panics on normal input.

## Two modes

- Loose (the default) compares leaf primitives with coercive `==`. So `"3"` and
  `3` are loosely equal, `null` and `undefined` are loosely equal, and `+0` and
  `-0` are loosely equal.
- Strict compares leaf primitives with `Object.is`. So `NaN` equals `NaN`, `+0`
  and `-0` differ, and `"3"` and `3` differ.

## The value model

JavaScript erases type information at runtime. The algorithm branches on the
runtime kind of each value: object, array, `Map`, `Set`, `Date`, `RegExp`,
typed array, `ArrayBuffer`, and `SharedArrayBuffer`. Rust keeps that information
in the type system, so the crate models the value space with the `Value` enum
and branches on it.

```rust
use deep_equal::{deep_equal, Options, Value};

let a = Value::Object(vec![
    ("a".into(), Value::Num(2.0)),
    ("b".into(), Value::Str("4".into())),
]);
let b = Value::Object(vec![
    ("a".into(), Value::Num(2.0)),
    ("b".into(), Value::Num(4.0)),
]);

// Loose: the string "4" coerces to the number 4.
assert!(deep_equal(&a, &b, Options::LOOSE));
// Strict: a string and a number are never equal.
assert!(!deep_equal(&a, &b, Options::STRICT));
```

`deep_equal_loose` and `deep_equal_strict` are shorthands for the two modes.

## What it covers

- Recursive object and array equality, order independent over keys.
- Loose and strict leaf coercion: number, string, boolean, null, undefined.
- `Map` and `Set` equality, order independent, with deep object keys and
  members and loose primitive coercion.
- `Date` timestamps, `RegExp` source and canonical flags.
- Typed array brand and byte equality, `ArrayBuffer` and `SharedArrayBuffer`
  byte equality.
- Signed zero, NaN, and the infinities.

## Installation

```toml
[dependencies]
deep-equal = "0.1"
```

## License

Licensed under the [MIT license](LICENSE).
