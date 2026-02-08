# Macro `derive_struct!`

La macro `derive_struct!` es el núcleo de yuuka. Permite definir jerarquías complejas de structs anidados usando un DSL conciso similar a JSON. Todos los tipos en línea se extraen automáticamente en definiciones independientes de struct/enum a nivel superior.

## Sintaxis básica

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

Esto genera un struct con `#[derive(Debug, Clone, Default)]` aplicado automáticamente:

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## Estructuras anidadas

Define sub-structs en línea directamente dentro de un struct padre especificando `NombreCampo { ... }` como el tipo:

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

Esto genera **tres** structs independientes: `Root`, `Info` y `Detail`. Cada uno es un struct Rust normal con todos los campos públicos dentro del módulo generado.

Puedes anidar a profundidad arbitraria:

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

## Estructuras anónimas

Omite el nombre del tipo para crear structs con nombre automático:

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

El struct anónimo se nombra automáticamente `_Root_0_anonymous`. Cuando hay múltiples tipos anónimos, se numeran secuencialmente:

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
// Genera: _Root_0_anonymous (para a), _Root_1_anonymous (para c)
```

Los structs anónimos pueden estar profundamente anidados:

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

> **Consejo**: Usa la macro [`auto!`](./auto-macro.md) para construir instancias de structs anónimos sin necesidad de conocer los nombres generados.

---

## Tipos array (Vec)

Usa la sintaxis `[Tipo { ... }]` para definir campos `Vec<Tipo>`:

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// Genera el campo: items: Vec<Item>
```

### Elementos array anónimos

```rust
derive_struct!(
    Root {
        items: [{
            name: String,
        }],
    }
);
// Genera: items: Vec<_Root_0_anonymous>
```

### Arrays de enums

```rust
derive_struct!(
    Root {
        statuses: [enum Status {
            Active,
            Inactive,
        }],
    }
);
// Genera: statuses: Vec<Status>
```

### Arrays de enums anónimos

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
// Genera: values: Vec<_Root_0_anonymous>
```

---

## Tipos opcionales (Option)

Agrega `?` al nombre del campo para envolver el tipo en `Option<T>`:

```rust
derive_struct!(
    Root {
        required: String,
        optional?: String,
    }
);
// Genera:
//   required: String,
//   optional: Option<String>,
```

### Option con struct en línea

```rust
derive_struct!(
    Root {
        detail?: Detail {
            info: String,
        },
    }
);
// Genera: detail: Option<Detail>
```

### Option con struct anónimo

```rust
derive_struct!(
    Root {
        data?: {
            value: String,
        },
    }
);
// Genera: data: Option<_Root_0_anonymous>
```

### Option con enum en línea

```rust
derive_struct!(
    Root {
        status?: enum Status {
            Active,
            Inactive,
        },
    }
);
// Genera: status: Option<Status>
```

### Option dentro de variantes de enum

La sintaxis `?` también funciona dentro de variantes struct de enums:

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

## Valores por defecto

Asigna valores por defecto con `=` después del tipo:

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

### Comportamiento

- Los campos **con** un `= valor` explícito usan ese valor en el `impl Default` generado.
- Los campos **sin** `= valor` usan `Default::default()` (por ejemplo, `0` para números, `""` para String, `false` para bool).
- Si **algún** campo tiene un valor por defecto personalizado, la macro genera un bloque `impl Default` manual en lugar de `#[derive(Default)]`.

### Valores por defecto para arrays

```rust
derive_struct!(
    Root {
        // Vacío por defecto
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Vec está vacío por defecto
```

Con un valor por defecto explícito para el array:

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

// Los nuevos elementos obtienen los valores por defecto de Item
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### Valores por defecto para enums

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

Array de enum con valores por defecto:

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
assert_eq!(root.members[0], Member::Arisu); // Del valor por defecto del Vec
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // Del valor por defecto del enum
```

---

## Enumeraciones en línea

Define enums en línea dentro de campos de struct:

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

### Formas de variantes

Las variantes de enum soportan tres formas:

**Variantes unit** — sin datos asociados:

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**Variantes tipo struct** — campos con nombre, que a su vez pueden contener structs y enums en línea:

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

**Variantes tipo tupla** — datos posicionales, soportando structs en línea, múltiples campos y tipos estáticos:

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### Enums anidados en variantes

Los enums pueden estar anidados dentro de variantes de enum:

```rust
// Enum en variante tipo struct
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

// Enum en variante tipo tupla
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

### Arrays de enums en variantes

```rust
// Vec<enum> en variante tipo struct
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

// Vec<enum> en variante tipo tupla
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

## Tipos por referencia

Usa la sintaxis de ruta para referenciar tipos definidos externamente:

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

> **Nota**: Dado que los tipos generados viven dentro de un módulo (`__Root`), normalmente necesitas `super::` para referenciar tipos del ámbito exterior. La ruta exacta depende de dónde esté definido el tipo externo en relación con la invocación de la macro.

Los nombres de campo son flexibles — tanto los nombres en snake_case como en PascalCase funcionan:

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
