# `derive_struct!` Macro

The `derive_struct!` macro is the core of yuuka. It lets you define complex nested struct hierarchies using a concise, JSON-like DSL. All inline types are automatically extracted into independent top-level struct/enum definitions.

## Basic Syntax

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

This generates a struct with `#[derive(Debug, Clone, Default)]` automatically applied:

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## Nested Structs

Define inline sub-structs directly inside a parent struct by specifying `FieldName { ... }` as the type:

```rust
derive_struct!(
    Root {
        info: Info {
            name: String,
            detail: Detail {
                level: u32,
                score: f64,
            },
        },
    }
);
```

This generates **three** independent structs: `Root`, `Info`, and `Detail`. Each is a normal Rust struct with all fields public within the generated module.

You can nest to arbitrary depth:

```rust
derive_struct!(
    Root {
        a: A {
            b: B {
                c: C {
                    d: D {
                        value: String,
                    },
                },
            },
        },
    }
);
```

---

## Anonymous Structs

Omit the type name to create auto-named structs:

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

The anonymous struct is automatically named `_Root_0_anonymous`. When there are multiple anonymous types, they are numbered sequentially:

```rust
derive_struct!(
    Root {
        a: {
            b: String,
        },
        c: {
            d: f64,
        },
    }
);
// Generates: _Root_0_anonymous (for a), _Root_1_anonymous (for c)
```

Anonymous structs can be deeply nested:

```rust
derive_struct!(
    Root {
        a: {
            b: String,
            c: {
                d: f64 = std::f64::consts::PI,
                e: {
                    f: bool = false,
                },
            },
            g: {
                h: i32 = -114514,
            },
        },
    }
);

let root = Root::default();
assert_eq!(root.a.c.d, std::f64::consts::PI);
assert!(!root.a.c.e.f);
assert_eq!(root.a.g.h, -114514);
```

> **Tip**: Use the [`auto!`](./auto-macro.md) macro to construct anonymous struct instances without knowing the generated names.

---

## Array (Vec) Types

Use `[Type { ... }]` syntax to define `Vec<Type>` fields:

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// Generates field: items: Vec<Item>
```

### Anonymous Array Elements

```rust
derive_struct!(
    Root {
        items: [{
            name: String,
        }],
    }
);
// Generates: items: Vec<_Root_0_anonymous>
```

### Enum Arrays

```rust
derive_struct!(
    Root {
        statuses: [enum Status {
            Active,
            Inactive,
        }],
    }
);
// Generates: statuses: Vec<Status>
```

### Anonymous Enum Arrays

```rust
derive_struct!(
    Root {
        values: [enum {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        }],
    }
);
// Generates: values: Vec<_Root_0_anonymous>
```

---

## Optional (Option) Types

Append `?` to the field name to wrap the type in `Option<T>`:

```rust
derive_struct!(
    Root {
        required: String,
        optional?: String,
    }
);
// Generates:
//   required: String,
//   optional: Option<String>,
```

### Option with Inline Struct

```rust
derive_struct!(
    Root {
        detail?: Detail {
            info: String,
        },
    }
);
// Generates: detail: Option<Detail>
```

### Option with Anonymous Struct

```rust
derive_struct!(
    Root {
        data?: {
            value: String,
        },
    }
);
// Generates: data: Option<_Root_0_anonymous>
```

### Option with Inline Enum

```rust
derive_struct!(
    Root {
        status?: enum Status {
            Active,
            Inactive,
        },
    }
);
// Generates: status: Option<Status>
```

### Option Inside Enum Variants

The `?` syntax also works inside enum struct variants:

```rust
derive_struct!(
    Root {
        action?: enum Action {
            Midori { detail?: String },
        },
    }
);
```

---

## Default Values

Assign default values with `=` after the type:

```rust
derive_struct!(
    Config {
        host: String = "localhost".to_string(),
        port: u16 = 8080,
        debug: bool = false,
    }
);

let config = Config::default();
assert_eq!(config.host, "localhost");
assert_eq!(config.port, 8080);
assert_eq!(config.debug, false);
```

### Behavior

- Fields **with** an explicit `= value` use that value in the generated `impl Default`.
- Fields **without** `= value` use `Default::default()` (e.g., `0` for numbers, `""` for String, `false` for bool).
- If **any** field has a custom default, the macro generates a manual `impl Default` block instead of `#[derive(Default)]`.

### Default Values for Arrays

```rust
derive_struct!(
    Root {
        // Empty by default
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Vec is empty by default
```

With an explicit array default:

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String = "unnamed".to_string(),
        }] = vec![Item { name: "first".to_string() }],
    }
);

let mut root = Root::default();
assert_eq!(root.items.len(), 1);
assert_eq!(root.items[0].name, "first");

// New items get the Item-level defaults
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### Default Values for Enums

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        member: enum Member {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        } = Midori,
    }
);

let root = Root::default();
assert_eq!(root.member, Member::Midori);
```

Enum array with defaults:

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        members: [enum Member {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        } = Midori] = vec![Member::Arisu],
    }
);

let mut root = Root::default();
assert_eq!(root.members[0], Member::Arisu); // From the Vec default
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // From the enum default
```

---

## Inline Enums

Define enums inline within struct fields:

```rust
derive_struct!(
    Root {
        status: enum Status {
            Active,
            Inactive,
        },
    }
);
```

### Variant Forms

Enum variants support three forms:

**Unit variants** — no associated data:

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**Struct-like variants** — named fields, which can themselves contain inline structs and enums:

```rust
a: enum Member {
    Momoi {
        skill: Skill {
            name: String,
        },
    },
    Midori { skills: Vec<String>, level: usize },
    Yuzu {
        skill: SkillYuzu {
            name: String,
        },
        level: usize,
    },
    Arisu { level: usize },
}
```

**Tuple-like variants** — positional data, supporting inline structs, multiple fields, and static types:

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### Nested Enums

Enums can be nested inside enum variants:

```rust
// Enum in struct-like variant
derive_struct!(
    Root {
        a: enum Member {
            Arisu {
                ty: enum ArisuType {
                    Arisu,
                    Key,
                },
            },
        },
    }
);
let _ = Root { a: Member::Arisu { ty: ArisuType::Key } };

// Enum in tuple-like variant
derive_struct!(
    Root {
        a: enum Member {
            Arisu(enum ArisuType {
                Arisu,
                Key,
            }),
        },
    }
);
let _ = Root { a: Member::Arisu(ArisuType::Key) };
```

### Enum Arrays in Variants

```rust
// Vec<enum> in struct-like variant
derive_struct!(
    Root {
        a: enum Member {
            Arisu {
                ty: [enum ArisuType {
                    Arisu,
                    Key,
                }],
            },
        },
    }
);

// Vec<enum> in tuple-like variant
derive_struct!(
    Root {
        a: enum Member {
            Arisu([enum ArisuType {
                Arisu,
                Key,
            }]),
        },
    }
);
```

---

## Reference Types

Use path syntax to reference externally defined types:

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct ExternalType {
    data: f64,
}

derive_struct!(
    Root {
        name: String,
        external: super::ExternalType,
    }
);
```

> **Note**: Because generated types live inside a module (`__Root`), you typically need `super::` to reference types from the outer scope. The exact path depends on where the external type is defined relative to the macro invocation.

Field names are flexible — both snake_case and PascalCase names work:

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
