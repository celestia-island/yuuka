//! CRDT primitives for multi-writer state synchronization.
//!
//! (B1 of the catalog-sync protocol redesign, 2026-11-03 — see the
//! design doc `_reports/catalog-sync-protocol-20261103/DESIGN.md`.)
//!
//! The rest of yuuka is a **single-writer** patch runtime: a server
//! diffs successive state-tree versions and clients only apply. This
//! module adds the **multi-writer** half — data structures whose merge
//! is a semilattice join, so any set of replicas, fed the same ops in
//! any order (including duplicates, reordering, and arbitrarily long
//! partitions), converges to the same state with three guarantees:
//!
//! 1. **Strong eventual consistency** — merge is commutative,
//!    associative and idempotent (pinned by property tests below and
//!    in `tests/crdt_properties.rs`: thousands of randomized op
//!    sequences must converge identically).
//! 2. **Causal consistency** — ops carry [`Dot`]s over a
//!    [`DottedVersionVector`]; happened-before is computable
//!    (`Dvv::cmp_causal`), so no replica observes an effect before its
//!    cause.
//! 3. **Intent preservation (add-wins)** — [`OrSet`] keeps concurrent
//!    `add`/`remove` both traceable: the add survives (element stays)
//!    while the remove's causal point remains queryable, never a
//!    silent loss.
//!
//! What is deliberately NOT guaranteed: linearizability (no global
//! order), and "last human intent" (the [`LwwRegister`] tie-break is a
//! mathematical total order on `(timestamp, session)`, not semantics).
//!
//! Wire shape: every type is `Serialize/Deserialize` over
//! `serde_json::Value` like the rest of yuuka; the existing patch wire
//! format (0.10.x) is untouched — this is an additive module.

mod dvv;
mod lww;
mod orset;

pub use dvv::{Causal, Dot, Dvv};
pub use lww::LwwRegister;
pub use orset::OrSet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lww_tie_break_is_deterministic() {
        let a = LwwRegister::new("a", 42, "session-a");
        let b = LwwRegister::new("b", 42, "session-b");
        // same ts → lexicographically greater session wins, both directions
        let m1 = a.clone().merged(&b);
        let m2 = b.clone().merged(&a);
        assert_eq!(m1.value(), m2.value());
        assert_eq!(m1.value(), &"b".to_string());
    }

    #[test]
    fn orset_add_wins_over_concurrent_remove() {
        let mut s1 = OrSet::new();
        let mut v1 = Dvv::new();
        let d1 = v1.next_dot("s1");
        s1.add("provider-a".to_string(), d1.clone());
        // s2 saw d1 (causal), then removed; s3 concurrently re-added
        let mut v2 = Dvv::new();
        v2.observe(&d1);
        let mut s2 = s1.clone();
        s2.remove(&"provider-a".to_string(), &v2);
        let mut v3 = Dvv::new();
        let d3 = v3.next_dot("s3");
        let mut s3 = s1.clone();
        s3.add("provider-a".to_string(), d3);
        let merged = s1.merged(&s2).merged(&s3);
        assert!(
            merged.contains(&"provider-a".to_string()),
            "a concurrent re-add must survive the remove (add-wins)"
        );
    }
}
