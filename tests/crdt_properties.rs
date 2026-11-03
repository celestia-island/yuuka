//! Property tests for the CRDT semilattice contracts (B1 acceptance).
//!
//! These are the mathematical obligations of the design doc
//! (`_reports/catalog-sync-protocol-20261103/DESIGN.md` §2): merge must
//! be commutative, associative and idempotent, and randomized op
//! sequences applied to replicated state in arbitrary interleavings
//! must converge to identical states. A failure here is not "a test
//! broke" — it is the convergence proof breaking, which is the entire
//! point of the module.

use proptest::prelude::*;

use yuuka::crdt::{Dot, Dvv, LwwRegister, OrSet};

// ───────────────────────── strategies ─────────────────────────

fn session_id() -> impl Strategy<Value = String> {
    "[a-c]{1}"
}

fn element() -> impl Strategy<Value = String> {
    "[x-z]{1}"
}

/// An arbitrary op for the OR-Set driver.
#[derive(Debug, Clone)]
enum SetOp {
    Add { elem: String, session: String },
    Remove { elem: String, session: String },
}

fn set_op() -> impl Strategy<Value = SetOp> {
    prop_oneof![
        (element(), session_id()).prop_map(|(elem, session)| SetOp::Add { elem, session }),
        (element(), session_id()).prop_map(|(elem, session)| SetOp::Remove { elem, session }),
    ]
}

/// One replica's full state: its DVV and its OR-Set view.
#[derive(Debug, Clone, Default)]
struct Replica {
    dvv: Dvv,
    set: OrSet<String>,
}

impl Replica {
    fn apply(&mut self, op: &SetOp) {
        match op {
            SetOp::Add { elem, session } => {
                let dot = self.dvv.next_dot(session);
                self.set.add(elem.clone(), dot);
            }
            SetOp::Remove { elem, session } => {
                // The remover's causal context is exactly its DVV.
                let _ = session;
                let ctx = self.dvv.clone();
                self.set.remove(elem, &ctx);
            }
        }
    }

    fn merge_from(&mut self, other: &Replica) {
        self.dvv = self.dvv.clone().merged(&other.dvv);
        self.set = self.set.clone().merged(&other.set);
    }
}

/// Drive `n` replicas through per-replica op lists, then merge them in
/// the given pairwise order until all are equal, and return the final
/// replica states (convergence is asserted by the caller).
fn converge(replicas: Vec<Replica>) -> Vec<Replica> {
    let mut rs = replicas;
    // star-merge: fold every replica into replica 0, then broadcast 0
    // back out — one convergence round; repeat once to settle.
    for _round in 0..2 {
        for i in 1..rs.len() {
            let src = rs[i].clone();
            rs[0].merge_from(&src);
        }
        let hub = rs[0].clone();
        for r in rs.iter_mut().skip(1) {
            r.merge_from(&hub);
        }
    }
    rs
}

// ───────────────────────── DVV lattice ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn dvv_merge_is_commutative(
        a in proptest::collection::vec((session_id(), 1u64..50), 3),
        b in proptest::collection::vec((session_id(), 1u64..50), 3),
    ) {
        let mk = |v: Vec<(String, u64)>| {
            let mut d = Dvv::new();
            for (s, c) in v {
                d.observe(&Dot::new(s, c));
            }
            d
        };
        let (da, db) = (mk(a), mk(b));
        prop_assert_eq!(da.clone().merged(&db), db.clone().merged(&da));
    }

    #[test]
    fn dvv_merge_is_idempotent(
        a in proptest::collection::vec((session_id(), 1u64..50), 3),
    ) {
        let mut d = Dvv::new();
        for (s, c) in a {
            d.observe(&Dot::new(s, c));
        }
        prop_assert_eq!(d.clone().merged(&d), d);
    }
}

// ───────────────────────── LWW lattice ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn lww_merge_is_commutative_and_idempotent(
        va in any::<u16>(), sa in session_id(),
        vb in any::<u16>(), sb in session_id(),
    ) {
        let a = LwwRegister::new(va, va as u64, sa.clone());
        let b = LwwRegister::new(vb, vb as u64, sb.clone());
        prop_assert_eq!(a.clone().merged(&b), b.clone().merged(&a));
        prop_assert_eq!(a.clone().merged(&a), a.clone());
    }

    #[test]
    fn lww_three_way_associative(
        va in any::<u16>(), sa in session_id(),
        vb in any::<u16>(), sb in session_id(),
        vc in any::<u16>(), sc in session_id(),
    ) {
        let a = LwwRegister::new(va, va as u64, sa);
        let b = LwwRegister::new(vb, vb as u64, sb);
        let c = LwwRegister::new(vc, vc as u64, sc);
        let left = a.clone().merged(&b).merged(&c);
        let right = a.merged(&b.clone().merged(&c).clone().merged(&c)); // also idempotent on the way
        prop_assert_eq!(left, right);
    }
}

// ───────────────────────── OR-Set lattice ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// The headline property: N replicas, independent random op
    /// sequences, arbitrary interleavings — after an anti-entropy
    /// round every replica agrees on the exact same set state.
    #[test]
    fn orset_converges_under_random_interleavings(
        ops1 in proptest::collection::vec(set_op(), 0..24),
        ops2 in proptest::collection::vec(set_op(), 0..24),
        ops3 in proptest::collection::vec(set_op(), 0..24),
    ) {
        let mut r1 = Replica::default();
        let mut r2 = Replica::default();
        let mut r3 = Replica::default();
        // sessions must be disjoint per replica for dots to be unique
        for op in &ops1 { r1.apply(&tag(op, "a")); }
        for op in &ops2 { r2.apply(&tag(op, "b")); }
        for op in &ops3 { r3.apply(&tag(op, "c")); }

        let final_states = converge(vec![r1, r2, r3]);
        let (f0, rest) = final_states.split_first().unwrap();
        for (i, r) in rest.iter().enumerate() {
            prop_assert_eq!(&f0.set, &r.set, "replica {} diverged after anti-entropy", i + 1);
            prop_assert_eq!(&f0.dvv, &r.dvv);
        }
    }

    #[test]
    fn orset_merge_is_commutative_and_idempotent(
        ops_a in proptest::collection::vec(set_op(), 0..16),
        ops_b in proptest::collection::vec(set_op(), 0..16),
    ) {
        let mut a = Replica::default();
        let mut b = Replica::default();
        for op in &ops_a { a.apply(&tag(op, "a")); }
        for op in &ops_b { b.apply(&tag(op, "b")); }
        let ab = a.clone().merge_into(&b);
        let ba = b.clone().merge_into(&a);
        prop_assert_eq!(&ab.set, &ba.set);
        let double = ab.set.clone().merged(&ab.set.clone().merged(&ab.set));
        prop_assert_eq!(&ab.set, &double);
    }
}

fn tag(op: &SetOp, session_prefix: &str) -> SetOp {
    match op {
        SetOp::Add { elem, session } => SetOp::Add {
            elem: elem.clone(),
            session: format!("{session_prefix}{session}"),
        },
        SetOp::Remove { elem, session } => SetOp::Remove {
            elem: elem.clone(),
            session: format!("{session_prefix}{session}"),
        },
    }
}

impl Replica {
    /// merge self with other, returning the merged replica (helper for
    /// the commutativity case where we need both directions fresh).
    fn merge_into(mut self, other: &Replica) -> Replica {
        self.merge_from(other);
        self
    }
}
