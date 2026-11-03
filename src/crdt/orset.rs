//! Add-wins observed-remove set — row-level existence under
//! multi-writer merge (the OR-Set).
//!
//! Each element carries the **set of dots** that currently assert it
//! (an element re-added after a removal gains a fresh dot — that is
//! what makes "add wins" precise). A removal tombstones exactly the
//! dots the remover could see (its causal context at remove time), so:
//!
//! - a remove that **causally follows** an add kills that add
//!   (`contains` goes false once no live dot remains);
//! - a **concurrent** add survives the remove (its dot was invisible
//!   to the remover) — add-wins;
//! - both intents stay traceable: the tombstone dots record the
//!   remove, the surviving dots record the add.
//!
//! Merge is point-wise union of both maps — commutative, associative,
//! idempotent.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{Dot, Dvv};

/// A multi-writer set with add-wins semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "E: serde::Serialize + Ord",
    deserialize = "E: serde::de::Deserialize<'de> + Ord",
))]
pub struct OrSet<E> {
    /// element → live-plus-shadowed add dots (a dot stays here even
    /// after tombstoning; `contains` consults the tombstones).
    adds: BTreeMap<E, BTreeSet<Dot>>,
    /// dots removed by some replica (the remove intents).
    tombstones: BTreeSet<Dot>,
}

// Manual Default: the derive would demand `E: Default`, which maps
// don't need (their emptiness is independent of the key type).
impl<E> Default for OrSet<E> {
    fn default() -> Self {
        Self {
            adds: BTreeMap::new(),
            tombstones: BTreeSet::new(),
        }
    }
}

impl<E: Ord + Clone> OrSet<E> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add the element under a fresh dot (allocate it from the
    /// replica's own DVV first: `let dot = dvv.next_dot(session)`).
    pub fn add(&mut self, elem: E, dot: Dot) {
        self.adds.entry(elem).or_default().insert(dot);
        // A re-add of a dot we ourselves tombstoned locally revives the
        // element for OUR view until merge — but a fresh dot is the
        // contract (`next_dot` never repeats), so this is a no-op guard.
    }

    /// Remove the element, tombstoning every add dot visible to the
    /// remover's causal context. Concurrent adds (invisible dots)
    /// survive — add-wins.
    pub fn remove(&mut self, elem: &E, context: &Dvv) {
        if let Some(dots) = self.adds.get(elem) {
            let dead: Vec<Dot> = dots
                .iter()
                .filter(|d| context.causal(d) == super::Causal::Included)
                .cloned()
                .collect();
            for d in dead {
                self.tombstones.insert(d);
            }
        }
    }

    /// The element is present iff at least one of its add dots is not
    /// tombstoned.
    pub fn contains(&self, elem: &E) -> bool {
        self.adds
            .get(elem)
            .is_some_and(|dots| dots.iter().any(|d| !self.tombstones.contains(d)))
    }

    /// Live elements (unspecified order — `BTreeMap` keys, so
    /// deterministic by `E: Ord`; useful for snapshot projection).
    pub fn elements(&self) -> impl Iterator<Item = &E> {
        self.adds.keys().filter(|e| self.contains(e))
    }

    /// The semilattice join: union of add-dot sets and tombstones.
    pub fn merged(mut self, other: &OrSet<E>) -> OrSet<E> {
        for (e, dots) in &other.adds {
            self.adds
                .entry(e.clone())
                .or_default()
                .extend(dots.iter().cloned());
        }
        self.tombstones.extend(other.tombstones.iter().cloned());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(dots: &[Dot]) -> Dvv {
        let mut v = Dvv::new();
        for d in dots {
            v.observe(d);
        }
        v
    }

    #[test]
    fn causally_later_remove_wins() {
        let mut s = OrSet::new();
        let mut v = Dvv::new();
        let d1 = v.next_dot("s1");
        s.add("row".to_string(), d1.clone());
        assert!(s.contains(&"row".to_string()));
        s.remove(&"row".to_string(), &ctx(&[d1]));
        assert!(
            !s.contains(&"row".to_string()),
            "a remove that saw the add must kill it"
        );
    }

    #[test]
    fn concurrent_add_survives_remove() {
        let mut base = OrSet::new();
        let mut v0 = Dvv::new();
        let d0 = v0.next_dot("s0");
        base.add("row".to_string(), d0.clone());

        // remover saw only d0
        let mut r = base.clone();
        r.remove(&"row".to_string(), &ctx(&[d0]));
        // concurrent adder never saw the remove
        let mut a = base.clone();
        let mut va = Dvv::new();
        let d2 = va.next_dot("s2");
        a.add("row".to_string(), d2);

        let merged = base.clone().merged(&r).merged(&a);
        assert!(
            merged.contains(&"row".to_string()),
            "the concurrent add dot survives"
        );
        // and the merged view agrees from the other merge direction
        let merged2 = a.merged(&r).merged(&base);
        assert_eq!(merged, merged2);
    }

    #[test]
    fn merge_is_idempotent() {
        let mut s = OrSet::new();
        let mut v = Dvv::new();
        s.add("x".to_string(), v.next_dot("s"));
        assert_eq!(s.clone().merged(&s), s);
    }
}
