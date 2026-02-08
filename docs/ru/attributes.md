# Атрибуты и видимость

В этом документе описывается управление derive-макросами, макросами атрибутов, видимостью и межкрейтовым экспортом для типов, сгенерированных `derive_struct!` и `derive_enum!`.

---

## Дополнительные derive-макросы

Поместите `#[derive(...)]` перед именем типа, чтобы добавить derive-макросы к сгенерированному корневому типу:

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

> **Примечание**: `Debug` и `Clone` всегда выводятся автоматически. Их не нужно указывать.

То же самое работает для `derive_enum!`:

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

## Макросы атрибутов

Размещайте макросы атрибутов после `#[derive(...)]`:

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

## Рекурсивное распространение атрибутов (`#[macros_recursive]`)

Используйте `#[macros_recursive(...)]` для распространения атрибутов на **все** вложенные встроенные типы:

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
// Все вложенные уровни используют camelCase: "nickName", "simplifiedChinese", "firstName" и т.д.
```

`#[macros_recursive(...)]` применяет указанные атрибуты к каждой структуре и перечислению, сгенерированным в иерархии — не только к корневому типу.

---

## Атрибуты уровня поля

Размещайте атрибуты непосредственно перед именем поля:

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

// "live_in" сериализуется как "location" вместо "liveIn"
```

### Атрибуты уровня вариантов для перечислений

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

## Атрибуты типа для встроенных типов (разделитель `#[derive]`)

Вы можете применять `#[derive(...)]` и атрибуты к встроенным типам struct/enum, определённым в поле. Размещайте их **перед именем поля**, используя `#[derive(...)]` для разделения атрибутов поля и атрибутов типа:

### Именованные встроенные типы

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

// Root получает #[serde(deny_unknown_fields)]
// Location получает #[derive(PartialEq)] и #[serde(rename_all = "UPPERCASE")]
// Поле "location" переименовывается в "position"
```

### Анонимные встроенные типы

Для анонимных типов используйте `#[derive]` (пустой derive) в качестве разделителя:

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

// Пустой #[derive] разделяет атрибуты уровня поля (выше) от атрибутов уровня типа (ниже)
```

### Атрибуты вариантов перечислений

Тот же паттерн работает для tuple-вариантов перечислений:

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

А также для анонимных вариантов перечислений:

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

## Видимость

### Модификатор `pub`

Используйте `pub` для того, чтобы сделать сгенерированные типы и их модуль публичными:

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

Это генерирует `pub mod __Root` и `pub use __Root::*`, делая все типы доступными извне текущего модуля.

### Видимость по умолчанию

Без `pub` типы имеют видимость `pub(crate)`:

```rust
derive_struct!(
    Root {
        name: String,
    }
);
// Генерирует: pub(crate) mod __Root { ... }
// Генерирует: pub(crate) use __Root::*;
```

> **Примечание**: Объявления `pub` обычно используются на уровне модуля или крейта (вне функций). Внутри тестовых функций видимость не имеет значения.

---

## Межкрейтовое использование

Для экспорта сгенерированных типов и их вспомогательных макросов `auto!` для использования в других крейтах используйте `#[macro_export]`:

### Крейт-библиотека

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

> **Примечание**: `#[macro_export]` можно размещать до или после `#[derive(...)]` — оба варианта работают.

### Потребляющий крейт

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

### Как это работает

`#[macro_export]` делает сгенерированные вспомогательные макросы `macro_rules!` (например, `__auto_TestStruct!`) доступными на уровне корня крейта. Без этого атрибута вспомогательные макросы видны только внутри определяющего крейта, и `auto!` не будет работать из внешних крейтов.

### Настройка Cargo.toml

Для крейта-библиотеки убедитесь, что он может быть корректно слинкован:

```toml
[lib]
crate-type = ["rlib", "dylib"]
```

---

## Полный пример

Вот комплексный пример, объединяющий несколько возможностей атрибутов:

```rust
use serde::{Serialize, Deserialize};
use yuuka::derive_struct;

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    #[macros_recursive(serde(rename_all = "camelCase"))]
    pub AppConfig {
        app_name: String = "MyApp".to_string(),
        #[serde(rename = "ver")]
        app_version: String = "1.0.0".to_string(),
        #[derive(PartialEq)]
        #[serde(deny_unknown_fields)]
        database: Database {
            host: String = "localhost".to_string(),
            port: u16 = 5432,
            db_name: String = "mydb".to_string(),
        },
        logging: {
            log_level: String = "info".to_string(),
            log_file?: String,
        },
    }
);
```

В этом примере:

- `#[macros_recursive(serde(rename_all = "camelCase"))]` применяет camelCase ко всем вложенным типам
- `#[serde(rename = "ver")]` — атрибут уровня поля для `app_version`
- `#[derive(PartialEq)]` и `#[serde(deny_unknown_fields)]` — атрибуты уровня типа для `Database`
- Анонимная структура для `logging` наследует рекурсивные атрибуты
- `log_file?: String` генерирует `log_file: Option<String>`
