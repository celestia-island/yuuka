//! Patch operations for the state-tree synchronization protocol.
//!
//! The state tree follows a "single server-side writer" model: every change
//! originates server-side on a tree instance, is diffed against the previous
//! version into a list of [`PatchOp`]s (see [`crate::diff`]) and broadcast to
//! the clients subscribed to the matching viewport. Clients only ever
//! [`apply`] — there is no concurrent-writer conflict handling on their side.
//!
//! The merge semantics are an RFC 7396 (JSON Merge Patch) variant:
//!
//! - `set` — deep merge: object keys merge one by one (the new value wins on
//!   collision), non-objects replace outright;
//! - `replace` — wholesale replacement, no deep merge. For enums /
//!   tagged unions that must be swapped atomically (e.g. `work_status` going
//!   from `{Running:{}}` to `{Completed:{}}`) — the deliberate core
//!   difference from plain RFC 7396;
//! - `del` — delete the key at the path (deleting the root resets it to an
//!   empty object rather than `null`).
//!
//! This is deliberately simpler than full RFC 6902 JSON-Patch
//! (add/remove/replace/move/copy/test): this scenario has no concurrent
//! writers and no rename need, so the extra complexity buys nothing. Lost
//! patches are healed by the periodic full-snapshot fallback pushed per
//! active viewport, so the protocol self-recovers eventually.
//!
//! The serde representation is wire-locked to the historical `plana`
//! `Sync.StatePatch` notification (`op` / `path` / optional `value`, with
//! lowercase op tags); the golden-JSON tests in `tests/patch_compat.rs`
//! pin it byte for byte.
//!
//! Malformed ops are never silently swallowed: a `set`/`replace` whose
//! `value` is missing (the wire format leaves it optional, so a broken
//! producer can omit it) is logged, skipped and counted (see
//! [`apply_all_with_skipped_count`]) instead of no-oping unnoticed. The
//! periodic full-snapshot fallback eventually heals the resulting
//! divergence.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::merge::merge_patch;
use crate::path::split;

/// A single patch operation. `path` is dot-separated (`state.agents.hubris`).
///
/// Serialized, this is exactly the `params` shape of the `Sync.StatePatch`
/// notification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchOp {
    pub op: PatchKind,
    pub path: String,
    /// With `set`/`replace`: the value to write. Always `None` for `del`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// How a [`PatchOp`] writes at its target path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchKind {
    /// Deep merge (RFC 7396): objects merge key by key, scalars/arrays
    /// overwrite. For incremental updates such as partial field changes of
    /// a complete agent object.
    Set,
    /// Wholesale replacement (no deep merge): the new value replaces the old
    /// one entirely. For enums / tagged unions that must be swapped as a
    /// whole (e.g. `work_status` from `{Running:{}}` to `{Completed:{}}`).
    Replace,
    /// Delete the key at the path.
    Del,
}

impl PatchOp {
    /// Builds a deep-merge (`set`) op.
    pub fn set(path: impl Into<String>, value: Value) -> Self {
        Self {
            op: PatchKind::Set,
            path: path.into(),
            value: Some(value),
        }
    }

    /// Builds a wholesale-replacement op.
    pub fn replace(path: impl Into<String>, value: Value) -> Self {
        Self {
            op: PatchKind::Replace,
            path: path.into(),
            value: Some(value),
        }
    }

    /// Builds a key-deletion op.
    pub fn del(path: impl Into<String>) -> Self {
        Self {
            op: PatchKind::Del,
            path: path.into(),
            value: None,
        }
    }

    /// Whether the op carries everything [`apply`] needs to act on it.
    ///
    /// `set`/`replace` write `self.value`, so a missing value cannot be
    /// applied. The wire format keeps `value` optional (`del` never has
    /// one), which lets a malformed producer send a valueless `set` —
    /// such an op is rejected by [`apply`] instead of being silently
    /// ignored.
    pub fn is_well_formed(&self) -> bool {
        !matches!(self.op, PatchKind::Set | PatchKind::Replace) || self.value.is_some()
    }
}

