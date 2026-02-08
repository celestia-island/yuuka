# Atributos y visibilidad

Este documento cubre cómo controlar las macros derive, macros de atributos, visibilidad y exportación entre crates para los tipos generados por `derive_struct!` y `derive_enum!`.

---

## Macros derive extra

Coloca `#[derive(...)]` antes del nombre del tipo para agregar macros derive al tipo raíz generado:

```rust
use serde::{Serialize, Deserialize};
use yuuka::derive_struct;

derive_struct!(
    #[derive(Serialize, Deserialize)]
    Root {
        name: String,
        value: i32,
    }
);
```

> **Nota**: `Debug` y `Clone` se derivan automáticamente siempre. No necesitas especificarlos.

Lo mismo funciona para `derive_enum!`:

```rust
use yuuka::derive_enum;

derive_enum!(
    #[derive(Serialize, Deserialize)]
    enum Status {
        Active,
        Inactive,
    }
);
```

---

## Macros de atributos

Coloca macros de atributos después de `#[derive(...)]`:

```rust
derive_struct!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    Root {
        user_name: String,
        home_dir: String,
    }
);

let json = serde_json::to_string(&Root {
    user_name: "langyo".to_string(),
    home_dir: "/home/langyo".to_string(),
}).unwrap();
assert_eq!(json, r#"{"userName":"langyo","homeDir":"/home/langyo"}"#);
```

---

## Propagación recursiva de atributos

Usa `#[macros_recursive(...)]` para propagar atributos a **todos** los tipos en línea anidados:

```rust
derive_struct!(
    #[derive(Serialize, Deserialize)]
    #[macros_recursive(serde(rename_all = "camelCase"))]
    Root {
        nick_name: {
            chinese: {
                simplified_chinese: {
                    first_name: {
                        origin: String = "早濑".to_string(),
                        meme: String = "旱濑".to_string(),
                    },
                    last_name: String = "优香".to_string(),
                },
                traditional_chinese: {
                    first_name: String = "早瀨".to_string(),
                    last_name: String = "優香".to_string(),
                },
            },
            japanese: {
                first_name: String = "早瀬".to_string(),
                last_name: String = "ユウカ".to_string(),
            },
        },
    }
);

let json = serde_json::to_string(&Root::default()).unwrap();
// Todos los niveles anidados usan camelCase: "nickName", "simplifiedChinese", "firstName", etc.
```

`#[macros_recursive(...)]` aplica los atributos especificados a cada struct y enum generado en la jerarquía — no solo al tipo raíz.

---

## Atributos a nivel de campo

Coloca atributos directamente antes del nombre de un campo:

```rust
derive_struct!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    Root {
        nick_name: String,
        #[serde(rename = "location")]
        live_in: String,
    }
);

// "live_in" se serializa como "location" en lugar de "liveIn"
```

### Atributos de variante para enums

```rust
derive_enum!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Member {
        SaibaMomoi,
        SaibaMidori,
        #[serde(rename = "yuzu")]
        HanaokaYuzu,
        TendouAris,
    } = HanaokaYuzu
);

let json = serde_json::to_string(&Member::default()).unwrap();
assert_eq!(json, r#""yuzu""#);
```

---

## Atributos de tipo en tipos en línea

Puedes aplicar `#[derive(...)]` y atributos a tipos struct/enum en línea definidos en un campo. Colócalos **antes del nombre del campo**, usando `#[derive(...)]` para separar los atributos del campo de los atributos del tipo:

### Tipos en línea con nombre

```rust
derive_struct!(
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    Root {
        nick_name: String,
        #[serde(rename = "position")]
        #[derive(PartialEq)]
        #[serde(rename_all = "UPPERCASE")]
        location: Location {
            country: String,
            address: String,
        },
    }
);

// Root obtiene #[serde(deny_unknown_fields)]
// Location obtiene #[derive(PartialEq)] y #[serde(rename_all = "UPPERCASE")]
// El campo "location" se renombra a "position"
```

### Tipos en línea anónimos

Para tipos anónimos, usa `#[derive]` (derive vacío) como separador:

```rust
derive_struct!(
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    Root {
        nick_name: String,
        #[serde(rename = "position")]
        #[derive]
        #[serde(rename_all = "UPPERCASE")]
        location: {
            country: String = "kivotos".to_string(),
            address: String = "777".to_string(),
        },
    }
);

// El #[derive] vacío separa los atributos a nivel de campo (arriba) de los atributos a nivel de tipo (abajo)
```

### En variantes de enum

El mismo patrón funciona para variantes tipo tupla de enum:

```rust
derive_enum!(
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    enum Group {
        #[serde(rename = "777")]
        #[derive(PartialEq)]
        #[serde(rename_all = "UPPERCASE")]
        Millennium(enum Millennium {
            GameDevelopment(enum GameDevelopment {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            }),
            #[serde(rename = "C&C")]
            CAndC,
            Veritas,
        }),
    }
);
```

