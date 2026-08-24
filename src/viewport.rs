//! Viewport snapshot computation — path-prefix subscription + subtree
//! cropping.
//!
//! A client declares which path prefixes it is looking at (e.g.
//! `state.agents`); the server crops the global state tree down to the
//! subtrees under those prefixes and pushes them as a `Sync.StateSnapshot`.
//! This is the server side of "what is visible on screen": a viewport is a
//! set of path prefixes, independent of the scroll position of any concrete
//! list (deliberately kept simple, stable and predictable — windowing of
//! large lists is a later, per-domain concern that reuses chunk_history's
//! virtual-total pattern, not something handled at this layer).
//!
//! Snapshots serve two purposes:
//!
//! 1. **First frame**: `state.subscribe` returns the current viewport
//!    snapshot immediately (replacing the old fetchInitialData pull).
//! 2. **Periodic fallback**: the ws_bridge pushes a full snapshot for every
//!    active viewport every ~3s — loss recovery and eventual consistency
//!    (self-healing even when incremental patches are lost).
//!
//! Migration note: this is `plana` `sync::snapshot`
//! (`packages/sync/src/snapshot.rs`); the `split_path` calls are routed
//! through [`crate::path::split`], whose semantics are identical.

use serde_json::{Map, Value};

use crate::path::split;

/// Returns whether `path` falls inside any viewport prefix (bidirectional
/// matching).
///
/// An op is inside the viewport iff, for one of the viewport prefixes:
/// - **the op is a descendant of (or equal to) the prefix**: op
///   `state.agents.hubris` hits prefix `state.agents` and `state`. The
///   common case (leaf-level increments).
/// - **the op is an ancestor of the prefix**: op `state` (a whole-subtree
///   set) hits prefixes `state.x` and `state.agents.hubris` — because that
///   set covers the part the viewport cares about. This happens when the
///   server-side diff merges a whole fresh subtree into a single
///   branch-level op (cheaper than splitting it into leaf ops); the client
///   must receive it to apply.
///
/// The empty-string prefix counts as the root — it matches every path.
pub fn path_in_viewport(path: &str, viewport: &[String]) -> bool {
    if viewport.is_empty() {
        return false;
    }
    let segs = split(path);
    viewport.iter().any(|p| overlaps(p, &segs))
}

/// Whether `prefix` and `path_segs` overlap (one is a prefix of the other).
fn overlaps(prefix: &str, path_segs: &[&str]) -> bool {
    let prefix_segs = split(prefix);
    if prefix_segs.is_empty() {
        return true; // Empty prefix = root = matches everything.
    }
    let n = prefix_segs.len().min(path_segs.len());
    // The first n segments must agree (bidirectional: whichever is shorter
    // is the potential prefix).
    path_segs[..n]
        .iter()
        .zip(prefix_segs[..n].iter())
        .all(|(p, pre)| p == pre)
}

/// One-directional prefix check: whether `prefix` is a prefix of
/// `path_segs` (the prefix is shorter or equally long). Used by
/// `normalize_prefixes` to tell whether one prefix is contained by another
/// (closer to the root).
fn is_prefix(prefix: &str, path_segs: &[&str]) -> bool {
    let prefix_segs = split(prefix);
    if prefix_segs.is_empty() {
        return true;
    }
    if prefix_segs.len() > path_segs.len() {
        return false;
    }
    path_segs[..prefix_segs.len()]
        .iter()
        .zip(prefix_segs.iter())
        .all(|(p, pre)| p == pre)
}

/// Crops the subtrees of `root` covered by `viewport`.
///
/// Returns an object with each prefix's subtree attached at the prefix's
/// full (rebuilt) dotted path. The implementation descends for each prefix
/// and merges the found value into the result object. When several
/// prefixes are in a parent/child relation (e.g. `state` and
/// `state.agents`), the shorter prefix's subtree would already contain the
/// longer one's, so ordering by descending prefix length (shorter writes
/// first, longer overwrites the more precise subkey) would only duplicate
/// data. For simplicity this implementation instead uses *normalized
/// prefixes*: sub-prefixes contained by another prefix are dropped
/// (`state.agents` is contained by `state` → only `state` is kept),
/// guaranteeing a duplicate-free result object.
pub fn snapshot(root: &Value, viewport: &[String]) -> Value {
    if viewport.is_empty() {
        return Value::Object(Map::new());
    }
    let normalized = normalize_prefixes(viewport);
    let mut out = Map::new();
    for p in &normalized {
        let sub = descend_const(root, p);
        // Attach the subtree onto the result object: the full path of the
        // prefix is rebuilt.
        insert_at_path(&mut out, p, sub);
    }
    Value::Object(out)
}

