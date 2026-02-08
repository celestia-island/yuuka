# Macro `derive_struct!`

La macro `derive_struct!` est le cœur de yuuka. Elle vous permet de définir des hiérarchies complexes de structures imbriquées en utilisant un DSL concis, semblable à JSON. Tous les types en ligne sont automatiquement extraits en définitions indépendantes de structures/énumérations de premier niveau.

## Syntaxe de base

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

Cela génère une structure avec `#[derive(Debug, Clone, Default)]` appliqué automatiquement :

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## Structures imbriquées

Définissez des sous-structures en ligne directement à l'intérieur d'une structure parente en spécifiant `NomChamp { ... }` comme type :

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

Cela génère **trois** structures indépendantes : `Root`, `Info` et `Detail`. Chacune est une structure Rust normale dont tous les champs sont publics au sein du module généré.

Vous pouvez imbriquer à une profondeur arbitraire :

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

## Structures anonymes

Omettez le nom du type pour créer des structures nommées automatiquement :

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

La structure anonyme est automatiquement nommée `_Root_0_anonymous`. Lorsqu'il y a plusieurs types anonymes, ils sont numérotés séquentiellement :

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
// Génère : _Root_0_anonymous (pour a), _Root_1_anonymous (pour c)
```

Les structures anonymes peuvent être profondément imbriquées :

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

> **Astuce** : Utilisez la macro [`auto!`](./auto-macro.md) pour construire des instances de structures anonymes sans connaître les noms générés.

---

## Types tableau (Vec)

Utilisez la syntaxe `[Type { ... }]` pour définir des champs `Vec<Type>` :

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// Génère le champ : items: Vec<Item>
```

### Éléments de tableau anonymes

```rust
derive_struct!(
    Root {
        items: [{
            name: String,
        }],
    }
);
// Génère : items: Vec<_Root_0_anonymous>
```

### Tableaux d'énumérations

```rust
derive_struct!(
    Root {
        statuses: [enum Status {
            Active,
            Inactive,
        }],
    }
);
// Génère : statuses: Vec<Status>
```

### Tableaux d'énumérations anonymes

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
// Génère : values: Vec<_Root_0_anonymous>
```

---

## Types optionnels (Option)

Ajoutez `?` au nom du champ pour encapsuler le type dans `Option<T>` :

```rust
derive_struct!(
    Root {
        required: String,
        optional?: String,
    }
);
// Génère :
//   required: String,
//   optional: Option<String>,
```

### Option avec structure en ligne

```rust
derive_struct!(
    Root {
        detail?: Detail {
            info: String,
        },
    }
);
// Génère : detail: Option<Detail>
```

### Option avec structure anonyme

```rust
derive_struct!(
    Root {
        data?: {
            value: String,
        },
    }
);
// Génère : data: Option<_Root_0_anonymous>
```

### Option avec énumération en ligne

```rust
derive_struct!(
    Root {
        status?: enum Status {
            Active,
            Inactive,
        },
    }
);
// Génère : status: Option<Status>
```

### Option dans les variantes d'énumération

La syntaxe `?` fonctionne également à l'intérieur des variantes struct d'une énumération :

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

## Valeurs par défaut

Assignez des valeurs par défaut avec `=` après le type :

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

### Comportement

- Les champs **avec** un `= valeur` explicite utilisent cette valeur dans le `impl Default` généré.
- Les champs **sans** `= valeur` utilisent `Default::default()` (par ex. `0` pour les nombres, `""` pour String, `false` pour bool).
- Si **un** champ possède une valeur par défaut personnalisée, la macro génère un bloc `impl Default` manuel au lieu de `#[derive(Default)]`.

### Valeurs par défaut pour les tableaux

```rust
derive_struct!(
    Root {
        // Vide par défaut
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Le Vec est vide par défaut
```

Avec une valeur par défaut explicite pour le tableau :

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

// Les nouveaux éléments obtiennent les valeurs par défaut de Item
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### Valeurs par défaut pour les énumérations

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

Tableau d'énumérations avec valeurs par défaut :

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
assert_eq!(root.members[0], Member::Arisu); // Depuis la valeur par défaut du Vec
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // Depuis la valeur par défaut de l'énumération
```

---

## Énumérations en ligne

Définissez des énumérations en ligne dans les champs de structures :

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

### Formes de variantes

Les variantes d'énumération supportent trois formes :

**Variantes unitaires** — sans données associées :

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**Variantes de type struct** — champs nommés, qui peuvent eux-mêmes contenir des structures et énumérations en ligne :

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

**Variantes de type tuple** — données positionnelles, supportant les structures en ligne, les champs multiples et les types statiques :

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### Énumérations imbriquées

Les énumérations peuvent être imbriquées dans les variantes d'énumération :

```rust
// Énumération dans une variante de type struct
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

// Énumération dans une variante de type tuple
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

### Tableaux d'énumérations dans les variantes

```rust
// Vec<enum> dans une variante de type struct
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

// Vec<enum> dans une variante de type tuple
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

## Types par référence

Utilisez la syntaxe de chemin pour référencer des types définis à l'extérieur :

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

> **Remarque** : Étant donné que les types générés se trouvent à l'intérieur d'un module (`__Root`), vous devez généralement utiliser `super::` pour référencer des types depuis la portée extérieure. Le chemin exact dépend de l'emplacement du type externe par rapport à l'invocation de la macro.

Les noms de champs sont flexibles — les noms en snake_case et PascalCase fonctionnent tous les deux :

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
