//! Before/after diffing into [`PatchOp`] lists.
//!
//! [`diff`] walks two versions of a state tree and emits the op set that
//! reconstructs `after` from `before`:
//!
//! - both sides objects → recurse per key over the union; added/changed
//!   keys yield `set` ops, removed keys yield `del` ops;
//! - values differing (at least one side not an object) → a single `set` of
//!   the new value at the current path;
//! - equal values → no op.
//!
//! All generated paths are prefixed with `prefix` (usually `state`), so the
//! same trees can be diffed at any sub-root. Roundtrip guarantee:
//! `apply_all(before.clone(), diff(prefix, &before, &after)) == after` —
//! pinned by the property-based test in `tests/patch_compat.rs`, for the
//! state-tree domain where object members carry no explicit `null`s (the
//! one RFC 7396 caveat: a `null` member inside a wholesale-written object
//! value cannot survive the deep merge; nulls as whole leaf values
//! roundtrip fine).

use serde_json::Value;

use crate::patch::PatchOp;
use crate::path::join;

/// Diffs two versions of a state tree into the op list that transforms
/// `before` into `after`. See the module docs for the strategy.
pub fn diff(prefix: &str, before: &Value, after: &Value) -> Vec<PatchOp> {
    let mut ops = Vec::new();
    diff_into(prefix, before, after, &mut ops);
    ops
}

fn diff_into(prefix: &str, before: &Value, after: &Value, ops: &mut Vec<PatchOp>) {
    match (before, after) {
        (Value::Object(b), Value::Object(a)) => {
            // Keys that vanished → del.
            for k in b.keys() {
                if !a.contains_key(k) {
                    ops.push(PatchOp::del(join(prefix, k)));
                }
            }
            // Keys that were added or changed → set / recurse.
            for (k, av) in a {
                let path = join(prefix, k);
                match b.get(k) {
                    None => ops.push(PatchOp::set(path, av.clone())),
                    Some(bv) => diff_into(&path, bv, av, ops),
                }
            }
        }
        (b, a) if b == a => {
            // Identical values → no op.
        }
        (_, a) => {
            // Different values (at least one non-object) → set the new one.
            ops.push(PatchOp::set(prefix.to_string(), a.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    use crate::patch::{apply_all, PatchKind};

    // Ported from plana `packages/sync/src/patch.rs` (compat baseline).

    #[test]
    fn diff_detects_add_change_remove() {
        let before = json!({"hubris":{"status":"idle"},"kalos":{"status":"busy"}});
        let after = json!({"hubris":{"status":"busy","model":"glm"},"seia":{"status":"idle"}});
        let ops = diff("state.agents", &before, &after);
        // kalos deleted, hubris.status changed + hubris.model added, seia added.
        let has_del_kalos = ops
            .iter()
            .any(|o| o.op == PatchKind::Del && o.path == "state.agents.kalos");
        let has_set_hubris_status = ops.iter().any(|o| {
            o.op == PatchKind::Set
                && o.path == "state.agents.hubris.status"
                && o.value == Some(json!("busy"))
        });
        let has_set_hubris_model = ops.iter().any(|o| {
            o.op == PatchKind::Set
                && o.path == "state.agents.hubris.model"
                && o.value == Some(json!("glm"))
        });
        let has_set_seia = ops.iter().any(|o| {
            o.op == PatchKind::Set
                && o.path == "state.agents.seia"
                && o.value == Some(json!({"status":"idle"}))
        });
        assert!(has_del_kalos, "missing del kalos: {ops:?}");
        assert!(has_set_hubris_status, "missing set hubris.status: {ops:?}");
        assert!(has_set_hubris_model, "missing set hubris.model: {ops:?}");
        assert!(has_set_seia, "missing set seia: {ops:?}");
    }

    #[test]
    fn diff_identical_emits_nothing() {
        let v = json!({"a":{"b":1}});
        assert!(diff("state", &v, &v).is_empty());
    }

    #[test]
    fn apply_then_diff_roundtrip() {
        // before --apply(ops)--> after; diff(before, after) must produce an
        // op set that rebuilds after (applying it back to before).
        let before = json!({"agents":{"hubris":{"status":"idle"}}});
        let mut root = before.clone();
        apply_all(
            &mut root,
            &[
                PatchOp::set("agents.hubris.status", json!("busy")),
                PatchOp::set("agents.seia", json!({"status":"idle"})),
            ],
        );
        let after = root.clone();
        let ops = diff("", &before, &after);
        let mut rebuilt = before;
        apply_all(&mut rebuilt, &ops);
        assert_eq!(rebuilt, after);
    }

    // Additional coverage beyond the plana baseline.

    #[test]
    fn diff_non_object_change_sets_leaf() {
        // Object → scalar and scalar → object both collapse to a single
        // leaf `set`, never recursing into non-objects.
        let before = json!({"a":{"b":[1,2]}});
        let after = json!({"a":{"b":"done"}});
        let ops = diff("state", &before, &after);
        assert_eq!(ops, vec![PatchOp::set("state.a.b", json!("done"))]);

        let ops = diff("state", &after, &before);
        assert_eq!(ops, vec![PatchOp::set("state.a.b", json!([1, 2]))]);
    }
}
