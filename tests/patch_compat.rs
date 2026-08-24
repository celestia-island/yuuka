//! Compatibility suite for the state-patch runtime modules.
//!
//! Covers four things beyond the in-module unit tests (which are ported
//! verbatim from `plana` `packages/sync/src/patch.rs` as the compat
//! baseline):
//!
//! 1. the serde wire format of [`yuuka::patch::PatchOp`] — pinned byte for
//!    byte against the historical `plana` `Sync.StatePatch` params shape;
//! 2. the official RFC 7396 appendix-A test cases against
//!    [`yuuka::merge::merge_patch`] (plus the one deliberate deviation
//!    inherited from plana, case A.14);
//! 3. a property-based roundtrip: `apply_all(diff(b, a))` reconstructs `a`
//!    from random trees;
//! 4. proof that the yuuka API covers the entire pub surface of plana's
//!    `sync::patch` module.

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// 1. Golden JSON: the wire format must match plana byte for byte.
// ---------------------------------------------------------------------------

#[test]
fn golden_set_op_serializes_like_plana() {
    let op = yuuka::patch::PatchOp::set("state.agents.hubris", json!({"status":"idle"}));
    // Field order (op, path, value) and the lowercase op tag are locked.
    assert_eq!(
        serde_json::to_string(&op).unwrap(),
        r#"{"op":"set","path":"state.agents.hubris","value":{"status":"idle"}}"#
    );
    assert_eq!(
        serde_json::to_value(&op).unwrap(),
        json!({"op":"set","path":"state.agents.hubris","value":{"status":"idle"}})
    );
}

#[test]
fn golden_replace_op_serializes_like_plana() {
    let op = yuuka::patch::PatchOp::replace("state.work_status", json!({"Running":{"code":1}}));
    assert_eq!(
        serde_json::to_string(&op).unwrap(),
        r#"{"op":"replace","path":"state.work_status","value":{"Running":{"code":1}}}"#
    );
    assert_eq!(
        serde_json::to_value(&op).unwrap(),
        json!({"op":"replace","path":"state.work_status","value":{"Running":{"code":1}}})
    );
}

#[test]
fn golden_del_op_omits_the_value_key() {
    let op = yuuka::patch::PatchOp::del("state.agents.kalos");
    // `value` is skipped entirely — the wire object has exactly two keys.
    assert_eq!(
        serde_json::to_string(&op).unwrap(),
        r#"{"op":"del","path":"state.agents.kalos"}"#
    );
    let as_value = serde_json::to_value(&op).unwrap();
    assert_eq!(as_value.as_object().unwrap().len(), 2);
    assert!(as_value.get("value").is_none());
}

#[test]
fn golden_patch_kind_tags_are_lowercase() {
    use yuuka::patch::PatchKind;
    assert_eq!(serde_json::to_value(PatchKind::Set).unwrap(), json!("set"));
    assert_eq!(
        serde_json::to_value(PatchKind::Replace).unwrap(),
        json!("replace")
    );
    assert_eq!(serde_json::to_value(PatchKind::Del).unwrap(), json!("del"));
}

