# Ejemplos

Ejemplos de uso real y una explicación del código generado por las macros de yuuka.

---

## Pack de idiomas (i18n)

Un caso de uso típico: definir una estructura de pack de idiomas anidada que se mapea directamente a archivos JSON. Soporta serialización/deserialización con serde.

```rust
use anyhow::Result;
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

fn main() -> Result<()> {
    derive_struct!(
        #[derive(PartialEq, Serialize, Deserialize)]
        LanguagePack {
            是: String,
            否: String,
            确认: String,
            取消: String,
            保存: String,
            主页: {
                启动: String,
                设置: String,
            },
            设置: {
                虚拟机路径: String,
                程序本体路径: String,
                网络配置: {
                    网络配置: String,
                    是否启用代理: String,
                    代理地址: String,
                    是否启用IPV6: String,
                },
            },
        }
    );

    let config = auto!(LanguagePack {
        是: "Yes".to_string(),
        否: "No".to_string(),
        确认: "Confirm".to_string(),
        取消: "Cancel".to_string(),
        保存: "Save".to_string(),
        主页: {
            启动: "Start".to_string(),
            设置: "Settings".to_string(),
        },
        设置: {
            虚拟机路径: "VM Path".to_string(),
            程序本体路径: "Program Path".to_string(),
            网络配置: {
                网络配置: "Network Config".to_string(),
                是否启用代理: "Enable Proxy".to_string(),
                代理地址: "Proxy Address".to_string(),
                是否启用IPV6: "Enable IPV6".to_string(),
            },
        },
    });

    // Deserializar desde JSON
    let json_raw = r#"
    {
        "是": "Yes", "否": "No", "确认": "Confirm",
        "取消": "Cancel", "保存": "Save",
        "主页": { "启动": "Start", "设置": "Settings" },
        "设置": {
            "虚拟机路径": "VM Path",
            "程序本体路径": "Program Path",
            "网络配置": {
                "网络配置": "Network Config",
                "是否启用代理": "Enable Proxy",
                "代理地址": "Proxy Address",
                "是否启用IPV6": "Enable IPV6"
            }
        }
    }"#;

    let config_from_json = serde_json::from_str::<LanguagePack>(json_raw)?;
    assert_eq!(config, config_from_json);
    assert_eq!(config.设置.网络配置.代理地址, "Proxy Address");

    Ok(())
}
```

### Puntos clave del pack de idiomas

- Los nombres de campo pueden ser **no ASCII** (caracteres chinos, etc.) — funcionan tanto como identificadores de Rust como claves JSON.
- La macro `auto!` maneja la construcción de sub-structs anónimos (主页, 设置, 网络配置) sin problemas.
- Los tipos generados son completamente compatibles con serde para serialización JSON de ida y vuelta.

---

## Configuración de enrutador

Un ejemplo más complejo que modela una configuración de proxy inverso / enrutador de servidor con arrays anidados y enums en línea.

```rust
use anyhow::Result;
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

fn main() -> Result<()> {
    derive_struct!(
        #[derive(PartialEq, Serialize, Deserialize)]
        Config {
            port: u16,
            services: [Service {
                domain: Vec<String>,
                rules: [Rule {
                    pattern: String,
                    method: enum Method {
                        Redirect { url: String },
                        Proxy { host: String },
                        StaticFile { path: String },
                        StaticDir { path: String },
                    },
                }],
            }],
        }
    );

    let config = auto!(Config {
        port: 8080,
        services: vec![Service {
            domain: vec!["example.com".to_string()],
            rules: vec![
                Rule {
                    pattern: "^/$".to_string(),
                    method: Method::Redirect {
                        url: "https://example.com/index.html".to_string(),
                    },
                },
                Rule {
                    pattern: "^/api".to_string(),
                    method: Method::Proxy {
                        host: "http://localhost:8081".to_string(),
                    },
                },
                Rule {
                    pattern: "^/static".to_string(),
                    method: Method::StaticDir {
                        path: "/var/www/static".to_string(),
                    },
                },
            ],
        }],
    });

    // Esta estructura se mapea directamente hacia/desde JSON
    let json = serde_json::to_string_pretty(&config)?;
    let config_from_json: Config = serde_json::from_str(&json)?;
    assert_eq!(config, config_from_json);

    Ok(())
}
```

