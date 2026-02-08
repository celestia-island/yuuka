# Macro `derive_enum!`

La macro `derive_enum!` define tipos enum independientes con el mismo estilo de sintaxis DSL que `derive_struct!`. Soporta las tres formas de variantes, tipos anidados y valores por defecto.

## Sintaxis básica

```rust
use yuuka::derive_enum;

derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);
```

Esto genera un enum con `#[derive(Debug, Clone)]` aplicado automáticamente.

---

## Formas de variantes

### Variantes unit

Variantes simples sin datos asociados:

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

### Variantes tipo struct

Variantes con campos con nombre. Los campos pueden usar definiciones de struct en línea:

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
// Genera un struct `Target` independiente junto al enum `Action`.
```

### Variantes tipo tupla

Variantes con datos posicionales. Pueden contener structs en línea, enums y tipos estáticos:

```rust
derive_enum!(
    enum Message {
        Text(String),
        Data(Payload { content: Vec<u8>, size: usize }),
        Multi(String, i32, bool),
    }
);
```

### Variantes mixtas

Las tres formas pueden coexistir en un mismo enum:

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

## Enumeraciones anidadas

Las variantes de enum pueden contener otros enums en línea:

### En variantes tipo tupla

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

### Enums anidados anónimos

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

Los enums anónimos dentro de variantes se nombran como `_Root_0_anonymous`. Puedes referenciarlos a través del módulo:

```rust
let _ = Root::D(__Root::_Root_0_anonymous::E);
```

> **Consejo**: Usa [`auto!`](./auto-macro.md) para evitar lidiar con nombres anónimos generados. `auto!(Root::D::E)` resuelve la ruta automáticamente.

### Enums anónimos profundamente anidados

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

// Construcción manual:
let _ = A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))));

// Con auto!:
use yuuka::auto;
let _ = auto!(A::B::C::D::E::F);
let _ = auto!(A::B::C::D::E::G("hello".to_string()));
```

---

## Valores por defecto

Especifica una variante por defecto con `= NombreVariante` después de la llave de cierre:

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

### Valor por defecto para variantes tipo tupla

```rust
derive_enum!(
    enum Value {
        Int(i32),
        Text(String),
    } = Int(0)
);
```

### Valor por defecto para enums anónimos anidados

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

> **Nota**: Cuando no se especifica un valor por defecto, el `impl Default` generado usa `unimplemented!()`, lo cual entrará en pánico en tiempo de ejecución si se llama. Siempre especifica un valor por defecto si planeas usar `Default::default()`.

---

## Derives y atributos extra

Al igual que `derive_struct!`, puedes pasar `#[derive(...)]` y macros de atributos:

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

Consulta [Atributos y visibilidad](./attributes.md) para detalles completos sobre macros de atributos, propagación recursiva y atributos a nivel de variante.

---

## Interacción con derive_struct

Los enums se pueden definir en línea dentro de `derive_struct!`. `derive_enum!` es útil cuando necesitas definir un enum independiente fuera de un contexto de struct. Ambas macros comparten la misma sintaxis DSL para variantes de enum — todo lo que funciona dentro de `derive_struct!` para enums en línea también funciona en `derive_enum!` como definición independiente.

Consulta [derive_struct! — Enumeraciones en línea](./derive-struct.md#enumeraciones-en-línea) para ver cómo se usan los enums dentro de definiciones de struct.
