<img src="splash.png" alt="yuuka" />

![GitHub License](https://img.shields.io/github/license/celestia-island/yuuka)
[![Crates.io Version](https://img.shields.io/crates/v/yuuka)](https://docs.rs/yuuka)

## Introduction

This is a helper library to generate complex and nested structures by a simple macro. It is based on the `serde` library that is used to serialize and deserialize data in Rust.

The name `yuuka` comes from the character [Yuuka](https://bluearchive.wiki/wiki/Yuuka) in the game [Blue Archive](https://bluearchive.jp/).

## Quick Start

```rust
use yuuka::derive_config;

derive_config!(GameDevelopment {
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
});

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
- [ ] Anonymous struct support that can use `auto!` macro to confirm the auto-generated field name
- [ ] Default value support that can use `=` to assign the default value
- [ ] `pub` and `pub(crate)` identifier support
- [ ] `strum` integration, including control the case of the field name (e.g. `snake_case`, `camelCase`, `PascalCase`)
- [ ] Write a homepage for this library
- [x] Upload to `crates.io`
