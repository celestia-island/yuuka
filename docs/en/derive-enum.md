# `derive_enum!` Macro

The `derive_enum!` macro defines standalone enum types with the same DSL syntax style as `derive_struct!`. It supports all three variant forms, nested types, and default values.

## Basic Syntax

```rust
use yuuka::derive_enum;

derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);
```

This generates an enum with `#[derive(Debug, Clone)]` automatically applied.

---

## Variant Forms

### Unit Variants

Simple variants with no associated data:

```rust
derive_enum!(
    enum Direction {
        North,
        South,
        East,
        West,
    }
);
```

### Struct-like Variants

Variants with named fields. Fields can use inline struct definitions:

```rust
derive_enum!(
    enum Action {
        Move { x: f64, y: f64 },
        Attack {
            target: Target {
                id: u64,
                name: String,
            },
            damage: u32,
        },
    }
);
// Generates an independent `Target` struct alongside the `Action` enum.
```

### Tuple-like Variants

Variants with positional data. Can contain inline structs, enums, and static types:

```rust
derive_enum!(
    enum Message {
        Text(String),
        Data(Payload { content: Vec<u8>, size: usize }),
        Multi(String, i32, bool),
    }
);
```

### Mixed Variants

All three forms can coexist in a single enum:

```rust
derive_enum!(
    #[derive(PartialEq, Serialize, Deserialize)]
    enum Router {
        Home,
        User { id: u64, name: String },
        Error(String),
    }
);
```

---

## Nested Enums

Enum variants can contain other inline enums:

### In Tuple Variants

```rust
derive_enum!(
    enum Group {
        Millennium(enum Millennium {
            GameDevelopment(enum GameDevelopment {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            }),
            CAndC,
            Veritas,
        }),
    }
);

let _ = Group::Millennium(Millennium::GameDevelopment(GameDevelopment::Yuzu));
```

### Anonymous Nested Enums

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        A,
        B(i32),
        C { a: String, b: i32 },
        D(enum {
            E,
            F(i32),
            G { a: String, b: i32 },
        }),
    }
);
```

Anonymous enums inside variants are named like `_Root_0_anonymous`. You can reference them via the module:

```rust
let _ = Root::D(__Root::_Root_0_anonymous::E);
```

> **Tip**: Use [`auto!`](./auto-macro.md) to avoid dealing with generated anonymous names. `auto!(Root::D::E)` resolves the path automatically.

### Deeply Nested Anonymous Enums

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum A {
        B(enum {
            C(enum {
                D(enum {
                    E(enum {
                        F,
                        G(String),
                    }),
                }),
            }),
        }),
    }
);

// Manual construction:
let _ = A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))));

// With auto!:
use yuuka::auto;
let _ = auto!(A::B::C::D::E::F);
let _ = auto!(A::B::C::D::E::G("hello".to_string()));
```

---

## Default Values

Specify a default variant with `= VariantName` after the closing brace:

```rust
derive_enum!(
    enum Theme {
        Light,
        Dark,
        System,
    } = Dark
);

let theme = Theme::default();
// theme == Theme::Dark
```

### Default for Tuple Variants

```rust
derive_enum!(
    enum Value {
        Int(i32),
        Text(String),
    } = Int(0)
);
```

### Default for Nested Anonymous Enums

```rust
derive_enum!(
    enum Group {
        Millennium(enum {
            GameDevelopment(enum GameDevelopment {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            } = Yuzu),
            CAndC,
            Veritas,
        } = GameDevelopment(Default::default())),
    } = Millennium(Default::default())
);

// Group::default() == Group::Millennium(GameDevelopment(Yuzu))
```

> **Note**: When no default value is specified, the generated `impl Default` uses `unimplemented!()`, which will panic at runtime if called. Always specify a default if you plan to use `Default::default()`.

---

## Extra Derive and Attribute Macros

Just like `derive_struct!`, you can pass `#[derive(...)]` and attribute macros:

```rust
derive_enum!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Member {
        SaibaMomoi,
        SaibaMidori,
        HanaokaYuzu,
        TendouAris,
    } = SaibaMidori
);

let json = serde_json::to_string(&Member::default()).unwrap();
assert_eq!(json, r#""saiba_midori""#);
```

See [Attributes & Visibility](./attributes.md) for full details on attribute macros, recursive propagation, and variant-level attributes.