/// Drops sub-prefixes that are contained by another prefix (avoiding
/// duplicate cropping). The input does not need to be ordered.
///
/// Example: `["state.agents", "state"]` → `["state"]` (the latter contains
/// the former).
fn normalize_prefixes(prefixes: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for p in prefixes {
        let p_segs = split(p);
        // p is contained by an already-kept prefix → skip.
        let dominated = kept.iter().any(|k| is_prefix(k, &p_segs));
        if dominated {
            continue;
        }
        // Drop already-kept prefixes that are contained by p (p is
        // shorter / closer to the root).
        let p_clone = p.clone();
        kept.retain(|k| {
            let k_segs = split(k);
            !is_prefix(&p_clone, &k_segs)
        });
        kept.push(p_clone);
    }
    kept
}

/// Descends along a dotted path inside `root` to fetch the value
/// (read-only). A missing segment yields Null.
fn descend_const(root: &Value, path: &str) -> Value {
    let segs = split(path);
    let mut cur = root;
    for seg in segs {
        match cur {
            Value::Object(map) => match map.get(seg) {
                Some(v) => cur = v,
                None => return Value::Null,
            },
            _ => return Value::Null,
        }
    }
    cur.clone()
}

/// Creates the object chain along `path` inside `obj` and places `value`
/// at the leaf.
fn insert_at_path(obj: &mut Map<String, Value>, path: &str, value: Value) {
    let segs = split(path);
    if segs.is_empty() {
        // Root path: merge value's keys into obj (value should be an
        // object; same-key entries overwrite).
        if let Value::Object(v) = value {
            for (k, vv) in v {
                obj.insert(k, vv);
            }
        }
        return;
    }
    let mut cur = obj;
    let (last_segs, leaf) = segs.split_at(segs.len() - 1);
    for seg in last_segs {
        let entry = cur
            .entry((*seg).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        cur = entry.as_object_mut().expect("just promoted");
    }
    cur.insert(leaf[0].to_string(), value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Ported from plana `packages/sync/src/snapshot.rs` (compat baseline).

    #[test]
    fn prefix_match_basic() {
        let vp = vec!["state.agents".to_string()];
        assert!(path_in_viewport("state.agents.hubris", &vp));
        assert!(path_in_viewport("state.agents", &vp));
        assert!(!path_in_viewport("state.devices", &vp));
    }

    #[test]
    fn prefix_match_ancestor_op_hits_descendant_viewport() {
        // Key point: the server-side diff can emit branch-level ops (e.g.
        // set("state", {agents:{...}}) when a whole subtree is new). A
        // client subscribed to the subpath state.agents.hubris must be hit
        // by that branch-level op because it covers the part of the tree
        // the viewport cares about (bidirectional matching).
        let vp = vec!["state.agents.hubris".to_string()];
        assert!(
            path_in_viewport("state", &vp),
            "ancestor op (state) must hit descendant viewport (state.agents.hubris)"
        );
        assert!(path_in_viewport("state.agents", &vp));
        assert!(path_in_viewport("state.agents.hubris", &vp));
        assert!(path_in_viewport("state.agents.hubris.status", &vp));
        // Completely unrelated paths still miss.
        assert!(!path_in_viewport("devices.n1", &vp));
    }

    #[test]
    fn prefix_match_root_matches_all() {
        let vp = vec!["".to_string()];
        assert!(path_in_viewport("anything.here", &vp));
    }

    #[test]
    fn empty_viewport_matches_nothing() {
        let vp: Vec<String> = vec![];
        assert!(!path_in_viewport("state.agents", &vp));
    }

    #[test]
    fn snapshot_crops_subtree() {
        let root = json!({
            "state": {
                "agents": {"hubris": {"status": "idle"}},
                "devices": {"node1": {"online": true}}
            }
        });
        let snap = snapshot(&root, &["state.agents".to_string()]);
        assert_eq!(
            snap,
            json!({"state": {"agents": {"hubris": {"status": "idle"}}}})
        );
        // devices is outside the viewport and must not appear.
        assert!(snap.get("devices").is_none() || snap.get("state.devices").is_none());
    }

    #[test]
    fn snapshot_multiple_prefixes() {
        let root = json!({
            "state": {
                "agents": {"hubris": 1},
                "devices": {"n1": 2},
                "reports": {"r1": 3}
            }
        });
        let snap = snapshot(&root, &["state.agents".into(), "state.devices".into()]);
        assert_eq!(
            snap,
            json!({"state": {"agents": {"hubris": 1}, "devices": {"n1": 2}}})
        );
    }

    #[test]
    fn snapshot_normalizes_nested_prefixes() {
        let root = json!({"state": {"agents": {"a": 1}, "devices": {"d": 2}}});
        // state.agents is contained by state → only state is kept.
        let snap = snapshot(&root, &["state".into(), "state.agents".into()]);
        assert_eq!(
            snap,
            json!({"state": {"agents": {"a": 1}, "devices": {"d": 2}}})
        );
    }

    #[test]
    fn snapshot_empty_viewport_returns_empty() {
        let root = json!({"state": {"a": 1}});
        let snap = snapshot(&root, &[]);
        assert!(snap.as_object().map(|m| m.is_empty()).unwrap_or(true));
    }

    #[test]
    fn snapshot_root_prefix_returns_whole_tree() {
        let root = json!({"state": {"a": 1}, "meta": {"b": 2}});
        let snap = snapshot(&root, &["".to_string()]);
        assert_eq!(snap, root);
    }

    // Additional boundary cases beyond the plana baseline.

    #[test]
    fn prefix_match_any_of_several_prefixes() {
        let vp = vec!["state.agents".into(), "meta.uptime".into()];
        assert!(path_in_viewport("state.agents.hubris", &vp));
        assert!(path_in_viewport("meta.uptime", &vp));
        // Ancestor of either prefix hits too (branch-level op).
        assert!(path_in_viewport("meta", &vp));
        assert!(!path_in_viewport("state.devices", &vp));
        assert!(!path_in_viewport("logs", &vp));
    }

    #[test]
    fn prefix_match_root_prefix_dominates_the_rest() {
        // The root prefix coexisting with others still matches everything.
        let vp = vec!["state".into(), "".to_string()];
        assert!(path_in_viewport("anything.here", &vp));
        assert!(path_in_viewport("state.agents", &vp));
    }

    #[test]
    fn snapshot_normalizes_in_any_input_order() {
        let root = json!({"a": {"b": {"c": 1}, "x": 2}});
        let long_first = snapshot(&root, &["a.b".into(), "a".into()]);
        let short_first = snapshot(&root, &["a".into(), "a.b".into()]);
        // Whichever order the prefixes arrive in, a.b is contained by a,
        // so both normalize to ["a"].
        assert_eq!(long_first, short_first);
        assert_eq!(long_first, json!({"a": {"b": {"c": 1}, "x": 2}}));
    }

    #[test]
    fn snapshot_three_level_normalization() {
        let root = json!({"a": {"b": {"c": 1, "d": 2}, "e": 3}});
        let snap = snapshot(&root, &["a.b.c".into(), "a".into(), "a.b.d".into()]);
        assert_eq!(snap, json!({"a": {"b": {"c": 1, "d": 2}, "e": 3}}));
    }

    #[test]
    fn snapshot_missing_segment_yields_null() {
        let root = json!({"state": {"agents": {"a": 1}}});
        // The prefix path does not exist in the tree: the rebuilt path is
        // still present, with a Null leaf.
        let snap = snapshot(&root, &["state.missing".to_string()]);
        assert_eq!(snap, json!({"state": {"missing": null}}));
        // Descending through a non-object scalar also yields Null.
        let snap = snapshot(&root, &["state.agents.a.deeper".to_string()]);
        assert_eq!(snap, json!({"state": {"agents": {"a": {"deeper": null}}}}));
    }

    #[test]
    fn snapshot_deeply_nested_prefix_rebuilds_every_level() {
        let root = json!({"a": {"b": {"c": {"d": {"e": 42}}}}});
        let snap = snapshot(&root, &["a.b.c.d".to_string()]);
        assert_eq!(snap, json!({"a": {"b": {"c": {"d": {"e": 42}}}}}));
    }

    #[test]
    fn snapshot_scalar_root_with_root_prefix_returns_empty_object() {
        // A non-object root merged at the empty (root) prefix contributes
        // no keys, so the snapshot is the empty object.
        let root = json!(42);
        let snap = snapshot(&root, &["".to_string()]);
        assert_eq!(snap, json!({}));
    }

    #[test]
    fn snapshot_duplicate_prefixes_are_kept_once() {
        let root = json!({"state": {"a": 1}});
        let snap = snapshot(&root, &["state".into(), "state".into()]);
        assert_eq!(snap, json!({"state": {"a": 1}}));
    }
}
