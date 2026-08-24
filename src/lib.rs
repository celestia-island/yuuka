//! Nested structure derivation macros for Rust, plus the runtime half of
//! the state-tree synchronization protocol.
//!
//! # Macros
//!
//! This crate is the public facade of the `yuuka` macro family: the
//! procedural macros themselves live in [`yuuka_macros`] and are re-exported
//! here so that the historical `yuuka::derive_struct!`, `yuuka::derive_enum!`
//! and `yuuka::auto!` paths keep working unchanged.
//!
//! - [`derive_struct!`] generates nested structs from a concise DSL-like syntax.
//! - [`derive_enum!`] generates enums (and associated structs) from the same syntax.
//! - [`auto!`] constructs values of the generated types with value-only syntax.
//!
//! # State patch runtime
//!
//! Alongside the macros, yuuka ships the runtime modules of the state-tree
//! synchronization protocol (migrated from `plana` `packages/sync`):
//!
//! - [`patch`]: [`PatchOp`](patch::PatchOp) — a single `set` / `replace` /
//!   `del` operation on a dotted path, with a serde wire format that stays
//!   byte-compatible with the `Sync.StatePatch` notification.
//! - [`merge`]: [`merge_patch`](merge::merge_patch) — the RFC 7396 JSON
//!   Merge Patch core behind `set` operations.
//! - [`diff`]: [`diff`](diff::diff) — turn a before/after state pair into
//!   the op list that reconstructs `after` from `before`.
//! - [`viewport`]: [`path_in_viewport`](viewport::path_in_viewport) /
//!   [`snapshot`](viewport::snapshot) — crop the state tree down to the
//!   path prefixes a client subscribed to (the `Sync.StateSnapshot`
//!   payload).
//! - [`path`]: dotted-path helpers (`split` / `join` / `segments` /
//!   `display`) shared by the modules above.

pub use yuuka_macros::*;

pub mod diff;
pub mod merge;
pub mod patch;
pub mod path;
pub mod viewport;
