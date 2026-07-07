# 属性与可见性

本文档介绍如何控制由 `derive_struct!` 和 `derive_enum!` 生成的类型的 derive 宏、属性宏、可见性和跨 crate 导出。

---

## 额外的 Derive 宏

在类型名称前放置 `#[derive(...)]` 以为生成的根类型添加 derive 宏：

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

> **注意**：`Debug` 和 `Clone` 始终自动派生，无需手动指定。

`derive_enum!` 同样如此：

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

## 属性宏

将属性宏放在 `#[derive(...)]` 之后：

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

## 递归属性传播

使用 `#[macros_recursive(...)]` 将属性传播到**所有**嵌套的内联类型：

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
// 所有嵌套层级都使用 camelCase："nickName"、"simplifiedChinese"、"firstName" 等
```

`#[macros_recursive(...)]` 将指定属性应用于层次结构中生成的每个 struct 和 enum — 不仅仅是根类型。

---

## 字段级属性

直接在字段名前放置属性：

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

// "live_in" 序列化为 "location" 而不是 "liveIn"
```

### 枚举变体级属性

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

## 内联类型的类型级属性

可以为字段中定义的内联 struct/enum 类型应用 `#[derive(...)]` 和属性。将它们放在**字段名之前**，使用 `#[derive(...)]` 分隔字段属性和类型属性：

### 具名内联类型

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

// Root 获得 #[serde(deny_unknown_fields)]
// Location 获得 #[derive(PartialEq)] 和 #[serde(rename_all = "UPPERCASE")]
// 字段 "location" 被重命名为 "position"
```

### 匿名内联类型

对于匿名类型，使用 `#[derive]`（空 derive）作为分隔符：

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

// 空的 #[derive] 分隔了上方的字段级属性和下方的类型级属性
```

### 枚举变体上的属性

同样的模式适用于枚举元组变体：

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

以及匿名枚举变体：

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

## 可见性

### `pub` 修饰符

使用 `pub` 使生成的类型及其模块公开：

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

这会生成 `pub mod __Root` 和 `pub use __Root::*`，使所有类型从当前模块外部可访问。

### 默认可见性

不使用 `pub` 时，类型为 `pub(crate)`：

```rust
derive_struct!(
    Root {
        name: String,
    }
);
// 生成：pub(crate) mod __Root { ... }
// 生成：pub(crate) use __Root::*;
```

> **注意**：`pub` 声明通常在模块或 crate 级别（函数外部）使用。在测试函数内部，可见性并不重要。

---

## 跨 Crate 使用

要导出生成的类型及其 `auto!` 辅助宏以在其他 crate 中使用，使用 `#[macro_export]`：

### 库 Crate

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

> **注意**：`#[macro_export]` 可以放在 `#[derive(...)]` 之前或之后 — 两种位置都有效。

### 使用方 Crate

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

### 工作原理

`#[macro_export]` 使生成的 `macro_rules!` 辅助宏（如 `__auto_TestStruct!`）在 crate 根级别可用。没有此属性，辅助宏仅在定义 crate 内可见，`auto!` 无法从外部 crate 工作。

### Cargo.toml 配置

对于库 crate，确保可以正确链接：

```toml
[lib]
crate-type = ["rlib", "dylib"]
```
