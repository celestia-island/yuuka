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
    projects: ProjectMap {
        rust: RustProjectMap {
            game_launcher: String,
            forum_app: String,
        },
        typescript: TypeScriptProjectMap {
            website: String,
            backend_entry: String,
        },
        cpp: CppProjectMap {
            rpg_maker: String,
        },
    },
});

let config = GameDevelopment {
    description: "A game development team".to_string(),
    members: Members {
        script_writer: "Momoi".to_string(),
        illustrator: "Midori".to_string(),
        programmer: "Yuzu".to_string(),
        tester: vec!["Arisu".to_string(), "Key".to_string()],
    },
    projects: ProjectMap {
        rust: RustProjectMap {
            game_launcher: "777 Game Launcher".to_string(),
            forum_app: "777 Forum App".to_string(),
        },
        typescript: TypeScriptProjectMap {
            website: "777 Website".to_string(),
            backend_entry: "777 Control Center".to_string(),
        },
        cpp: CppProjectMap {
            rpg_maker: "RPG Maker".to_string(),
        },
    },
};

let serialized = serde_json::to_string(&config).unwrap();
println!("{}", serialized);
```

## TODO

- [ ] Array type support
- [ ] Enum type support
- [ ] `strum` integration, including control the case of the field name (e.g. `snake_case`, `camelCase`, `PascalCase`)
- [ ] Write a homepage for this library
- [ ] Upload to `crates.io`
