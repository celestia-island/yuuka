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
    /// Audit yuuka-R1-2: an op tag this build does not know. The whole
    /// patch list used to hard-fail on deserialization (mixed-version
    /// deployments lost every patch until the ~3s snapshot self-heal);
    /// unknown tags now decode into this variant and are skipped with
    /// the rest of the malformed ops.
    #[serde(other)]
    Unknown,
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
    // Audit yuuka-R1-1: refuse op paths deeper than MAX_DEPTH — the
    // resulting Value tree would abort the process on recursive drop.
    // Audit yuuka-R1-2: unknown op tags decode into `Unknown` and are
    // skipped (the snapshot self-heal covers the missing mutation).
    if op.op == PatchKind::Unknown {
        eprintln!(
            "yuuka::patch: skipping unknown op tag at path {:?}",
            op.path
        );
        return;
    }
    let depth = op.path.split('.').count();
    if depth > MAX_DEPTH {
        eprintln!(
            "yuuka::patch: refusing op at path depth {depth} (> {MAX_DEPTH}): \
             deep paths can abort the process via recursive drop"
        );
        return;
    }

    let segments = split(&op.path);
    match op.op {
        PatchKind::Unknown => {
            // Already logged and skipped by the audit guard above.
        }
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
            // Audit yuuka-R1-3: del on a nonexistent path is a TRUE
            // no-op — descending via descend_mut would auto-create the
            // ghost parent chain, leaving client state permanently
            // drifted from the server (snapshot-only self-heal).
            let (parent_segs, leaf) = segments.split_at(segments.len() - 1);
            // The existence probe walks immutably; only when the whole
            // parent chain exists does descend_mut run (which would
            // otherwise auto-create the ghost chain).
            let parent_exists = {
                let mut probe = &*root;
                let mut ok = true;
                for seg in parent_segs {
                    match probe.get(*seg) {
                        Some(v) if v.is_object() => probe = v,
                        _ => {
                            ok = false;
                            break;
                        }
                    }
                }
                ok
            };
            if !parent_exists {
                return;
            }
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

/// Audit yuuka-R1-1: the maximum op-path depth. Deeper paths build
/// arbitrarily deep `Value` trees whose recursive Drop (and the recursive
/// merge/diff walks) abort the process on stack overflow — an
/// uncataught abort, not a panic. 128 aligns with serde_json's default
/// recursion limit.
pub const MAX_DEPTH: usize = 128;

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

        // Audit yuuka-R1-3: del through a missing path is a TRUE no-op —
        // the ghost parent chain is no longer created (the old behavior
        // left client state permanently drifted from the server).
        let mut root = json!({});
        apply(&mut root, &PatchOp::del("a.b.c"));
        assert_eq!(root, json!({}));
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

#[cfg(test)]
mod hardening_tests {
    use super::*;
    use serde_json::json;

    /// Audit yuuka-R1-1: op paths deeper than MAX_DEPTH are refused —
    /// deep trees would abort the process via recursive Drop.
    #[test]
    fn deep_paths_are_refused() {
        let mut root = json!({});
        let deep_path = vec!["a"; MAX_DEPTH + 1].join(".");
        apply(&mut root, &PatchOp::set(&deep_path, json!(1)));
        assert_eq!(root, json!({}), "over-deep path must not build a tree");
        // At the boundary itself the op still applies.
        let boundary = vec!["a"; MAX_DEPTH].join(".");
        apply(&mut root, &PatchOp::set(&boundary, json!(1)));
        assert!(root
            .pointer(&format!("/{}", vec!["a"; MAX_DEPTH].join("/")))
            .is_some());
    }

    /// Audit yuuka-R1-2: unknown op tags decode into `Unknown` and are
    /// skipped instead of failing the whole patch list.
    #[test]
    fn unknown_op_tag_is_skipped() {
        let raw = r#"[{"op":"set","path":"x","value":1},{"op":"transmogrify","path":"y"}]"#;
        let ops: Vec<PatchOp> = serde_json::from_str(raw).expect("decode must not hard-fail");
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[1].op, PatchKind::Unknown);

        let mut root = json!({});
        apply_all(&mut root, &ops);
        // The unknown op is skipped; the known one applied.
        assert_eq!(root, json!({"x": 1}));
    }

    /// Audit yuuka-R1-3: del through a missing parent chain is a true
    /// no-op — no ghost intermediate objects.
    #[test]
    fn del_missing_path_is_true_noop() {
        let mut root = json!({});
        apply(&mut root, &PatchOp::del("ghost.path.here"));
        assert_eq!(root, json!({}), "no ghost parent chain may be created");
    }
}
