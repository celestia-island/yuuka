//! RFC 7396 (JSON Merge Patch) core.
//!
//! [`merge_patch`] realizes the `MergePatch(Target, Patch)` pseudocode of
//! RFC 7396 section 2, with one deliberate deviation inherited from the
//! plana implementation (see below):
//!
//! - both sides objects → merged key by key; a `null` value in the patch
//!   removes that key from the target;
//! - anything else → the patch replaces the target wholesale.
//!
//! The deviations (both pinned in `tests/patch_compat.rs`): when the
//! *patch* is an object but the *target* is not, strict RFC 7396 would
//! reset the target to `{}` before merging (appendix-A case 14); this
//! implementation instead returns the object patch verbatim, which lets
//! [`PatchOp::set`](crate::patch::PatchOp::set) write null-bearing objects
//! wholesale over non-object leaves. And the value of a key absent from
//! the target is inserted verbatim without recursing (appendix-A case 15),
//! so nulls nested inside newly-added objects survive. Together these keep
//! the diff/apply roundtrip sound. The remaining trade-off — a top-level
//! `null` member inside a wholesale-written object value is eaten when the
//! target already is an object — is inherent to RFC 7396 ("not appropriate
//! for documents that make use of explicit null values"); the state-tree
//! domain uses nulls only as tombstones at leaf positions.
//!
//! The official appendix-A test cases of the RFC (plus the pinned A.14
//! deviation) are exercised in `tests/patch_compat.rs`.

use serde_json::Value;

/// RFC 7396 merge patch: `target` is the current value, `patch` the new one.
///
/// - both objects → recursive key-wise merge (a `null` patch value deletes
///   the target key, per RFC 7396 semantics);
/// - otherwise → `patch` overwrites `target` outright (including
///   `patch == null`, which yields `null`). Note that an *object* patch
///   over a non-object target also lands here, returning the patch
///   verbatim — the appendix-A case-14 deviation documented at the module
///   level.
///
/// A target `null` under a key the patch does not mention is preserved —
/// only *patch* nulls delete (RFC 7396 appendix A.13).
pub fn merge_patch(target: Value, patch: Value) -> Value {
    match (target, patch) {
        (Value::Object(mut t), Value::Object(p)) => {
            for (k, v) in p {
                match v {
                    // null = delete the key (RFC 7396 semantics).
                    Value::Null => {
                        t.remove(&k);
                    }
                    pv => {
                        let merged = match t.remove(&k) {
                            Some(tv) => merge_patch(tv, pv.clone()),
                            None => pv,
                        };
                        t.insert(k, merged);
                    }
                }
            }
            Value::Object(t)
        }
        // Any non-object side → the patch wins (patch = null means deletion).
        (_, p) => p,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Ported from plana `packages/sync/src/patch.rs` (compat baseline).

    #[test]
    fn merge_patch_object_recursive() {
        let t = json!({"a":{"x":1,"y":2},"b":3});
        let p = json!({"a":{"y":20,"z":3},"c":4});
        assert_eq!(
            merge_patch(t, p),
            json!({"a":{"x":1,"y":20,"z":3},"b":3,"c":4})
        );
    }

    #[test]
    fn merge_patch_null_deletes_key() {
        let t = json!({"a":{"x":1,"y":2}});
        let p = json!({"a":{"x":null}});
        assert_eq!(merge_patch(t, p), json!({"a":{"y":2}}));
    }
}
