<p align="center"><img src="https://raw.githubusercontent.com/celestia-island/yuuka/master/docs/logo.webp" alt="Yuuka" width="240" /></p>

<h1 align="center">Yuuka</h1>

<p align="center"><strong>JSON merge/patch foundation library &amp; nested structure construction macros</strong></p>

[![License: SySL](https://img.shields.io/badge/license-SySL%201.0-blue)](./LICENSE) [![Crates.io Version](https://img.shields.io/crates/v/yuuka)](https://docs.rs/yuuka)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/celestia-island/yuuka/test.yml)

## Introduction

`yuuka` is a small foundation library for Rust with two complementary halves:

1. **A JSON state-patch runtime** — [RFC 7396](https://www.rfc-editor.org/rfc/rfc7396) merge patch, dotted-path patch operations, before/after diffing and viewport snapshots over `serde_json::Value`. This is the wire-level core of the Celestia state-tree synchronization protocol, extracted from `plana` `packages/sync` so that servers and clients share one implementation.
2. **Nested structure construction macros** — `derive_struct!`, `derive_enum!` and `auto!`, a concise DSL on top of `serde` for declaring and building deeply nested configuration types.

The name `yuuka` comes from the character [Yuuka](https://bluearchive.wiki/wiki/Yuuka) in the game [Blue Archive](https://bluearchive.jp/).

For more information, visit the official documentation at [yuuka.docs.celestia.world](https://yuuka.docs.celestia.world)

> Current release: **1.0.0-rc.1** — the patch wire format is locked; the API is settling ahead of the 1.0.0 final.

## State Patch Runtime

Five modules implement the runtime half of the state-tree synchronization protocol. The model is "single server-side writer": the server diffs successive versions of a state tree into patch ops and broadcasts them to clients subscribed to the matching viewport; clients only ever apply. There is no concurrent-writer conflict handling — lost patches are healed by periodic full-snapshot fallbacks, so the protocol self-recovers.

### `patch` — patch operations (`PatchOp`)

A single operation on a dotted path, applied with `apply` / `apply_all`:

```rust
use serde_json::json;
use yuuka::patch::{apply, PatchOp};

let mut state = json!({"agents": {"hubris": {"status": "idle", "model": "gpt"}}});

// `set` deep-merges: untouched sibling keys survive.
apply(&mut state, &PatchOp::set("agents.hubris.status", json!("busy")));
assert_eq!(
    state,
    json!({"agents": {"hubris": {"status": "busy", "model": "gpt"}}})
);

// `replace` swaps the subtree wholesale: no residue of the old value.
apply(&mut state, &PatchOp::replace("agents.hubris", json!({"work_status": {"Completed": {}}})));
assert_eq!(
    state,
    json!({"agents": {"hubris": {"work_status": {"Completed": {}}}}})
);
```

### `merge` — RFC 7396 core (`merge_patch`)

The `MergePatch(Target, Patch)` recursion behind every `set` operation: objects merge key by key, a `null` in the patch deletes the key, anything else replaces outright.

```rust
use serde_json::json;
use yuuka::merge::merge_patch;

let target = json!({"a": {"x": 1, "y": 2}, "b": 3});
let patch = json!({"a": {"y": 20, "z": 3}, "c": 4});

assert_eq!(
    merge_patch(target, patch),
    json!({"a": {"x": 1, "y": 20, "z": 3}, "b": 3, "c": 4})
);
```

### `diff` — before/after diffing (`diff`)

Turns two versions of a state tree into the op list that reconstructs `after` from `before`. The roundtrip `apply_all(before, diff("", &before, &after)) == after` (empty prefix = whole tree) is pinned by property-based tests.

```rust
use serde_json::json;
use yuuka::diff::diff;
use yuuka::patch::apply_all;

// Two versions of the `state.agents` subtree:
let before = json!({"hubris": {"status": "idle"}});
let after = json!({"hubris": {"status": "busy", "model": "glm"}, "seia": {"status": "idle"}});

// Diff with the prefix naming where the subtree lives in the global root.
let ops = diff("state.agents", &before, &after);
let mut root = json!({"state": {"agents": before}});
apply_all(&mut root, &ops);
assert_eq!(root, json!({"state": {"agents": after}}));
```

### `viewport` — subscription snapshots (`snapshot`, `path_in_viewport`)

Clients subscribe to path prefixes; the server crops the global tree down to the visible subtrees and pushes them as snapshots.

```rust
use serde_json::json;
use yuuka::viewport::{path_in_viewport, snapshot};

let root = json!({
    "state": {
        "agents": {"hubris": {"status": "idle"}},
        "devices": {"node1": {"online": true}}
    }
});
let viewport = vec!["state.agents".to_string()];

assert!(path_in_viewport("state.agents.hubris.status", &viewport));
assert!(!path_in_viewport("state.devices.node1", &viewport));

let snap = snapshot(&root, &viewport);
assert_eq!(snap, json!({"state": {"agents": {"hubris": {"status": "idle"}}}}));
```

### `path` — dotted-path helpers

`split` / `join` / `segments` / `display` utilities shared by the modules above. Paths are dot-separated (`state.agents.hubris`); the empty string denotes the root.

```rust
use yuuka::path::{display, split};

let segs = split("state.agents.hubris");
assert_eq!(segs, vec!["state", "agents", "hubris"]);
assert_eq!(display(&segs).to_string(), "state.agents");
```

### Semantics and wire compatibility

The merge semantics are an RFC 7396 variant with one deliberate extension:

- `set` — deep merge: object keys merge one by one (the new value wins on collision), non-objects replace outright;
- `replace` — wholesale replacement, no deep merge. For enums / tagged unions that must be swapped atomically (e.g. `work_status` going from `{Running:{}}` to `{Completed:{}}`) — the core difference from plain RFC 7396;
- `del` — delete the key at the path (deleting the root resets it to an empty object rather than `null`).

The serde representation of `PatchOp` is **wire-locked** to the historical `plana` `Sync.StatePatch` notification (`op` / `path` / optional `value`, lowercase op tags). The golden-JSON tests in `tests/patch_compat.rs` pin it byte for byte — do not change the serialization shape.

## Nested Structure Macros

The macro half generates complex, nested structures from a concise syntax, on top of the `serde` library that is used to serialize and deserialize data in Rust:

- `derive_struct!` generates nested structs from a DSL-like syntax;
- `derive_enum!` generates enums (and associated structs) from the same syntax;
- `auto!` constructs values of the generated types with value-only syntax.

```rust
use serde::{Serialize, Deserialize};
use yuuka::derive_struct;

derive_struct!(
    #[derive(Serialize, Deserialize)]
    GameDevelopment {
        description: String,
        members: Members {
            script_writer: String,
            illustrator: String,
            programmer: String,
            tester: Vec<String>,
        },
        projects: [Project {
            project_name: String,
            engine: String,
        }],
    }
);

let config = auto!(GameDevelopment {
    description: "A game development team".to_string(),
    members: {
        script_writer: "Momoi".to_string(),
        illustrator: "Midori".to_string(),
        programmer: "Yuzu".to_string(),
        tester: vec!["Arisu".to_string(), "Key".to_string()],
    },
    projects: vec![
        Project {
            project_name: "777 Game Launcher".to_string(),
            engine: "Tauri".to_string(),
        },
        Project {
            project_name: "Blue Archive".to_string(),
            engine: "Unity".to_string(),
        },
    ]
});
```

More information can be found in the official documentation at [yuuka.docs.celestia.world](https://yuuka.docs.celestia.world).

## Repository Layout

Since v0.7.0 this repository is a Cargo workspace with two crates:

| Crate | Path | Description |
|-------|------|-------------|
| `yuuka` | repository root | Public facade that re-exports every macro and hosts the merge/patch/diff/path/viewport runtime modules, keeping the `yuuka::derive_struct!` and friends paths stable for downstream users |
| `yuuka-macros` | `macros/` | The `proc-macro` implementation crate providing `derive_struct!`, `derive_enum!` and `auto!` |

Both crates share a single version through `workspace.package.version`. The integration tests under `tests/` invoke the macros through the facade crate, so downstream usage is unaffected by the split.

## AI-Generated Code Disclosure

NOTICE: This software includes code generated by artificial intelligence. See the LICENSE file for the Synthetic Source License terms, including model disclosure requirements.

## License

Licensed under the [Synthetic Source License (SySL), Version 1.0](./LICENSE); see also the [SySL website](https://sysl.celestia.world).
