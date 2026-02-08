# Макрос `derive_enum!`

Макрос `derive_enum!` определяет самостоятельные типы перечислений с тем же стилем DSL-синтаксиса, что и `derive_struct!`. Он поддерживает все три формы вариантов, вложенные типы и значения по умолчанию.

## Базовый синтаксис

```rust
use yuuka::derive_enum;

derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);
```

Это генерирует перечисление с автоматически применённым `#[derive(Debug, Clone)]`.

---

## Формы вариантов

### Unit-варианты

Простые варианты без ассоциированных данных:

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

### Struct-подобные варианты

Варианты с именованными полями. Поля могут использовать встроенные определения структур:

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
// Генерирует независимую структуру `Target` наряду с перечислением `Action`.
```

### Tuple-подобные варианты

Варианты с позиционными данными. Могут содержать встроенные структуры, перечисления и статические типы:

```rust
derive_enum!(
    enum Message {
        Text(String),
        Data(Payload { content: Vec<u8>, size: usize }),
        Multi(String, i32, bool),
    }
);
```

### Смешанные варианты

Все три формы могут сосуществовать в одном перечислении:

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

## Вложенные перечисления

Варианты перечислений могут содержать другие встроенные перечисления:

### В tuple-вариантах

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

### Анонимные вложенные перечисления

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

Анонимные перечисления внутри вариантов получают имена вида `_Root_0_anonymous`. Вы можете ссылаться на них через модуль:

```rust
let _ = Root::D(__Root::_Root_0_anonymous::E);
```

> **Совет**: Используйте [`auto!`](./auto-macro.md), чтобы не работать с сгенерированными анонимными именами. `auto!(Root::D::E)` автоматически разрешает путь.

### Глубоко вложенные анонимные перечисления

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

// Ручное конструирование:
let _ = A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))));

// С помощью auto!:
use yuuka::auto;
let _ = auto!(A::B::C::D::E::F);
let _ = auto!(A::B::C::D::E::G("hello".to_string()));
```

---

## Значения по умолчанию

Укажите вариант по умолчанию с помощью `= ИмяВарианта` после закрывающей фигурной скобки:

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

### Значение по умолчанию для tuple-вариантов

```rust
derive_enum!(
    enum Value {
        Int(i32),
        Text(String),
    } = Int(0)
);
```

### Значение по умолчанию для вложенных анонимных перечислений

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

> **Примечание**: Если значение по умолчанию не указано, сгенерированная реализация `impl Default` использует `unimplemented!()`, что вызовет панику при выполнении. Всегда указывайте значение по умолчанию, если планируете использовать `Default::default()`.

---

## Дополнительные derive и макросы атрибутов

Как и в `derive_struct!`, вы можете передавать `#[derive(...)]` и макросы атрибутов:

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

Подробности о макросах атрибутов, рекурсивном распространении и атрибутах уровня вариантов смотрите в [Атрибуты и видимость](./attributes.md).

---

## Взаимодействие с `derive_struct!`

Перечисления, определённые с помощью `derive_enum!`, могут свободно использоваться совместно с `derive_struct!`. Вы также можете определять перечисления встроенно внутри полей структур — подробности об этом смотрите в разделе [Встроенные перечисления](./derive-struct.md#встроенные-перечисления) документации `derive_struct!`.
