# Yuuka

![yuuka](splash.png)

![Crates.io License](https://img.shields.io/crates/l/yuuka)
[![Crates.io Version](https://img.shields.io/crates/v/yuuka)](https://docs.rs/yuuka)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/celestia-island/yuuka/test.yml)

## Introduction

This is a helper library to generate complex and nested structures by a simple macro. It is based on the `serde` library that is used to serialize and deserialize data in Rust.

The name `yuuka` comes from the character [Yuuka](https://bluearchive.wiki/wiki/Yuuka) in the game [Blue Archive](https://bluearchive.jp/).

> Still in development, the API may change in the future.

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

More information can be found in the documentation ([English](https://github.com/celestia-island/yuuka/tree/next/docs/en/introduction.md) | [简体中文](https://github.com/celestia-island/yuuka/tree/next/docs/zh-hans/introduction.md) | [繁體中文](https://github.com/celestia-island/yuuka/tree/next/docs/zh-hant/introduction.md)).
