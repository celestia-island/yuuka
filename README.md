<img src="splash.png" alt="yuuka" />

![Crates.io License](https://img.shields.io/crates/l/yuuka)
[![Crates.io Version](https://img.shields.io/crates/v/yuuka)](https://docs.rs/yuuka)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/celestia-island/yuuka/test.yml)

## Introduction

This is a helper library to generate complex and nested structures by a simple macro. It is based on the `serde` library that is used to serialize and deserialize data in Rust.

The name `yuuka` comes from the character [Yuuka](https://bluearchive.wiki/wiki/Yuuka) in the game [Blue Archive](https://bluearchive.jp/).

## Quick Start

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

let config = GameDevelopment {
    description: "A game development team".to_string(),
    members: Members {
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
};
```

## TODO

- [x] Array type support
- [x] Enum type support
- [x] Anonymous struct support that can use `auto!` macro to confirm the auto-generated field name
- [x] Default value support that can use `=` to assign the default value
- [x] `pub` and `pub(crate)` identifier support
- [x] Support custom derive macro.
- [ ] Write a homepage for this library
- [x] Upload to `crates.io`
