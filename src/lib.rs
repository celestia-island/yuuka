//! Nested structure derivation macros for Rust.
//!
//! This crate is the public facade of the `yuuka` macro family: the
//! procedural macros themselves live in [`yuuka_macros`] and are re-exported
//! here so that the historical `yuuka::derive_struct!`, `yuuka::derive_enum!`
//! and `yuuka::auto!` paths keep working unchanged.
//!
//! - [`derive_struct!`] generates nested structs from a concise DSL-like syntax.
//! - [`derive_enum!`] generates enums (and associated structs) from the same syntax.
//! - [`auto!`] constructs values of the generated types with value-only syntax.

pub use yuuka_macros::*;
