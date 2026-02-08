# Macro `derive_enum!`

La macro `derive_enum!` définit des types d'énumérations autonomes avec le même style de syntaxe DSL que `derive_struct!`. Elle supporte les trois formes de variantes, les types imbriqués et les valeurs par défaut.

## Syntaxe de base

```rust
use yuuka::derive_enum;

derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);
```

Cela génère une énumération avec `#[derive(Debug, Clone)]` appliqué automatiquement.

---

## Formes de variantes

### Variantes unitaires

Variantes simples sans données associées :

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

### Variantes de type struct

Variantes avec des champs nommés. Les champs peuvent utiliser des définitions de structures en ligne :

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
// Génère une structure `Target` indépendante aux côtés de l'énumération `Action`.
```

### Variantes de type tuple

Variantes avec des données positionnelles. Peuvent contenir des structures en ligne, des énumérations et des types statiques :

```rust
derive_enum!(
    enum Message {
        Text(String),
        Data(Payload { content: Vec<u8>, size: usize }),
        Multi(String, i32, bool),
    }
);
```

### Variantes mixtes

Les trois formes peuvent coexister dans une même énumération :

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

## Énumérations imbriquées

Les variantes d'une énumération peuvent contenir d'autres énumérations en ligne :

### Dans les variantes tuple

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

### Énumérations imbriquées anonymes

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

Les énumérations anonymes à l'intérieur des variantes sont nommées comme `_Root_0_anonymous`. Vous pouvez les référencer via le module :

```rust
let _ = Root::D(__Root::_Root_0_anonymous::E);
```

> **Astuce** : Utilisez [`auto!`](./auto-macro.md) pour éviter de manipuler directement les noms anonymes générés. `auto!(Root::D::E)` résout automatiquement le chemin.

### Énumérations anonymes profondément imbriquées

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

// Construction manuelle :
let _ = A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))));

// Avec auto! :
use yuuka::auto;
let _ = auto!(A::B::C::D::E::F);
let _ = auto!(A::B::C::D::E::G("hello".to_string()));
```

---

## Valeurs par défaut

Spécifiez une variante par défaut avec `= NomVariante` après l'accolade fermante :

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

### Valeur par défaut pour les variantes tuple

```rust
derive_enum!(
    enum Value {
        Int(i32),
        Text(String),
    } = Int(0)
);
```

### Valeur par défaut pour les énumérations anonymes imbriquées

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

> **Remarque** : Lorsqu'aucune valeur par défaut n'est spécifiée, le `impl Default` généré utilise `unimplemented!()`, ce qui provoquera un panic à l'exécution si appelé. Spécifiez toujours une valeur par défaut si vous prévoyez d'utiliser `Default::default()`.

---

## Macros derive et attributs supplémentaires

Tout comme `derive_struct!`, vous pouvez passer des `#[derive(...)]` et des macros d'attributs :

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

Consultez [Attributs et visibilité](./attributes.md) pour tous les détails sur les macros d'attributs, la propagation récursive et les attributs au niveau des variantes.
