# Exemples

Exemples d'utilisation concrets et explication du code généré par les macros yuuka.

---

## Pack de langue (i18n)

Un cas d'utilisation typique : définir une structure de pack de langue imbriquée qui correspond directement aux fichiers JSON. Supporte la sérialisation/désérialisation avec serde.

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

    // Désérialisation depuis JSON
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

### Points clés du pack de langue

- Les noms de champs peuvent être **non-ASCII** (caractères chinois, etc.) — ils fonctionnent à la fois comme identifiants Rust et comme clés JSON.
- La macro `auto!` gère la construction des sous-structures anonymes (主页, 设置, 网络配置) de manière transparente.
- Les types générés sont entièrement compatibles avec serde pour la sérialisation JSON aller-retour.

---

## Configuration de routeur serveur

Un exemple plus complexe modélisant une configuration de proxy inverse / routeur serveur avec des tableaux imbriqués et des énumérations en ligne.

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

    // Cette structure correspond directement au format JSON aller-retour
    let json = serde_json::to_string_pretty(&config)?;
    let config_from_json: Config = serde_json::from_str(&json)?;
    assert_eq!(config, config_from_json);

    Ok(())
}
```

### Points clés du routeur

- **`[Service { ... }]`** génère `services: Vec<Service>` avec `Service` comme structure indépendante.
- **`[Rule { ... }]`** imbriqué dans Service génère un autre `Vec<Rule>` avec la structure Rule en ligne.
- **`enum Method { ... }`** définit une énumération en ligne, avec des variantes de type struct pour différentes méthodes de routage.
- L'ensemble de la configuration peut être chargé depuis / sauvegardé vers du JSON.

---

## Configuration d'application avec énumérations

Un exemple combinant des structures imbriquées et des énumérations en ligne pour une configuration d'application :

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    AppConfig {
        app_name: String = "MonApp".to_string(),
        version: String = "1.0.0".to_string(),
        database: {
            host: String = "localhost".to_string(),
            port: u16 = 5432,
            driver: enum DbDriver {
                Postgres,
                MySQL,
                SQLite,
            } = Postgres,
        },
        logging: {
            level: enum LogLevel {
                Debug,
                Info,
                Warn,
                Error,
            } = Info,
            output: enum LogOutput {
                Console,
                File { path: String },
            } = Console,
        },
    }
);

let config = AppConfig::default();
assert_eq!(config.app_name, "MonApp");
assert_eq!(config.database.port, 5432);

let custom = auto!(AppConfig {
    app_name: "Serveur".to_string(),
    version: "2.0.0".to_string(),
    database: {
        host: "db.example.com".to_string(),
        port: 3306,
        driver: auto!(DbDriver::MySQL),
    },
    logging: {
        level: auto!(LogLevel::Debug),
        output: auto!(LogOutput::File {
            path: "/var/log/app.log".to_string(),
        }),
    },
});
```

### Points clés de la configuration

- Les valeurs par défaut (`= valeur`) permettent de créer des configurations prêtes à l'emploi via `Default::default()`.
- Les énumérations en ligne comme `DbDriver` et `LogLevel` offrent une correspondance de motifs sûre au niveau des types.
- Les structures anonymes (database, logging) gardent la configuration organisée sans polluer l'espace de noms.
- `auto!` simplifie la construction en résolvant automatiquement les types anonymes imbriqués.

---

## Structure du code généré

Comprendre ce que yuuka génère aide à déboguer et à travailler efficacement avec la bibliothèque.

Lorsque vous écrivez :

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

La macro génère approximativement :

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

    // Macros auxiliaires pour auto!
    macro_rules! __auto_Root {
        (name $($tt:tt)*) => { $($tt)* };
        (child { $($tt:tt)* }) => { ::yuuka::auto!(Child { $($tt)* }) };
        (child $($tt:tt)*) => { $($tt)* };
        // ... autres règles pour chaque champ
    }

    macro_rules! __auto_Child {
        (value $($tt:tt)*) => { $($tt)* };
        // ... autres règles pour chaque champ
    }
}
pub use __Root::*;
```

### Aspects essentiels

1. **Encapsulation en module** : Tous les types sont placés dans un module nommé `__NomType` pour éviter les collisions de noms. Le tout est réexporté avec `use __NomType::*`.

2. **Dérivations automatiques** : `Debug` et `Clone` sont toujours ajoutés. Vos macros `#[derive(...)]` personnalisées sont ajoutées à la suite.

3. **Implémentation de Default** : Si aucun champ n'a de valeur par défaut personnalisée → `#[derive(Default)]`. Si un champ a `= valeur` → `impl Default { ... }` manuel.

4. **Macros auxiliaires** : Pour chaque type, une macro `__auto_NomType!` est générée. Ce sont des macros `macro_rules!` que la macro procédurale `auto!` appelle pour résoudre les types de champs — en particulier les noms de structures/énumérations anonymes.

5. **Imports super** : `use super::*` importe la portée extérieure dans le module, c'est pourquoi les types externes nécessitent le préfixe `super::` lorsqu'ils sont référencés.

### Convention de nommage des modules

| Entrée | Nom du module |
| --- | --- |
| `Root { ... }` | `__Root` |
| `Config { ... }` | `__Config` |
| Champ anonyme dans Root | `_Root_0_anonymous`, `_Root_1_anonymous`, ... |
| Champ anonyme dans l'énumération A | `_A_0_anonymous`, `_A_1_anonymous`, ... |