### Puntos clave del enrutador

- **`[Service { ... }]`** genera `services: Vec<Service>` con `Service` como struct independiente.
- **`[Rule { ... }]`** anidado dentro de Service genera otro `Vec<Rule>` con struct Rule en línea.
- **`enum Method { ... }`** define un enum en línea, con variantes tipo struct para diferentes métodos de enrutamiento.
- Toda la configuración se puede cargar desde / guardar en JSON.

---

## Configuración con enumeraciones

Los enums definidos mediante `derive_enum!` se combinan naturalmente con `derive_struct!` para configuraciones tipadas:

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, derive_enum, auto};

derive_enum!(
    #[derive(PartialEq, Serialize, Deserialize)]
    enum LogLevel {
        Debug,
        Info,
        Warn,
        Error,
    } = Info
);

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    AppConfig {
        name: String = "my_app".to_string(),
        log_level: super::LogLevel,
        database: {
            host: String = "localhost".to_string(),
            port: u16 = 5432,
            pool_size: u32 = 10,
        },
    }
);

let config = auto!(AppConfig {
    log_level: LogLevel::Warn,
    ..Default::default()
});

assert_eq!(config.name, "my_app");
assert_eq!(config.database.port, 5432);
```

### Puntos clave de la configuración

- `derive_enum!` define `LogLevel` como un tipo independiente con valor por defecto `Info`.
- `derive_struct!` referencia `LogLevel` mediante `super::LogLevel` (porque los tipos generados viven en un módulo).
- `..Default::default()` rellena todos los campos no especificados con sus valores por defecto.
- Ambas macros comparten la misma compatibilidad con serde para serialización.

---

## Estructura del código generado

Entender lo que yuuka genera ayuda a depurar y trabajar efectivamente con la biblioteca.

Cuando escribes:

```rust
derive_struct!(
    #[derive(Serialize)]
    pub Root {
        name: String,
        child: Child {
            value: i32,
        },
    }
);
```

La macro genera aproximadamente:

```rust
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
pub mod __Root {
    use super::*;

    #[derive(Debug, Clone, Serialize, Default)]
    pub struct Root {
        pub name: String,
        pub child: Child,
    }

    #[derive(Debug, Clone, Serialize, Default)]
    pub struct Child {
        pub value: i32,
    }

    // Macros auxiliares para auto!
    macro_rules! __auto_Root {
        (name $($tt:tt)*) => { $($tt)* };
        (child { $($tt:tt)* }) => { ::yuuka::auto!(Child { $($tt)* }) };
        (child $($tt:tt)*) => { $($tt)* };
        // ... más reglas para cada campo
    }

    macro_rules! __auto_Child {
        (value $($tt:tt)*) => { $($tt)* };
        // ... más reglas para cada campo
    }
}
pub use __Root::*;
```

### Aspectos clave del código generado

1. **Envoltorio de módulo**: Todos los tipos se colocan en un módulo llamado `__NombreTipo` para evitar colisiones de nombres. Todo se re-exporta con `use __NombreTipo::*`.

2. **Derives automáticos**: `Debug` y `Clone` siempre se agregan. Tus macros `#[derive(...)]` personalizadas se añaden a continuación.

3. **Implementación de Default**: Si ningún campo tiene valores por defecto personalizados → `#[derive(Default)]`. Si algún campo tiene `= valor` → `impl Default { ... }` manual.

4. **Macros auxiliares**: Para cada tipo, se genera una macro `__auto_NombreTipo!`. Son macros `macro_rules!` que la macro procedural `auto!` llama para resolver tipos de campo — particularmente nombres de structs/enums anónimos.

5. **Importaciones super**: `use super::*` trae el ámbito exterior al módulo, razón por la cual los tipos externos necesitan el prefijo `super::` al ser referenciados.

### Convención de nombres de módulos

| Entrada | Nombre del módulo |
| --- | --- |
| `Root { ... }` | `__Root` |
| `Config { ... }` | `__Config` |
| Campo anónimo en Root | `_Root_0_anonymous`, `_Root_1_anonymous`, ... |
| Campo anónimo en enum A | `_A_0_anonymous`, `_A_1_anonymous`, ... |
