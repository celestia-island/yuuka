# Примеры

Реальные примеры использования и объяснение кода, генерируемого макросами yuuka.

---

## Языковой пакет (i18n)

Типичный случай использования: определение вложенной структуры языкового пакета, которая напрямую соответствует JSON-файлам. Поддерживает сериализацию/десериализацию с помощью serde.

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

    // Десериализация из JSON
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

### Ключевые аспекты языкового пакета

- Имена полей могут быть **не-ASCII** (китайские символы и т.д.) — они работают как идентификаторы Rust и как ключи JSON.
- Макрос `auto!` бесшовно обрабатывает конструирование анонимных подструктур (主页, 设置, 网络配置).
- Сгенерированные типы полностью совместимы с serde для двусторонней JSON-сериализации.

---

## Конфигурация маршрутизатора

Более сложный пример моделирования конфигурации обратного прокси / серверного маршрутизатора с вложенными массивами и встроенными перечислениями.

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

    // Эта структура напрямую отображается в/из JSON
    let json = serde_json::to_string_pretty(&config)?;
    let config_from_json: Config = serde_json::from_str(&json)?;
    assert_eq!(config, config_from_json);

    Ok(())
}
```

### Ключевые аспекты маршрутизатора

- **`[Service { ... }]`** генерирует `services: Vec<Service>` с `Service` как независимой структурой.
- **Вложенный `[Rule { ... }]`** внутри Service генерирует ещё один `Vec<Rule>` со встроенной структурой Rule.
- **`enum Method { ... }`** определяет перечисление встроенно, со struct-подобными вариантами для различных методов маршрутизации.
- Вся конфигурация может быть загружена из JSON / сохранена в JSON.

---

## Конфигурация с перечислениями

Понимание того, что генерирует yuuka, помогает отлаживать и эффективно работать с библиотекой.

Когда вы пишете:

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

Макрос генерирует приблизительно следующее:

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

    // Вспомогательные макросы для auto!
    macro_rules! __auto_Root {
        (name $($tt:tt)*) => { $($tt)* };
        (child { $($tt:tt)* }) => { ::yuuka::auto!(Child { $($tt)* }) };
        (child $($tt:tt)*) => { $($tt)* };
        // ... дополнительные правила для каждого поля
    }

    macro_rules! __auto_Child {
        (value $($tt:tt)*) => { $($tt)* };
        // ... дополнительные правила для каждого поля
    }
}
pub use __Root::*;
```

---

## Структура сгенерированного кода

### Ключевые аспекты конфигурации

1. **Обёртка в модуль**: Все типы помещаются в модуль с именем `__ИмяТипа` для избежания конфликтов имён. Всё реэкспортируется с помощью `use __ИмяТипа::*`.

2. **Автоматические derive**: `Debug` и `Clone` добавляются всегда. Ваши пользовательские `#[derive(...)]` макросы добавляются к ним.

3. **Реализация Default**: Если ни одно поле не имеет пользовательских значений по умолчанию → `#[derive(Default)]`. Если хотя бы одно поле имеет `= значение` → ручная `impl Default { ... }`.

4. **Вспомогательные макросы**: Для каждого типа генерируется макрос `__auto_ИмяТипа!`. Это макросы `macro_rules!`, которые процедурный макрос `auto!` вызывает для разрешения типов полей — в особенности имён анонимных структур/перечислений.

5. **Импорт super**: `use super::*` вносит внешнюю область видимости в модуль, именно поэтому внешние типы требуют префикса `super::` при обращении.

### Соглашение об именовании модулей

| Входные данные | Имя модуля |
| --- | --- |
| `Root { ... }` | `__Root` |
| `Config { ... }` | `__Config` |
| Анонимное поле в Root | `_Root_0_anonymous`, `_Root_1_anonymous`, ... |
| Анонимное поле в enum A | `_A_0_anonymous`, `_A_1_anonymous`, ... |