#[test]
fn patch_ops_deserialize_like_plana() {
    use yuuka::patch::{PatchKind, PatchOp};

    let set: PatchOp = serde_json::from_str(r#"{"op":"set","path":"p","value":42}"#).unwrap();
    assert_eq!(set, PatchOp::set("p", json!(42)));

    let replace: PatchOp =
        serde_json::from_str(r#"{"op":"replace","path":"p","value":{"a":1}}"#).unwrap();
    assert_eq!(replace, PatchOp::replace("p", json!({"a":1})));

    let del: PatchOp = serde_json::from_str(r#"{"op":"del","path":"p"}"#).unwrap();
    assert_eq!(del, PatchOp::del("p"));
    assert_eq!(del.value, None);
    assert_eq!(del.op, PatchKind::Del);

    // An explicit JSON null for `value` also lands as `None` (plana
    // behavior: the skip only affects serialization).
    let del_null: PatchOp =
        serde_json::from_str(r#"{"op":"del","path":"p","value":null}"#).unwrap();
    assert_eq!(del_null, PatchOp::del("p"));
}

#[test]
fn patch_op_serde_roundtrips_through_value() {
    use yuuka::patch::PatchOp;

    for op in [
        PatchOp::set("state.a.b", json!({"x":[1,2],"y":"z"})),
        PatchOp::replace("state.c", json!("done")),
        PatchOp::del("state.d.e"),
    ] {
        let back: PatchOp = serde_json::from_value(serde_json::to_value(&op).unwrap()).unwrap();
        assert_eq!(back, op);
    }
}

// ---------------------------------------------------------------------------
// 2. RFC 7396 appendix A: the official example test cases.
// ---------------------------------------------------------------------------

#[test]
fn merge_patch_passes_rfc7396_appendix_a() {
    // Cases A.1–A.13 of the RFC, verbatim. (A.14 and A.15 are the two
    // deliberate plana deviations — see
    // `merge_patch_appendix_a_deviation_cases_are_pinned`.)
    let cases: &[(Value, Value, Value)] = &[
        (json!({"a":"b"}), json!({"a":"c"}), json!({"a":"c"})),
        (json!({"a":"b"}), json!({"b":"c"}), json!({"a":"b","b":"c"})),
        (json!({"a":"b"}), json!({"a":null}), json!({})),
        (
            json!({"a":"b","b":"c"}),
            json!({"a":null}),
            json!({"b":"c"}),
        ),
        (json!({"a":["b"]}), json!({"a":"c"}), json!({"a":"c"})),
        (json!({"a":"c"}), json!({"a":["b"]}), json!({"a":["b"]})),
        (
            json!({"a":{"b":"c"}}),
            json!({"a":{"b":"d","c":null}}),
            json!({"a":{"b":"d"}}),
        ),
        (json!({"a":[{"b":"c"}]}), json!({"a":[1]}), json!({"a":[1]})),
        (json!(["a", "b"]), json!(["c", "d"]), json!(["c", "d"])),
        (json!({"a":"b"}), json!(["c"]), json!(["c"])),
        (json!({"a":"foo"}), json!(null), json!(null)),
        (json!({"a":"foo"}), json!("bar"), json!("bar")),
        (json!({"e":null}), json!({"a":1}), json!({"e":null,"a":1})),
    ];
    assert_eq!(cases.len(), 13, "appendix-A cases 1-13 must be present");
    for (i, (original, patch, expected)) in cases.iter().enumerate() {
        assert_eq!(
            yuuka::merge::merge_patch(original.clone(), patch.clone()),
            *expected,
            "RFC 7396 appendix A case {} failed",
            i + 1
        );
    }
}

#[test]
fn merge_patch_appendix_a_deviation_cases_are_pinned() {
    // Case A.14: original [1,2], patch {"a":"b","c":null}. Strict RFC 7396
    // would reset the non-object target to {} before merging and produce
    // {"a":"b"}. The plana variant — kept verbatim here — returns the
    // object patch wholesale instead, because `PatchOp::set` must be able
    // to write null-bearing objects over non-object leaves for the
    // diff/apply roundtrip to hold. See
    // `nested_null_members_documented_limitation` for the other side of
    // this trade-off.
    assert_eq!(
        yuuka::merge::merge_patch(json!([1, 2]), json!({"a":"b","c":null})),
        json!({"a":"b","c":null})
    );

    // Case A.15: original {}, patch {"a":{"bb":{"ccc":null}}}. Strict RFC
    // 7396 recurses into newly-added values and strips their null members
    // ({"a":{"bb":{}}}); the plana variant inserts the value of an absent
    // key verbatim, so nulls nested inside newly-added objects survive.
    // This is what lets `diff`+`set` reproduce deeply null-bearing trees.
    assert_eq!(
        yuuka::merge::merge_patch(json!({}), json!({"a":{"bb":{"ccc":null}}})),
        json!({"a":{"bb":{"ccc":null}}})
    );
}

#[test]
fn nested_null_members_documented_limitation() {
    // RFC 7396 cannot distinguish "delete key" from "set key to null": a
    // null member inside a *wholesale-set* object value is eaten by the
    // deep merge when the target at that path is an object (or was just
    // auto-created by the descent). plana semantics, kept as-is — the
    // state-tree domain uses nulls only as tombstones at leaf positions.
    let mut root = json!({"x": {}});
    yuuka::patch::apply(
        &mut root,
        &yuuka::patch::PatchOp::set("x", json!({"a":null,"b":1})),
    );
    assert_eq!(root, json!({"x":{"b":1}})); // "a" eaten by the deep merge

    // Over a non-object leaf the object patch replaces wholesale — nulls
    // survive (the case-A.14 deviation working as intended).
    let mut root = json!({"x": 5});
    yuuka::patch::apply(
        &mut root,
        &yuuka::patch::PatchOp::set("x", json!({"a":null,"b":1})),
    );
    assert_eq!(root, json!({"x":{"a":null,"b":1}}));

    // Nulls as *whole leaf values* roundtrip fine, including inside
    // objects that diff walks key by key.
    for (before, after) in [
        (json!({}), json!({"a":null})),
        (json!({"a":1}), json!({"a":null})),
        (json!({"a":{"x":1}}), json!({"a":null})),
        (json!({"a":null}), json!({"a":{"x":1}})),
        (json!({"k":{"x":1}}), json!({"k":{"x":null,"z":1}})),
    ] {
        let mut rebuilt = before.clone();
        yuuka::patch::apply_all(&mut rebuilt, &yuuka::diff::diff("", &before, &after));
        assert_eq!(rebuilt, after, "before={before}, after={after}");
    }
}

// ---------------------------------------------------------------------------
// 3. Property roundtrip: apply_all(diff(before, after)) == after.
// ---------------------------------------------------------------------------

/// Deterministic xorshift64* PRNG — enough randomness for tree fuzzing
/// without pulling in a dependency.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// Keys never contain dots (state-tree convention), and come from a tiny
/// alphabet so before/after key sets collide and exercise add/change/del.
const KEYS: [&str; 4] = ["a", "b", "c", "d"];

/// Leaves are null-free: a null member inside a wholesale-written object
/// value cannot survive the RFC 7396 deep merge (see
/// `nested_null_members_documented_limitation`), so the roundtrip property
/// holds on the state-tree domain of null-free object members. Nulls as
/// whole leaf values are covered by the hand-picked cases above.
fn leaf(rng: &mut Rng) -> Value {
    match rng.below(8) {
        0 => json!(""),
        1 => json!(true),
        2 => json!(false),
        3 => json!(0),
        4 => json!(7),
        5 => json!(-3),
        6 => json!("text"),
        _ => json!(0.5),
    }
}

fn gen_value(rng: &mut Rng, depth: u32) -> Value {
    if depth == 0 {
        return leaf(rng);
    }
    match rng.below(10) {
        0..=3 => leaf(rng),
        4..=7 => {
            let mut map = serde_json::Map::new();
            for _ in 0..rng.below(4) {
                let key = KEYS[rng.below(KEYS.len() as u64) as usize];
                let value = gen_value(rng, depth - 1);
                map.insert(key.to_string(), value);
            }
            Value::Object(map)
        }
        _ => {
            let items: Vec<Value> = (0..rng.below(3))
                .map(|_| gen_value(rng, depth - 1))
                .collect();
            Value::Array(items)
        }
    }
}

#[test]
fn property_diff_apply_roundtrip() {
    // `seed | 1` keeps xorshift away from its all-zero fixed point.
    for seed in [1u64, 2, 7, 42, 0xDEAD_BEEF, 0x0123_4567_89AB_CDEF] {
        let mut rng = Rng(seed | 1);
        for _ in 0..200 {
            let before = gen_value(&mut rng, 4);
            let after = gen_value(&mut rng, 4);
            let mut rebuilt = before.clone();
            yuuka::patch::apply_all(&mut rebuilt, &yuuka::diff::diff("", &before, &after));
            assert_eq!(
                rebuilt, after,
                "roundtrip failed for seed {seed}: before={before}, after={after}"
            );
        }
    }
}

#[test]
fn diff_apply_roundtrip_on_root_type_swaps() {
    // Hand-picked adversarial pairs around the root and type swaps.
    let cases: &[(Value, Value)] = &[
        (json!(null), json!({})),
        (json!({}), json!(null)),
        (json!(5), json!({"a": {"b": "x"}})),
        (json!({"a": {"b": "x"}}), json!(5)),
        (json!({"a": {"b": 1}}), json!({"a": 1})),
        (json!({"a": 1}), json!({"a": {"b": 1}})),
        (json!([1, [2]]), json!({"a": [1, [2]]})),
        (json!({"a": [1, [2]]}), json!([1, [2]])),
        (json!({"a": "x"}), json!({})),
    ];
    for (before, after) in cases {
        let mut rebuilt = before.clone();
        yuuka::patch::apply_all(&mut rebuilt, &yuuka::diff::diff("", before, after));
        assert_eq!(
            &rebuilt, after,
            "roundtrip failed: before={before}, after={after}"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. API coverage of plana `sync::patch`'s entire pub surface.
// ---------------------------------------------------------------------------
//
// Mapping (plana → yuuka):
//
// | plana `sync::patch`            | yuuka                       |
// |--------------------------------|-----------------------------|
// | `PatchOp` (struct + 3 ctors)   | `yuuka::patch::PatchOp`     |
// | `PatchKind`                    | `yuuka::patch::PatchKind`   |
// | `apply` / `apply_all`          | `yuuka::patch::apply*`      |
// | `merge_patch`                  | `yuuka::merge::merge_patch` |
// | `diff`                         | `yuuka::diff::diff`         |
// | `split_path`                   | `yuuka::path::split`        |

#[test]
fn api_surface_matches_plana_patch_module() {
    use yuuka::patch::{self, PatchKind, PatchOp};

    // Function-pointer coercions pin the exact signatures plana callers
    // rely on.
    let merge_patch: fn(Value, Value) -> Value = yuuka::merge::merge_patch;
    let apply: fn(&mut Value, &PatchOp) = patch::apply;
    let apply_all: fn(&mut Value, &[PatchOp]) = patch::apply_all;
    let diff: fn(&str, &Value, &Value) -> Vec<PatchOp> = yuuka::diff::diff;
    let split: fn(&str) -> Vec<&str> = yuuka::path::split;

    // Constructors build the documented shapes.
    let set_op = PatchOp::set("state.a", json!(1));
    let replace_op = PatchOp::replace("state.b", json!([1]));
    let del_op = PatchOp::del("state.c");
    assert_eq!(set_op.op, PatchKind::Set);
    assert_eq!(replace_op.op, PatchKind::Replace);
    assert_eq!(del_op.op, PatchKind::Del);

    // The struct fields are public.
    let PatchOp { op, path, value } = &set_op;
    assert_eq!(*op, PatchKind::Set);
    assert_eq!(path, "state.a");
    assert_eq!(*value, Some(json!(1)));
    assert_eq!(del_op.value, None);

    // Derives required by plana callers: Debug/Clone/PartialEq on both
    // types (asserted via use), plus Copy/Eq on PatchKind.
    fn assert_derives<T: std::fmt::Debug + Clone + PartialEq>(v: T) -> T {
        v
    }
    let copied_kind = PatchKind::Set;
    let moved_kind = copied_kind;
    assert_eq!(copied_kind, moved_kind); // still usable → PatchKind: Copy.
    let cloned_op = assert_derives(set_op.clone());
    let kind = assert_derives(copied_kind);
    assert!(format!("{cloned_op:?}").contains("PatchOp"));
    assert!(format!("{kind:?}").contains("Set"));

    // The full pipeline runs through the yuuka paths: set writes a,
    // replace writes b, del of the absent c is a no-op.
    let mut root = json!({});
    apply(&mut root, &set_op);
    apply_all(&mut root, &[replace_op, del_op]);
    assert_eq!(root, json!({"state":{"a":1,"b":[1]}}));
    assert!(diff("state", &root, &root).is_empty());
    assert_eq!(split("state.a"), vec!["state", "a"]);
    assert_eq!(merge_patch(json!({}), json!({"x":1})), json!({"x":1}));
}
