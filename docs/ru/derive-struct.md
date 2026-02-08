# Макрос `derive_struct!`

Макрос `derive_struct!` — это ядро yuuka. Он позволяет определять сложные вложенные иерархии структур с помощью лаконичного JSON-подобного DSL. Все встроенные типы автоматически извлекаются в независимые определения структур/перечислений верхнего уровня.

## Базовый синтаксис

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

Это генерирует структуру с автоматически применённым `#[derive(Debug, Clone, Default)]`:

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## Вложенные структуры

Определяйте встроенные подструктуры непосредственно внутри родительской структуры, указав `ИмяТипа { ... }` в качестве типа:

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

Это генерирует **три** независимые структуры: `Root`, `Info` и `Detail`. Каждая является обычной структурой Rust со всеми полями, доступными внутри сгенерированного модуля.

Вложенность может быть произвольной глубины:

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

## Анонимные структуры

Опустите имя типа для создания автоматически именуемых структур:

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

Анонимная структура автоматически получает имя `_Root_0_anonymous`. При наличии нескольких анонимных типов они нумеруются последовательно:

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
// Генерирует: _Root_0_anonymous (для a), _Root_1_anonymous (для c)
```

Анонимные структуры могут быть глубоко вложенными:

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

> **Совет**: Используйте макрос [`auto!`](./auto-macro.md) для конструирования экземпляров анонимных структур без необходимости знать сгенерированные имена.

---

## Типы массивов (Vec)

Используйте синтаксис `[Type { ... }]` для определения полей типа `Vec<Type>`:

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// Генерирует поле: items: Vec<Item>
```

### Анонимные элементы массивов

```rust
derive_struct!(
    Root {
        items: [{
            name: String,
        }],
    }
);
// Генерирует: items: Vec<_Root_0_anonymous>
```

### Массивы перечислений

```rust
derive_struct!(
    Root {
        statuses: [enum Status {
            Active,
            Inactive,
        }],
    }
);
// Генерирует: statuses: Vec<Status>
```

### Анонимные массивы перечислений

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
// Генерирует: values: Vec<_Root_0_anonymous>
```

---

## Опциональные типы (Option)

Добавьте `?` к имени поля, чтобы обернуть тип в `Option<T>`:

```rust
derive_struct!(
    Root {
        required: String,
        optional?: String,
    }
);
// Генерирует:
//   required: String,
//   optional: Option<String>,
```

### Option со встроенной структурой

```rust
derive_struct!(
    Root {
        detail?: Detail {
            info: String,
        },
    }
);
// Генерирует: detail: Option<Detail>
```

### Option с анонимной структурой

```rust
derive_struct!(
    Root {
        data?: {
            value: String,
        },
    }
);
// Генерирует: data: Option<_Root_0_anonymous>
```

### Option со встроенным перечислением

```rust
derive_struct!(
    Root {
        status?: enum Status {
            Active,
            Inactive,
        },
    }
);
// Генерирует: status: Option<Status>
```

### Option внутри вариантов перечисления

Синтаксис `?` также работает внутри struct-вариантов перечислений:

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

## Значения по умолчанию

Присвойте значения по умолчанию с помощью `=` после типа:

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

### Поведение

- Поля **с** явным `= значение` используют это значение в сгенерированной реализации `impl Default`.
- Поля **без** `= значение` используют `Default::default()` (например, `0` для чисел, `""` для String, `false` для bool).
- Если **хотя бы одно** поле имеет пользовательское значение по умолчанию, макрос генерирует ручной блок `impl Default` вместо `#[derive(Default)]`.

### Значения по умолчанию для массивов

```rust
derive_struct!(
    Root {
        // Пусто по умолчанию
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Vec пуст по умолчанию
```

С явным значением по умолчанию для массива:

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

// Новые элементы получают значения по умолчанию уровня Item
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### Значения по умолчанию для перечислений

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

Массив перечислений со значениями по умолчанию:

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
assert_eq!(root.members[0], Member::Arisu); // Из значения по умолчанию Vec
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // Из значения по умолчанию enum
```

---

## Встроенные перечисления

Определяйте перечисления непосредственно в полях структур:

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

### Формы вариантов

Варианты перечислений поддерживают три формы:

**Unit-варианты** — без ассоциированных данных:

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**Struct-подобные варианты** — именованные поля, которые сами могут содержать встроенные структуры и перечисления:

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

**Tuple-подобные варианты** — позиционные данные, поддерживающие встроенные структуры, несколько полей и статические типы:

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### Вложенные перечисления

Перечисления могут быть вложены внутрь вариантов перечислений:

```rust
// Перечисление в struct-подобном варианте
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

// Перечисление в tuple-подобном варианте
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

### Массивы перечислений в вариантах

```rust
// Vec<enum> в struct-подобном варианте
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

// Vec<enum> в tuple-подобном варианте
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

## Ссылочные типы

Используйте синтаксис путей для ссылки на внешне определённые типы:

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

> **Примечание**: Поскольку сгенерированные типы находятся внутри модуля (`__Root`), обычно необходимо использовать `super::` для ссылки на типы из внешней области видимости. Точный путь зависит от того, где определён внешний тип относительно вызова макроса.

Имена полей гибки — работают как имена в snake_case, так и в PascalCase:

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
