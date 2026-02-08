# Macro `auto!`

La macro `auto!` simplifie la construction d'instances de types générés par `derive_struct!` et `derive_enum!`. Sa valeur principale est de résoudre automatiquement les noms de types anonymes — vous écrivez des chemins lisibles tandis que la macro les développe vers les noms générés corrects.

## Pourquoi `auto!` ?

Lorsque vous utilisez des structures ou énumérations anonymes, yuuka génère des noms comme `_Root_0_anonymous`. Les construire manuellement est verbeux et fragile :

```rust
derive_struct!(Root {
    data: {
        name: String,
        score: f64,
    },
});

// Sans auto! — vous devez connaître le nom généré
let val = Root {
    data: _Root_0_anonymous {
        name: "test".to_string(),
        score: 99.5,
    },
};

// Avec auto! — utilisez simplement { }
let val = auto!(Root {
    data: {
        name: "test".to_string(),
        score: 99.5,
    },
});
```

---

## Construction de structures

### Structure de base

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

### Structures anonymes imbriquées

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
assert_eq!(obj.d.e, "world"); // Depuis la valeur par défaut
assert_eq!(obj.d.f, 24);      // Défini explicitement
```

### Expression de propagation

Utilisez `..Default::default()` pour remplir les champs restants avec leurs valeurs par défaut, exactement comme la syntaxe de mise à jour de struct standard en Rust :

```rust
let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: 3.14,
    d: {
        f: 24,
        ..Default::default()  // e obtient sa valeur par défaut "world"
    },
});
```

---

## Construction d'énumérations

### Variante unitaire

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

### Variante tuple

```rust
assert_eq!(auto!(Root::B(42)), Root::B(42));
```

### Variante de type struct

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

## Résolution de chemins d'énumérations anonymes

C'est ici que `auto!` révèle tout son potentiel. Pour les énumérations anonymes imbriquées dans des variantes tuple, `auto!` résout le chemin à travers plusieurs niveaux :

### Un seul niveau

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

// Sans auto! — verbeux
let _ = Root::D(__Root::_Root_0_anonymous::E);

// Avec auto! — propre
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

### Chemins profondément imbriqués

`auto!` peut résoudre les chemins à travers un imbriquement arbitrairement profond d'énumérations anonymes :

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

// Résolution : A::B → _A_0_anonymous::C → _A_1_anonymous::D → _A_2_anonymous::E → _A_3_anonymous::F
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

## Utilisation mixte

Vous pouvez imbriquer des appels `auto!` à l'intérieur d'autres appels `auto!` ou de constructions de structures classiques :

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

## Utilisation inter-modules

`auto!` fonctionne au-delà des frontières de modules tant que les types et leurs macros auxiliaires sont accessibles :

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

Lorsque vous utilisez des types anonymes entre modules, assurez-vous que le module de définition est marqué avec `#[macro_use]` :

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

Pour l'utilisation inter-crates, consultez [Attributs et visibilité — Utilisation inter-crates](./attributes.md#utilisation-inter-crates).
