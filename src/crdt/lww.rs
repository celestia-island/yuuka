//! Last-writer-wins register — scalar fields under multi-writer merge.
//!
//! Each write stamps `(timestamp, session)`; merge takes the
//! lexicographically greater pair, so every replica picks the same
//! winner regardless of merge order — including exact ties broken by
//! session id. Ties on `(ts, session)` are the same write
//! (idempotent).
//!
//! The timestamp is caller-supplied (wall clock millis, or any
//! monotonic source; the structure only needs a total order on it).

use serde::{Deserialize, Serialize};

/// A multi-writer scalar: the value plus its causal stamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwwRegister<T> {
    value: T,
    ts: u64,
    session: String,
}

impl<T: Clone> LwwRegister<T> {
    pub fn new(value: T, ts: u64, session: impl Into<String>) -> Self {
        Self {
            value,
            ts,
            session: session.into(),
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    /// The semilattice join: greater `(ts, session)` wins. Commutative,
    /// associative and idempotent — the total order makes any merge
    /// order pick the same maximal element.
    pub fn merged(self, other: &LwwRegister<T>) -> LwwRegister<T> {
        if (other.ts, &other.session) > (self.ts, &self.session) {
            other.clone()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greater_timestamp_wins_regardless_of_direction() {
        let a = LwwRegister::new("old", 1, "s-a");
        let b = LwwRegister::new("new", 2, "s-b");
        assert_eq!(a.clone().merged(&b).value(), &"new".to_string());
        assert_eq!(b.clone().merged(&a).value(), &"new".to_string());
    }

    #[test]
    fn merge_is_idempotent() {
        let a = LwwRegister::new("v", 7, "s");
        assert_eq!(a.clone().merged(&a), a);
    }
}
