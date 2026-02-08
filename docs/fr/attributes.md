# Attributs et visibilité

Ce document couvre le contrôle des macros derive, des macros d'attributs, de la visibilité et de l'exportation inter-crates pour les types générés par `derive_struct!` et `derive_enum!`.

---

## Macros derive supplémentaires

Placez `#[derive(...)]` avant le nom du type pour ajouter des macros derive au type racine généré :

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

> **Remarque** : `Debug` et `Clone` sont toujours dérivés automatiquement. Vous n'avez pas besoin de les spécifier.

Le même principe s'applique pour `derive_enum!` :

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

## Macros d'attributs

Placez les macros d'attributs après `#[derive(...)]` :

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

## Propagation récursive des attributs

Utilisez `#[macros_recursive(...)]` pour propager des attributs à **tous** les types en ligne imbriqués :

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
// Tous les niveaux imbriqués utilisent camelCase : "nickName", "simplifiedChinese", "firstName", etc.
```

`#[macros_recursive(...)]` applique les attributs spécifiés à chaque structure et énumération générée dans la hiérarchie — pas seulement au type racine.

---

## Attributs au niveau des champs

Placez les attributs directement avant le nom d'un champ :

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

// "live_in" est sérialisé en "location" au lieu de "liveIn"
```

### Attributs au niveau des variantes pour les énumérations

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

## Attributs au niveau des types en ligne

Vous pouvez appliquer `#[derive(...)]` et des attributs aux types struct/enum en ligne définis dans un champ. Placez-les **avant le nom du champ**, en utilisant `#[derive(...)]` pour séparer les attributs du champ des attributs du type :

### Types en ligne nommés

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

// Root obtient #[serde(deny_unknown_fields)]
// Location obtient #[derive(PartialEq)] et #[serde(rename_all = "UPPERCASE")]
// Le champ "location" est renommé en "position"
```

### Types en ligne anonymes

Pour les types anonymes, utilisez `#[derive]` (derive vide) comme séparateur :

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

// Le #[derive] vide sépare les attributs au niveau du champ (au-dessus) des attributs au niveau du type (en dessous)
```

### Sur les variantes d'énumération

Le même schéma fonctionne pour les variantes tuple d'énumération :

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

Et pour les variantes d'énumération anonymes :

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

## Visibilité

### Modificateur `pub`

Utilisez `pub` pour rendre les types générés et leur module publics :

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

Cela génère `pub mod __Root` et `pub use __Root::*`, rendant tous les types accessibles depuis l'extérieur du module courant.

### Visibilité par défaut

Sans `pub`, les types ont la visibilité `pub(crate)` :

```rust
derive_struct!(
    Root {
        name: String,
    }
);
// Génère : pub(crate) mod __Root { ... }
// Génère : pub(crate) use __Root::*;
```

> **Remarque** : Les déclarations `pub` sont généralement utilisées au niveau du module ou du crate (en dehors des fonctions). À l'intérieur des fonctions de test, la visibilité n'a pas d'importance.

---

## Utilisation inter-crates

Pour exporter les types générés et leurs macros auxiliaires `auto!` afin de les utiliser dans d'autres crates, utilisez `#[macro_export]` :

### Crate bibliothèque

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

> **Remarque** : `#[macro_export]` peut être placé avant ou après `#[derive(...)]` — les deux positions fonctionnent.

### Crate consommateur

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

### Fonctionnement interne

`#[macro_export]` rend les macros `macro_rules!` auxiliaires générées (comme `__auto_TestStruct!`) disponibles au niveau racine du crate. Sans cet attribut, les macros auxiliaires ne sont visibles qu'à l'intérieur du crate de définition, et `auto!` ne fonctionnera pas depuis des crates externes.

### Configuration du Cargo.toml

Pour le crate bibliothèque, assurez-vous qu'il peut être lié correctement :

```toml
[lib]
crate-type = ["rlib", "dylib"]
```