/// Applies a single [`PatchOp`] to `root` (in place).
///
/// - `set` deep merge: descends along the path creating objects as needed,
///   then RFC 7396 merges at the leaf;
/// - `replace`: same descent, but the leaf value is replaced wholesale (no
///   deep merge) — for enums / tagged unions that swap as a whole;
/// - `del`: descends along the path and removes the final key.
///
/// Path descent auto-creates missing intermediate objects and promotes
/// non-object nodes to empty objects along the way. An empty-path `set`
/// merge-patches the root itself; an empty-path `replace` swaps the root;
/// an empty-path `del` resets the root to an empty object (not `null`, so
/// later descents keep working).
///
/// A malformed op (`set`/`replace` without a value) is logged and skipped:
/// the root is left untouched, and the skip is made visible rather than
/// silent so client drift against the server is observable.
pub fn apply(root: &mut Value, op: &PatchOp) {
    let segments = split(&op.path);
    match op.op {
        PatchKind::Set => {
            let Some(new_val) = op.value.clone() else {
                report_malformed(op);
                return;
            };
            if segments.is_empty() {
                *root = merge_patch(std::mem::take(root), new_val);
                return;
            }
            let target = descend_mut(root, &segments);
            *target = merge_patch(std::mem::take(target), new_val);
        }
        PatchKind::Replace => {
            let Some(new_val) = op.value.clone() else {
                report_malformed(op);
                return;
            };
            if segments.is_empty() {
                *root = new_val;
                return;
            }
            let target = descend_mut(root, &segments);
            *target = new_val;
        }
        PatchKind::Del => {
            if segments.is_empty() {
                // Deleting the root = reset to an empty object (not
                // Value::Null, or later path creation would fail).
                *root = Value::Object(Map::new());
                return;
            }
            let (parent_segs, leaf) = segments.split_at(segments.len() - 1);
            let parent = descend_mut(root, parent_segs);
            if let Value::Object(map) = parent {
                map.remove(leaf[0]);
            }
        }
    }
}

/// Applies a batch of ops in order. Used by clients merging a received
/// patch list. Malformed ops (`set`/`replace` without a value) are logged
/// and skipped; see [`apply_all_with_skipped_count`] for a variant that
/// also reports how many were skipped.
pub fn apply_all(root: &mut Value, ops: &[PatchOp]) {
    apply_all_with_skipped_count(root, ops);
}

/// Same as [`apply_all`], but returns the number of malformed ops that
/// were skipped (`set`/`replace` without a value). Valid ops still apply
/// in order; the count lets callers surface client drift against the
/// server instead of silently absorbing it.
pub fn apply_all_with_skipped_count(root: &mut Value, ops: &[PatchOp]) -> usize {
    let mut skipped = 0;
    for op in ops {
        if !op.is_well_formed() {
            skipped += 1;
        }
        apply(root, op);
    }
    if skipped > 0 {
        eprintln!(
            "yuuka::patch: {skipped} of {} patch op(s) were malformed (missing `value`) and skipped",
            ops.len()
        );
    }
    skipped
}

/// Logs a malformed op (finding Y2: it used to no-op silently, letting
/// client state drift away from the server unnoticed). Kept on stderr so
/// the crate needs no logging facade; journald picks it up in service
/// deployments.
fn report_malformed(op: &PatchOp) {
    eprintln!(
        "yuuka::patch: skipping malformed {:?} op at path {:?}: `value` is missing",
        op.op, op.path
    );
}

