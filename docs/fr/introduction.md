# Yuuka - Introduction

**Yuuka** est une bibliothèque Rust de macros procédurales qui vous permet de définir des hiérarchies complexes et profondément imbriquées de structures et d'énumérations en utilisant une syntaxe DSL concise, semblable à JSON. Elle est construite sur `serde` pour une sérialisation et une désérialisation transparentes.

## Installation

Ajoutez les lignes suivantes à votre `Cargo.toml` :

```toml
[dependencies]
yuuka = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> `serde` et `serde_json` sont optionnels mais couramment utilisés avec yuuka pour le support de la sérialisation.

## Macros principales

Yuuka exporte trois macros procédurales :

| Macro | Objectif |
| --- | --- |
| [`derive_struct!`](./derive-struct.md) | Définir des hiérarchies de structures imbriquées avec un DSL semblable à JSON |
| [`derive_enum!`](./derive-enum.md) | Définir des types d'énumérations avec différentes formes de variantes |
| [`auto!`](./auto-macro.md) | Construire des instances de types générés par les macros ci-dessus avec une syntaxe simplifiée |

Voir aussi :

- [Attributs et visibilité](./attributes.md) — Macros derive supplémentaires, propagation des attributs, contrôle de la visibilité et utilisation inter-crates
- [Exemples](./examples.md) — Exemples concrets et structure du code généré

## Démarrage rapide

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    GameConfig {
        title: String,
        window: Window {
            width: u32,
            height: u32,
            fullscreen: bool,
        },
        plugins: [Plugin {
            name: String,
            enabled: bool,
        }],
    }
);

let config = auto!(GameConfig {
    title: "My Game".to_string(),
    window: {
        width: 1920,
        height: 1080,
        fullscreen: true,
    },
    plugins: vec![
        Plugin {
            name: "Audio".to_string(),
            enabled: true,
        },
    ],
});
```

Cet unique appel à `derive_struct!` génère automatiquement trois structures indépendantes — `GameConfig`, `Window` et `Plugin` — toutes avec `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`. La macro `auto!` permet ensuite de construire des instances en utilisant des blocs `{ }` pour les sous-structures anonymes/en ligne sans avoir à connaître leurs noms générés.

## Index de la documentation

| Document | Description |
| --- | --- |
| [derive_struct!](./derive-struct.md) | Macro de définition de structures — structures imbriquées, structures anonymes, types Vec/Option, valeurs par défaut, énumérations en ligne, types par référence |
| [derive_enum!](./derive-enum.md) | Macro de définition d'énumérations — variantes unitaires/struct/tuple, énumérations imbriquées, valeurs par défaut |
| [auto!](./auto-macro.md) | Macro de construction d'instances — syntaxe simplifiée pour les types anonymes, chemins d'énumérations, expressions de propagation |
| [Attributs et visibilité](./attributes.md) | Macros derive, propagation des attributs, `#[macros_recursive]`, attributs au niveau des champs, visibilité, `#[macro_export]`, utilisation inter-crates |
| [Exemples](./examples.md) | Exemples concrets, explication de la structure du code généré |
