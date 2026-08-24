//! Minimal dotted-path utilities for the patch/diff modules.
//!
//! Paths in the state-tree protocol are dot-separated
//! (`state.agents.hubris`); the empty string denotes the root. There is no
//! escaping — keys of the state tree are identifiers (UUIDs and the like)
//! that never contain dots.
//!
//! Only the facilities needed by [`crate::patch`] and [`crate::diff`] live
//! here; the viewport-path helpers stay with the snapshot module of a later
//! migration phase.

use std::fmt;

/// Splits a dotted path into segments: `state.agents.hubris` →
/// `["state","agents","hubris"]`. The empty path yields an empty vector
/// (the root).
///
/// Migration note: this is `plana` `sync::patch::split_path`.
pub fn split(path: &str) -> Vec<&str> {
    segments(path).collect()
}

/// Iterates over the segments of a dotted path without allocating. The
/// empty path yields no segments; trailing dots yield an empty final
/// segment (exactly like `str::split('.')`).
pub fn segments(path: &str) -> Segments<'_> {
    Segments {
        rest: path,
        done: path.is_empty(),
    }
}

/// Joins a path prefix and one more key: `("state.agents", "hubris")` →
/// `"state.agents.hubris"`. An empty prefix denotes the root, so the key
/// is returned alone.
pub fn join(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

/// Renders a segment slice back into dotted form; see [`display`].
pub struct DisplayPath<'a> {
    parts: &'a [&'a str],
}

/// Wraps a segment slice so it formats as a dotted path:
/// `display(&["state","agents"])` renders as `state.agents`. The empty
/// slice renders as the empty string (the root).
pub fn display<'a>(parts: &'a [&'a str]) -> DisplayPath<'a> {
    DisplayPath { parts }
}

impl fmt::Display for DisplayPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, part) in self.parts.iter().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            f.write_str(part)?;
        }
        Ok(())
    }
}

/// Borrowing iterator over the segments of a dotted path; see [`segments`].
pub struct Segments<'a> {
    rest: &'a str,
    done: bool,
}

impl<'a> Iterator for Segments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.rest.split_once('.') {
            Some((head, tail)) => {
                self.rest = tail;
                Some(head)
            }
            None => {
                self.done = true;
                Some(self.rest)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_matches_str_split() {
        assert_eq!(split(""), Vec::<&str>::new());
        assert_eq!(split("state"), vec!["state"]);
        assert_eq!(
            split("state.agents.hubris"),
            vec!["state", "agents", "hubris"]
        );
        // Degenerate shapes stay faithful to `str::split('.')`.
        assert_eq!(split("."), vec!["", ""]);
        assert_eq!(split("a."), vec!["a", ""]);
        assert_eq!(split(".a"), vec!["", "a"]);
        assert_eq!(split("a..b"), vec!["a", "", "b"]);
    }

    #[test]
    fn segments_iterator_equals_split() {
        for path in ["", ".", "state", "state.agents.hubris", "a.", ".a", "a..b"] {
            assert_eq!(segments(path).collect::<Vec<_>>(), split(path));
        }
    }

    #[test]
    fn join_handles_root_prefix() {
        assert_eq!(join("", "state"), "state");
        assert_eq!(join("state", "agents"), "state.agents");
        assert_eq!(join("state.agents", "hubris"), "state.agents.hubris");
    }

    #[test]
    fn display_roundtrips_split() {
        assert_eq!(
            display(&split("state.agents.hubris")).to_string(),
            "state.agents.hubris"
        );
        assert_eq!(display(&[]).to_string(), "");
        assert_eq!(display(&["a"]).to_string(), "a");
        // Trailing/empty segments survive the roundtrip too.
        assert_eq!(display(&split("a.")).to_string(), "a.");
        assert_eq!(display(&split("a..b")).to_string(), "a..b");
    }
}