/// Descends along `segments`, auto-creating missing object keys (as empty
/// objects). Always returns the `&mut Value` at the leaf; root/intermediate
/// non-object nodes are promoted to empty objects to keep the descent going.
fn descend_mut<'a>(root: &'a mut Value, segments: &[&str]) -> &'a mut Value {
    let mut cur = root;
    for seg in segments {
        // A non-object node becomes an empty object first (otherwise the
        // descent could not continue).
        if !cur.is_object() {
            *cur = Value::Object(Map::new());
        }
        let map = cur.as_object_mut().expect("just promoted to object");
        cur = map
            .entry((*seg).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Ported from plana `packages/sync/src/patch.rs` (compat baseline).

    #[test]
    fn set_creates_nested_objects() {
        let mut root = json!({});
        apply(
            &mut root,
            &PatchOp::set("state.agents.hubris", json!({"status":"idle"})),
        );
        assert_eq!(
            root,
            json!({"state":{"agents":{"hubris":{"status":"idle"}}}})
        );
    }

    #[test]
    fn set_deep_merges_objects() {
        let mut root = json!({"state":{"agents":{"hubris":{"status":"idle","model":"gpt"}}}});
        apply(
            &mut root,
            &PatchOp::set("state.agents.hubris", json!({"status":"busy"})),
        );
        // The model key survives (deep merge); status is overwritten.
        assert_eq!(
            root,
            json!({"state":{"agents":{"hubris":{"status":"busy","model":"gpt"}}}})
        );
    }

    #[test]
    fn del_removes_key() {
        let mut root = json!({"state":{"agents":{"hubris":{},"kalos":{}}}});
        apply(&mut root, &PatchOp::del("state.agents.kalos"));
        assert_eq!(root, json!({"state":{"agents":{"hubris":{}}}}));
    }

    #[test]
    fn replace_overrides_without_deep_merge() {
        // replace is for enums / tagged unions: wholesale swap, no deep merge.
        let mut root = json!({"state":{"agents":{"hubris":{"work_status":{"Running":{}}}}}});
        apply(
            &mut root,
            &PatchOp::replace("state.agents.hubris.work_status", json!({"Completed": {}})),
        );
        // Wholesale replacement — no Running residue.
        assert_eq!(
            root,
            json!({"state":{"agents":{"hubris":{"work_status":{"Completed":{}}}}}})
        );
    }

    #[test]
    fn replace_creates_path() {
        let mut root = json!({});
        apply(
            &mut root,
            &PatchOp::replace("state.agents.hubris", json!({"x": 1})),
        );
        assert_eq!(root, json!({"state":{"agents":{"hubris":{"x":1}}}}));
    }

    // Additional coverage beyond the plana baseline.

    #[test]
    fn empty_path_targets_root() {
        // set on the root = merge-patch the root itself.
        let mut root = json!({"a":{"x":1}});
        apply(&mut root, &PatchOp::set("", json!({"a":{"y":2},"b":3})));
        assert_eq!(root, json!({"a":{"x":1,"y":2},"b":3}));

        // replace on the root = swap it wholesale.
        let mut root = json!({"a":1});
        apply(&mut root, &PatchOp::replace("", json!([1, 2])));
        assert_eq!(root, json!([1, 2]));

        // del on the root = reset to an empty object (not null).
        let mut root = json!({"a":1});
        apply(&mut root, &PatchOp::del(""));
        assert_eq!(root, json!({}));
    }

    #[test]
    fn descent_promotes_non_objects() {
        // A scalar mid-path becomes an empty object so the descent continues.
        let mut root = json!({"agents": 7});
        apply(
            &mut root,
            &PatchOp::set("agents.hubris.status", json!("busy")),
        );
        assert_eq!(root, json!({"agents":{"hubris":{"status":"busy"}}}));

        // del through a missing path is a no-op (parents get created, the
        // final key simply is not there).
        let mut root = json!({});
        apply(&mut root, &PatchOp::del("a.b.c"));
        assert_eq!(root, json!({"a":{"b":{}}}));
    }

    // Malformed-op defense: a valueless set/replace (the wire format keeps
    // `value` optional, so this deserializes fine) must not be a silent
    // no-op — it is logged, skipped and counted (finding Y2).

    #[test]
    fn set_without_value_is_skipped_and_counted_not_silent() {
        let malformed: PatchOp = serde_json::from_str(r#"{"op":"set","path":"state.a"}"#).unwrap();
        assert_eq!(malformed.value, None);
        assert!(!malformed.is_well_formed());

        let mut root = json!({"state":{"a":1}});
        apply(&mut root, &malformed);
        assert_eq!(
            root,
            json!({"state":{"a":1}}),
            "a malformed op must leave the state untouched"
        );
    }

    #[test]
    fn replace_without_value_is_skipped_and_counted() {
        let malformed: PatchOp =
            serde_json::from_str(r#"{"op":"replace","path":"state.a"}"#).unwrap();
        assert!(!malformed.is_well_formed());

        let mut root = json!({"state":{"a":1}});
        let skipped = apply_all_with_skipped_count(&mut root, &[malformed]);
        assert_eq!(skipped, 1, "the malformed op must be counted");
        assert_eq!(root, json!({"state":{"a":1}}));
    }

    #[test]
    fn malformed_ops_are_counted_while_valid_ops_still_apply() {
        let ops: Vec<PatchOp> = serde_json::from_str(
            r#"[
                {"op":"set","path":"a","value":1},
                {"op":"set","path":"b"},
                {"op":"replace","path":"c"},
                {"op":"del","path":"x"}
            ]"#,
        )
        .unwrap();
        assert_eq!(ops.iter().filter(|op| !op.is_well_formed()).count(), 2);

        let mut root = json!({});
        let skipped = apply_all_with_skipped_count(&mut root, &ops);
        assert_eq!(skipped, 2);
        assert_eq!(root, json!({"a":1}), "valid ops still apply in order");
    }

    #[test]
    fn well_formed_ops_pass_the_shape_check() {
        assert!(PatchOp::set("a", json!(null)).is_well_formed());
        assert!(PatchOp::replace("a", json!({"x":1})).is_well_formed());
        // `del` never carries a value — that is its documented shape.
        assert!(PatchOp::del("a").is_well_formed());
    }
}
