//! Map and Set cases.
//!
//! Maps and Sets compare order independent. Primitive keys and members coerce
//! in loose mode. Object keys and members need deep matching. These cover the
//! Map, Set, and Set-vs-Map paths including the second-iteration matching.

mod common;

use common::*;

#[test]
fn equal_maps() {
    check_both(
        "two equal Maps",
        &map(vec![(s("a"), n(1.0)), (s("b"), n(2.0))]),
        &map(vec![(s("b"), n(2.0)), (s("a"), n(1.0))]),
        true,
        true,
    );
}

#[test]
fn maps_with_inequal_values() {
    check_both(
        "two Maps with inequal values on the same key",
        &map(vec![(s("a"), arr(vec![n(1.0), n(2.0)]))]),
        &map(vec![(s("a"), arr(vec![n(2.0), n(1.0)]))]),
        false,
        false,
    );
}

#[test]
fn inequal_maps() {
    check_both(
        "two inequal Maps",
        &map(vec![(s("a"), n(1.0))]),
        &map(vec![(s("b"), n(1.0))]),
        false,
        false,
    );
}

#[test]
fn maps_with_object_keys() {
    check_both(
        "two equal Maps in different orders with object keys",
        &map(vec![
            (obj(vec![]), n(3.0)),
            (obj(vec![]), n(2.0)),
            (obj(vec![]), n(1.0)),
        ]),
        &map(vec![
            (obj(vec![]), n(1.0)),
            (obj(vec![]), n(2.0)),
            (obj(vec![]), n(3.0)),
        ]),
        true,
        true,
    );
}

#[test]
fn maps_nullish_keys_and_values() {
    check_both(
        "undefined keys, nullish values",
        &map(vec![(UNDEF, UNDEF)]),
        &map(vec![(UNDEF, NULL)]),
        true,
        false,
    );
    check_both(
        "null keys, nullish values",
        &map(vec![(NULL, UNDEF)]),
        &map(vec![(NULL, NULL)]),
        true,
        false,
    );
    check_both(
        "nullish keys",
        &map(vec![(UNDEF, n(3.0))]),
        &map(vec![(NULL, n(3.0))]),
        true,
        false,
    );
}

#[test]
fn maps_mixed_keys() {
    check_both(
        "equal Maps, different order, primitive keys",
        &map(vec![
            (obj(vec![]), NULL),
            (b(true), n(2.0)),
            (obj(vec![]), n(1.0)),
            (UNDEF, obj(vec![])),
        ]),
        &map(vec![
            (obj(vec![]), n(1.0)),
            (b(true), n(2.0)),
            (obj(vec![]), NULL),
            (UNDEF, obj(vec![])),
        ]),
        true,
        true,
    );
    check_both(
        "equal Maps, different order, mix of keys",
        &map(vec![
            (b(false), n(3.0)),
            (obj(vec![]), n(2.0)),
            (obj(vec![]), n(1.0)),
        ]),
        &map(vec![
            (obj(vec![]), n(1.0)),
            (obj(vec![]), n(2.0)),
            (b(false), n(3.0)),
        ]),
        true,
        true,
    );
}

#[test]
fn map_size_diff() {
    check_both(
        "empty Map vs one-entry Map",
        &map(vec![]),
        &map(vec![(obj(vec![]), n(1.0))]),
        false,
        false,
    );
}

#[test]
fn maps_same_size_object_then_primitive() {
    check_both(
        "inequal maps, primitive key, start with object key",
        &map(vec![(obj(vec![]), NULL), (b(false), n(3.0))]),
        &map(vec![(obj(vec![]), NULL), (b(true), n(2.0))]),
        false,
        false,
    );
    check_both(
        "inequal maps, primitive key, start with primitive key",
        &map(vec![(b(false), n(3.0)), (obj(vec![]), NULL)]),
        &map(vec![(b(true), n(2.0)), (obj(vec![]), NULL)]),
        false,
        false,
    );
}

#[test]
fn map_primitive_comparisons() {
    check_both(
        "primitive comparisons",
        &map(vec![(UNDEF, NULL), (s("+000"), n(2.0))]),
        &map(vec![(NULL, UNDEF), (b(false), s("2"))]),
        true,
        false,
    );
}

#[test]
fn map_null_key_vs_string_key() {
    check_both(
        "Map with null key vs Map with string key",
        &map(vec![(NULL, n(1.0))]),
        &map(vec![(s("x"), n(1.0))]),
        false,
        false,
    );
}

