# Макрос `auto!`

Макрос `auto!` упрощает конструирование экземпляров типов, сгенерированных `derive_struct!` и `derive_enum!`. Его основная ценность заключается в автоматическом разрешении имён анонимных типов — вы пишете человекочитаемые пути, а макрос раскрывает их в правильные сгенерированные имена.

## Основная концепция

При использовании анонимных структур или перечислений yuuka генерирует имена вида `_Root_0_anonymous`. Ручное конструирование таких типов громоздко и хрупко:

```rust
derive_struct!(Root {
    data: {
        name: String,
        score: f64,
    },
});

// Без auto! — необходимо знать сгенерированное имя
let val = Root {
    data: _Root_0_anonymous {
        name: "test".to_string(),
        score: 99.5,
    },
};

// С auto! — просто используйте { }
let val = auto!(Root {
    data: {
        name: "test".to_string(),
        score: 99.5,
    },
});
```

---

## Конструирование структур

### Базовая структура

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

### Вложенные анонимные структуры

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
assert_eq!(obj.d.e, "world"); // Из значения по умолчанию
assert_eq!(obj.d.f, 24);      // Задано явно
```

### Spread-выражение

Используйте `..Default::default()` для заполнения оставшихся полей значениями по умолчанию, как в стандартном синтаксисе обновления структур Rust:

```rust
let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: 3.14,
    d: {
        f: 24,
        ..Default::default()  // e получает значение по умолчанию "world"
    },
});
```

---

## Конструирование перечислений

### Unit-вариант

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

### Tuple-вариант

```rust
assert_eq!(auto!(Root::B(42)), Root::B(42));
```

### Struct-подобный вариант

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

## Разрешение путей анонимных перечислений

Именно здесь `auto!` проявляет себя наилучшим образом. Для анонимных перечислений, вложенных в tuple-варианты, `auto!` разрешает путь через несколько уровней:

### Один уровень

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

// Без auto! — многословно
let _ = Root::D(__Root::_Root_0_anonymous::E);

// С auto! — чисто
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

### Глубоко вложенные пути

`auto!` может разрешать пути через произвольно глубокую вложенность анонимных перечислений:

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

// Разрешает: A::B → _A_0_anonymous::C → _A_1_anonymous::D → _A_2_anonymous::E → _A_3_anonymous::F
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

## Смешанное использование

Вы можете вкладывать вызовы `auto!` друг в друга или в обычное конструирование структур:

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

## Межмодульное использование

`auto!` работает через границы модулей при условии, что типы и их вспомогательные макросы доступны в области видимости:

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

При использовании анонимных типов между модулями убедитесь, что определяющий модуль помечен `#[macro_use]`:

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

Информацию о межкрейтовом использовании смотрите в [Атрибуты и видимость — Межкрейтовое использование](./attributes.md#межкрейтовое-использование).
