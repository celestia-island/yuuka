//! Dotted version vectors — the causal backbone.
//!
//! One [`Dvv`] per replica tracks, per writer session, the highest op
//! counter it has **observed** (its own or others'). A [`Dot`]
//! (`(session, counter)`) names one op uniquely. The pair gives the
//! three-way causal judgement the protocol needs:
//!
//! - `a.dot ⊑ b.dvv` → a happened-before b's context;
//! - `b.dot ⊑ a.dvv` → the converse;
//! - neither → **concurrent** (both orders are legal; the CRDT merge
//!   resolves them deterministically).
//!
//! Merge is entry-wise `max` — itself commutative, associative and
//! idempotent, so DVVs merge in any order and re-merge for free.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Names one operation uniquely: the issuing session plus its
/// per-session monotonic counter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Dot {
    pub session: String,
    pub counter: u64,
}

impl Dot {
    pub fn new(session: impl Into<String>, counter: u64) -> Self {
        Self {
            session: session.into(),
            counter,
        }
    }
}

/// Per-replica causal context: session → highest counter observed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dvv {
    seen: BTreeMap<String, u64>,
}

/// Three-way causal judgement between a dot and a context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Causal {
    /// The dot is covered by the context (happened-before it).
    Included,
    /// The dot is unknown to the context.
    Concurrent,
}

impl Dvv {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate this session's next dot and observe it. Sessions MUST
    /// be unique per replica instance (generate once, persist).
    pub fn next_dot(&mut self, session: impl AsRef<str>) -> Dot {
        let s = session.as_ref();
        let next = self.seen.get(s).copied().unwrap_or(0) + 1;
        self.seen.insert(s.to_string(), next);
        Dot::new(s.to_string(), next)
    }

    /// Fold one observed dot into the context (idempotent).
    pub fn observe(&mut self, dot: &Dot) {
        let e = self.seen.entry(dot.session.clone()).or_insert(0);
        if dot.counter > *e {
            *e = dot.counter;
        }
    }

    /// Three-way judgement: is `dot` covered by this context?
    pub fn causal(&self, dot: &Dot) -> Causal {
        match self.seen.get(&dot.session) {
            Some(seen) if *seen >= dot.counter => Causal::Included,
            _ => Causal::Concurrent,
        }
    }

    /// Entry-wise max — the semilattice join. Commutative, associative,
    /// idempotent by construction (pinned in the property tests).
    pub fn merged(mut self, other: &Dvv) -> Dvv {
        for (s, c) in &other.seen {
            let e = self.seen.entry(s.clone()).or_insert(0);
            if *c > *e {
                *e = *c;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dots_are_strictly_monotonic_per_session() {
        let mut v = Dvv::new();
        let d1 = v.next_dot("s1");
        let d2 = v.next_dot("s1");
        assert_eq!(d1.counter, 1);
        assert_eq!(d2.counter, 2);
    }

    #[test]
    fn causal_judgement_is_three_way() {
        let mut v1 = Dvv::new();
        let d1 = v1.next_dot("s1"); // v1: s1@1

        let mut v2 = Dvv::new();
        v2.observe(&d1); // v2 saw s1@1
        let d2 = v2.next_dot("s2"); // v2: s1@1, s2@1

        assert_eq!(v2.causal(&d1), Causal::Included);
        assert_eq!(v1.causal(&d2), Causal::Concurrent); // v1 never saw s2
        let m = v1.merged(&v2);
        assert_eq!(m.causal(&d2), Causal::Included);
    }

    #[test]
    fn merge_is_commutative_and_idempotent_here() {
        let mut a = Dvv::new();
        a.next_dot("a");
        let mut b = Dvv::new();
        b.next_dot("b");
        assert_eq!(a.clone().merged(&b), b.clone().merged(&a));
        assert_eq!(a.clone().merged(&a), a);
    }
}
