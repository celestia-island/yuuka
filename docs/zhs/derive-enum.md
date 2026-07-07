# `derive_enum!` 宏

`derive_enum!` 宏用于定义独立的枚举类型，使用与 `derive_struct!` 相同的 DSL 语法风格。支持三种变体形式、嵌套类型和默认值。

## 基本语法

```rust
use yuuka::derive_enum;

derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);
```

生成的枚举自动带有 `#[derive(Debug, Clone)]`。

---

## 变体形式

### 单元变体

无关联数据的简单变体：

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

### 结构体变体

带命名字段的变体，字段可以使用内联结构体定义：

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
// 在 Action 枚举旁边生成独立的 Target 结构体
```

### 元组变体

带位置数据的变体，可包含内联结构体、枚举和静态类型：

```rust
derive_enum!(
    enum Message {
        Text(String),
        Data(Payload { content: Vec<u8>, size: usize }),
        Multi(String, i32, bool),
    }
);
```

### 混合变体

三种形式可以在同一个枚举中共存：

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

## 嵌套枚举

枚举变体可以包含其他内联枚举：

### 元组变体中的嵌套

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

### 匿名嵌套枚举

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

匿名枚举会被命名为 `_Root_0_anonymous`。可以通过模块引用：

```rust
let _ = Root::D(__Root::_Root_0_anonymous::E);
```

> **提示**：使用 [`auto!`](./auto-macro.md) 可以避免处理匿名名称。`auto!(Root::D::E)` 会自动解析路径。

### 深度嵌套的匿名枚举

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

// 手动构造：
let _ = A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))));

// 使用 auto!：
use yuuka::auto;
let _ = auto!(A::B::C::D::E::F);
let _ = auto!(A::B::C::D::E::G("hello".to_string()));
```

---

## 默认值

在闭合花括号后使用 `= VariantName` 指定默认变体：

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

### 元组变体的默认值

```rust
derive_enum!(
    enum Value {
        Int(i32),
        Text(String),
    } = Int(0)
);
```

### 嵌套匿名枚举的默认值

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

> **注意**：未指定默认值时，生成的 `impl Default` 使用 `unimplemented!()`，运行时调用会 panic。如果计划使用 `Default::default()`，请务必指定默认值。

---

## 额外的 Derive 和属性宏

与 `derive_struct!` 相同，可以传递 `#[derive(...)]` 和属性宏：

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

有关属性宏、递归传播和变体级属性的完整详情，请参阅[属性与可见性](./attributes.md)。