Y para variantes de enum anónimas:

```rust
derive_enum!(
    #[derive(Serialize, Deserialize)]
    enum Group {
        #[serde(rename = "777")]
        #[derive]
        #[serde(rename_all = "UPPERCASE")]
        Millennium(enum {
            GameDevelopment(enum GameDevelopment {
                Momoi, Midori, Yuzu, Arisu,
            } = Yuzu),
            CAndC,
        } = GameDevelopment(Default::default())),
    } = Millennium(Default::default())
);
```

---

## Visibilidad

### Modificador `pub`

Usa `pub` para hacer públicos los tipos generados y su módulo:

```rust
derive_struct!(
    pub Root {
        name: String,
    }
);

derive_enum!(
    pub enum Status {
        Active,
        Inactive,
    }
);
```

Esto genera `pub mod __Root` y `pub use __Root::*`, haciendo todos los tipos accesibles desde fuera del módulo actual.

### Visibilidad por defecto

Sin `pub`, los tipos son `pub(crate)`:

```rust
derive_struct!(
    Root {
        name: String,
    }
);
// Genera: pub(crate) mod __Root { ... }
// Genera: pub(crate) use __Root::*;
```

> **Nota**: Las declaraciones `pub` se usan típicamente a nivel de módulo o crate (fuera de funciones). Dentro de funciones de prueba, la visibilidad no importa.

---

## Uso entre crates

Para exportar los tipos generados y sus macros auxiliares de `auto!` para uso en otros crates, usa `#[macro_export]`:

### Crate de biblioteca

```rust
use yuuka::{derive_struct, derive_enum};

derive_struct!(
    #[derive(PartialEq)]
    #[macro_export]
    pub TestStruct {
        a: i32,
        b: String,
        c: {
            d: i32,
            e: String,
        },
    }
);

derive_enum!(
    #[macro_export]
    #[derive(PartialEq)]
    pub enum TestEnum {
        A(i32),
        B(String),
        C(enum C {
            D(i32),
            E(String),
            F(enum F {
                G(i32),
                H(String),
            }),
        }),
    }
);
```

> **Nota**: `#[macro_export]` se puede colocar antes o después de `#[derive(...)]` — ambas posiciones funcionan.

### Crate consumidor

```rust
use yuuka::auto;
use my_lib::*;

let test_struct = auto!(TestStruct {
    a: 1,
    b: "Hello".to_string(),
    c: {
        d: 2,
        e: "World".to_string(),
    },
});

let test_enum = auto!(TestEnum::C::F::H("Hello".to_string()));
assert_eq!(test_enum, TestEnum::C(C::F(F::H("Hello".to_string()))));
```

### Cómo funciona

`#[macro_export]` hace que las macros `macro_rules!` auxiliares generadas (como `__auto_TestStruct!`) estén disponibles a nivel de raíz del crate. Sin este atributo, las macros auxiliares solo son visibles dentro del crate que las define, y `auto!` no funcionará desde crates externos.

### Configuración de Cargo.toml

Para el crate de biblioteca, asegúrate de que se pueda enlazar correctamente:

```toml
[lib]
crate-type = ["rlib", "dylib"]
```

---

## Ejemplo completo

Combinando derives extra, atributos, propagación recursiva, atributos de campo y visibilidad:

```rust
use serde::{Serialize, Deserialize};
use yuuka::derive_struct;

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    #[macros_recursive(serde(rename_all = "camelCase"))]
    #[serde(deny_unknown_fields)]
    pub Config {
        app_name: String = "my_app".to_string(),
        #[serde(rename = "ver")]
        version: String = "1.0.0".to_string(),
        #[derive(PartialEq)]
        #[serde(rename_all = "UPPERCASE")]
        database: Database {
            host: String = "localhost".to_string(),
            port: u16 = 5432,
        },
        features: [Feature {
            name: String,
            enabled: bool = true,
        }],
        log_level: enum LogLevel {
            Debug,
            Info,
            Warn,
            Error,
        } = Info,
    }
);
```

Este ejemplo demuestra:

- `#[derive(PartialEq, Serialize, Deserialize)]` en el tipo raíz
- `#[macros_recursive(...)]` propagando `camelCase` a todos los tipos anidados
- `#[serde(deny_unknown_fields)]` como atributo a nivel de tipo
- `pub` para visibilidad pública
- Valores por defecto personalizados con `=`
- `#[serde(rename = "ver")]` como atributo a nivel de campo
- `#[derive(PartialEq)]` y `#[serde(rename_all = "UPPERCASE")]` como atributos de tipo en línea para `Database`
- Campos de array con structs en línea
- Enum en línea con valor por defecto