#[test]
fn maps_non_matching_object_keys_in_b() {
    check_both(
        "Maps with different object keys in b",
        &map(vec![
            (obj(vec![("a", n(1.0))]), s("x")),
            (obj(vec![("b", n(2.0))]), s("y")),
        ]),
        &map(vec![
            (obj(vec![("a", n(1.0))]), s("x")),
            (obj(vec![("c", n(3.0))]), s("y")),
        ]),
        false,
        false,
    );
}

#[test]
fn equal_sets() {
    check_both(
        "two equal Sets",
        &set(vec![s("a"), n(1.0), s("b"), n(2.0)]),
        &set(vec![s("b"), n(2.0), s("a"), n(1.0)]),
        true,
        true,
    );
}

#[test]
fn inequal_sets() {
    check_both(
        "two inequal Sets",
        &set(vec![s("a"), n(1.0)]),
        &set(vec![s("b"), n(1.0)]),
        false,
        false,
    );
}

#[test]
fn sets_with_object_members() {
    check_both(
        "two equal Sets in different orders",
        &set(vec![obj(vec![]), n(1.0), obj(vec![]), obj(vec![]), n(2.0)]),
        &set(vec![obj(vec![]), n(1.0), obj(vec![]), n(2.0), obj(vec![])]),
        true,
        true,
    );
}

#[test]
fn set_size_diff() {
    check_both(
        "two inequally sized Sets",
        &set(vec![]),
        &set(vec![n(1.0)]),
        false,
        false,
    );
}

#[test]
fn sets_loose_object_members() {
    check_both(
        "two loosely equal, strictly inequal Sets",
        &set(vec![obj(vec![("a", n(1.0))]), n(2.0)]),
        &set(vec![s("2"), obj(vec![("a", s("1"))])]),
        true,
        false,
    );
    check_both(
        "two inequal Sets with object members",
        &set(vec![obj(vec![("a", n(1.0))]), n(2.0)]),
        &set(vec![s("2"), obj(vec![("a", n(2.0))])]),
        false,
        false,
    );
}

#[test]
fn sets_strict_object_members_match_strictly() {
    // The only members are objects that are loosely but not strictly equal, so
    // no primitive mismatch short-circuits strict mode. Strict must compare the
    // object members strictly and report the Sets unequal.
    check_both(
        "Sets whose object members differ loosely-but-not-strictly",
        &set(vec![obj(vec![("a", n(1.0))])]),
        &set(vec![obj(vec![("a", s("1"))])]),
        true,
        false,
    );
}

#[test]
fn sets_primitive_coercion() {
    check_both(
        "more primitive comparisons",
        &set(vec![NULL, s(""), n(1.0), n(5.0), n(2.0), b(false)]),
        &set(vec![UNDEF, n(0.0), s("5"), b(true), s("2"), s("-000")]),
        true,
        false,
    );
}

#[test]
fn sets_loose_primitive_fails_in_b() {
    check_both(
        "Sets where loose primitive in b matches none in a",
        &set(vec![obj(vec![]), n(1.0)]),
        &set(vec![obj(vec![]), s("2")]),
        false,
        false,
    );
}

#[test]
fn set_vs_map() {
    check_both("Set and Map", &set(vec![]), &map(vec![]), false, false);
}

#[test]
fn set_duplicate_members_collapse() {
    // A Set drops repeated primitives, so the size is the distinct count.
    check_both(
        "Set with repeated member vs single member",
        &set(vec![n(1.0), n(1.0), n(1.0)]),
        &set(vec![n(1.0)]),
        true,
        true,
    );
    // Distinct counts after collapse stay unequal.
    check_both(
        "Set {1,2} vs Set {1,1}",
        &set(vec![n(1.0), n(2.0)]),
        &set(vec![n(1.0), n(1.0)]),
        false,
        false,
    );
}

#[test]
fn map_duplicate_keys_last_wins() {
    // A Map keeps one entry per key with the last assigned value.
    check_both(
        "Map with repeated key vs single entry",
        &map(vec![(s("a"), n(1.0)), (s("a"), n(2.0))]),
        &map(vec![(s("a"), n(2.0))]),
        true,
        true,
    );
    // The first write is shadowed, so matching the first value fails.
    check_both(
        "Map repeated key, last value differs",
        &map(vec![(s("a"), n(1.0)), (s("a"), n(2.0))]),
        &map(vec![(s("a"), n(1.0))]),
        false,
        false,
    );
}

#[test]
fn set_object_member_vs_loose_primitive_in_b() {
    // b holds an object plus a string primitive. The object splits off into
    // leftover, then the b primitive "5" probes leftover after the object
    // match. Loose coercion makes 5 and "5" equal, strict does not.
    check_both(
        "set object member vs loose primitive in b pass",
        &set(vec![obj(vec![]), n(5.0)]),
        &set(vec![obj(vec![]), s("5")]),
        true,
        false,
    );
}
