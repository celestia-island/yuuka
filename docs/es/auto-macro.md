# Macro `auto!`

La macro `auto!` simplifica la construcción de instancias de tipos generados por `derive_struct!` y `derive_enum!`. Su valor principal es resolver automáticamente los nombres de tipos anónimos — escribes rutas legibles mientras la macro las expande a los nombres generados correctos.

## Concepto central

Cuando usas structs o enums anónimos, yuuka genera nombres como `_Root_0_anonymous`. Construirlos manualmente es verboso y frágil:

```rust
derive_struct!(Root {
    data: {
        name: String,
        score: f64,
    },
});

// Sin auto! — debes conocer el nombre generado
let val = Root {
    data: _Root_0_anonymous {
        name: "test".to_string(),
        score: 99.5,
    },
};

// Con auto! — simplemente usa { }
let val = auto!(Root {
    data: {
        name: "test".to_string(),
        score: 99.5,
    },
});
```

---

## Construcción de structs

### Struct básico

```rust
derive_struct!(Root {
    a: String,
    b: i32,
});

let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
});
```

### Structs anónimos anidados

```rust
derive_struct!(Root {
    a: String,
    b: i32,
    c: f64,
    d: {
        e: String = "world".to_string(),
        f: i32,
    },
});

let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: std::f64::consts::PI,
    d: {
        f: 24,
        ..Default::default()
    },
});
assert_eq!(obj.d.e, "world"); // Del valor por defecto
assert_eq!(obj.d.f, 24);      // Establecido explícitamente
```

### Expresión de propagación

Usa `..Default::default()` para rellenar los campos restantes con valores por defecto, igual que la sintaxis estándar de actualización de struct en Rust:

```rust
let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: 3.14,
    d: {
        f: 24,
        ..Default::default()  // e obtiene su valor por defecto "world"
    },
});
```

---

## Construcción de enums

### Variante unit

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        A,
        B(i32),
        C { a: String, b: i32 },
    }
);

assert_eq!(auto!(Root::A), Root::A);
```

### Variante tipo tupla

```rust
assert_eq!(auto!(Root::B(42)), Root::B(42));
```

### Variante tipo struct

```rust
assert_eq!(
    auto!(Root::C {
        a: "hello".to_string(),
        b: 42,
    }),
    Root::C {
        a: "hello".to_string(),
        b: 42,
    }
);
```

---

## Resolución de rutas de enums anónimos

Aquí es donde `auto!` realmente brilla. Para enums anónimos anidados dentro de variantes tipo tupla, `auto!` resuelve la ruta a través de múltiples niveles:

### Un solo nivel

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        D(enum {
            E,
            F(i32),
            G { a: String, b: i32 },
        }),
    }
);

// Sin auto! — verboso
let _ = Root::D(__Root::_Root_0_anonymous::E);

// Con auto! — limpio
assert_eq!(auto!(Root::D::E), Root::D(__Root::_Root_0_anonymous::E));

assert_eq!(auto!(Root::D::F(42)), Root::D(__Root::_Root_0_anonymous::F(42)));

assert_eq!(
    auto!(Root::D::G {
        a: "hello".to_string(),
        b: 42,
    }),
    Root::D(__Root::_Root_0_anonymous::G {
        a: "hello".to_string(),
        b: 42,
    })
);
```

### Rutas profundamente anidadas

`auto!` puede resolver rutas a través de anidamiento arbitrariamente profundo de enums anónimos:

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

// Resuelve: A::B → _A_0_anonymous::C → _A_1_anonymous::D → _A_2_anonymous::E → _A_3_anonymous::F
assert_eq!(
    auto!(A::B::C::D::E::F),
    A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))))
);

assert_eq!(
    auto!(A::B::C::D::E::G("hello".to_string())),
    A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(
        _A_3_anonymous::G("hello".to_string())
    ))))
);
```

---

## Uso mixto

Puedes anidar llamadas a `auto!` dentro de otras llamadas a `auto!` o construcción regular de structs:

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        outer: {
            a: enum B {
                C {
                    c: i32,
                    d: f64,
                },
            },
        },
    }
);

let val = auto!(Root {
    outer: {
        a: auto!(B::C { c: 42, d: std::f64::consts::PI }),
    },
});

assert_eq!(val.outer.a, B::C { c: 42, d: std::f64::consts::PI });
```

---

## Uso entre módulos

`auto!` funciona a través de fronteras de módulos siempre que los tipos y sus macros auxiliares estén en el ámbito:

```rust
#[macro_use]
mod definitions {
    use yuuka::derive_struct;

    derive_struct!(
        #[derive(PartialEq)]
        pub Root {
            a: String,
            b: i32,
        }
    );
}

mod usage {
    use yuuka::auto;
    use super::definitions::*;

    #[test]
    fn test() {
        assert_eq!(
            auto!(Root {
                a: "hello".to_string(),
                b: 42,
            }),
            Root {
                a: "hello".to_string(),
                b: 42,
            }
        );
    }
}
```

Cuando uses tipos anónimos entre módulos, asegúrate de que el módulo que los define esté marcado con `#[macro_use]`:

```rust
#[macro_use]
mod definitions {
    use yuuka::derive_struct;

    derive_struct!(Root {
        data: {
            value: String,
        },
    });
}

mod usage {
    use yuuka::auto;
    use super::definitions::*;

    fn create() {
        let val = auto!(Root {
            data: {
                value: "hello".to_string(),
            },
        });
    }
}
```

Para uso entre crates, consulta [Atributos y visibilidad — Uso entre crates](./attributes.md#uso-entre-crates).
