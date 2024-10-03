//! # Yuuka
//!
//! ## Introduction
//!
//! This is a helper library to generate complex and nested structures by a simple macro. It is based on the `serde` library that is used to serialize and deserialize data in Rust.
//!
//! The name `yuuka` comes from the character [Yuuka](https://bluearchive.wiki/wiki/Yuuka) in the game [Blue Archive](https://bluearchive.jp/).
//!
//! ## Quick Start
//!
//! ```rust
//! use yuuka::derive_config;
//!
//! derive_config!(GameDevelopment {
//!     description: String,
//!     members: Members {
//!         script_writer: String,
//!         illustrator: String,
//!         programmer: String,
//!         tester: Vec<String>,
//!     },
//!     projects: [Project {
//!         project_name: String,
//!         engine: String,
//!     }],
//! });
//!
//! # fn main() {
//! let config = GameDevelopment {
//!     description: "A game development team".to_string(),
//!     members: Members {
//!         script_writer: "Momoi".to_string(),
//!         illustrator: "Midori".to_string(),
//!         programmer: "Yuzu".to_string(),
//!         tester: vec!["Arisu".to_string(), "Key".to_string()],
//!     },
//!     projects: vec![
//!         Project {
//!             project_name: "777 Game Launcher".to_string(),
//!             engine: "Tauri".to_string(),
//!         },
//!         Project {
//!             project_name: "Blue Archive".to_string(),
//!             engine: "Unity".to_string(),
//!         },
//!     ]
//! };
//!
//! # assert_eq!(config.members.script_writer, "Momoi");
//! # assert_eq!(config.projects.get(1).unwrap().project_name, "Blue Archive");
//!
//! # let serialized = serde_json::to_string(&config).unwrap();
//! # println!("{}", serialized);
//! # }
//! ```
pub use _macros::{auto, derive_struct};
