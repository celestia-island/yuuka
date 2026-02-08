# Yuuka - Introducción

**Yuuka** es una biblioteca de macros procedurales de Rust que permite definir jerarquías complejas y profundamente anidadas de structs y enums usando una sintaxis DSL concisa similar a JSON. Está construida sobre `serde` para una serialización y deserialización sin complicaciones.

## Instalación

Agrega lo siguiente a tu `Cargo.toml`:

```toml
[dependencies]
yuuka = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> `serde` y `serde_json` son opcionales, pero se usan comúnmente junto con yuuka para soporte de serialización.

## Macros principales

Yuuka exporta tres macros procedurales:

| Macro | Propósito |
| --- | --- |
| [`derive_struct!`](./derive-struct.md) | Definir jerarquías de structs anidados con un DSL similar a JSON |
| [`derive_enum!`](./derive-enum.md) | Definir tipos enum con diversas formas de variantes |
| [`auto!`](./auto-macro.md) | Construir instancias de tipos generados por las macros anteriores con sintaxis simplificada |

Consulta también:

- [Atributos y visibilidad](./attributes.md) — Macros derive extra, propagación de atributos, control de visibilidad y uso entre crates
- [Ejemplos](./examples.md) — Ejemplos del mundo real y estructura del código generado

## Inicio rápido

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

Esta única llamada a `derive_struct!` genera automáticamente tres structs independientes — `GameConfig`, `Window` y `Plugin` — todos con `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`. La macro `auto!` luego permite construir instancias usando bloques `{ }` para sub-structs anónimos/en línea sin necesidad de conocer sus nombres generados.

## Índice de documentación

| Documento | Descripción |
| --- | --- |
| [derive_struct!](./derive-struct.md) | Macro de definición de structs — structs anidados, structs anónimos, tipos Vec/Option, valores por defecto, enums en línea, tipos por referencia |
| [derive_enum!](./derive-enum.md) | Macro de definición de enums — variantes unit/struct/tuple, enums anidados, valores por defecto |
| [auto!](./auto-macro.md) | Macro de construcción de instancias — sintaxis simplificada para tipos anónimos, rutas de enum, expresiones de propagación |
| [Atributos y visibilidad](./attributes.md) | Macros derive, propagación de atributos, `#[macros_recursive]`, atributos a nivel de campo, visibilidad, `#[macro_export]`, uso entre crates |
| [Ejemplos](./examples.md) | Ejemplos del mundo real, explicación de la estructura del código generado |
