# `derive_struct!` 宏

`derive_struct!` 宏是 yuuka 的核心。它允许你使用简洁的、类似 JSON 的 DSL 定义复杂的嵌套结构体层次结构。所有内联类型会自动提取为独立的顶层 struct/enum 定义。

## 基本语法

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

生成的结构体自动带有 `#[derive(Debug, Clone, Default)]`：

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## 嵌套结构体

通过 `TypeName { ... }` 语法直接在父结构体内定义内联子结构体：

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

这会生成**三个**独立的结构体：`Root`、`Info` 和 `Detail`。每个都是标准的 Rust 结构体，所有字段在生成的模块内公开。

可以嵌套到任意深度：

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

## 匿名结构体

省略类型名称以创建自动命名的结构体：

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

匿名结构体会被自动命名为 `_Root_0_anonymous`。多个匿名类型按顺序编号：

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
// 生成：_Root_0_anonymous（对应 a）、_Root_1_anonymous（对应 c）
```

匿名结构体可以深度嵌套：

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

> **提示**：使用 [`auto!`](./auto-macro.md) 宏可以在不知道生成名称的情况下构造匿名结构体实例。

---

## 数组（Vec）类型

使用 `[Type { ... }]` 语法定义 `Vec<Type>` 字段：

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// 生成字段：items: Vec<Item>
```

### 匿名数组元素

```rust
derive_struct!(
    Root {
        items: [{
            name: String,
        }],
    }
);
// 生成：items: Vec<_Root_0_anonymous>
```

### 枚举数组

```rust
derive_struct!(
    Root {
        statuses: [enum Status {
            Active,
            Inactive,
        }],
    }
);
// 生成：statuses: Vec<Status>
```

### 匿名枚举数组

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
// 生成：values: Vec<_Root_0_anonymous>
```

---

## 可选（Option）类型

在字段名后追加 `?` 以将类型包裹为 `Option<T>`：

```rust
derive_struct!(
    Root {
        required: String,
        optional?: String,
    }
);
// 生成：
//   required: String,
//   optional: Option<String>,
```

### Option 与内联结构体

```rust
derive_struct!(
    Root {
        detail?: Detail {
            info: String,
        },
    }
);
// 生成：detail: Option<Detail>
```

### Option 与匿名结构体

```rust
derive_struct!(
    Root {
        data?: {
            value: String,
        },
    }
);
// 生成：data: Option<_Root_0_anonymous>
```

### Option 与内联枚举

```rust
derive_struct!(
    Root {
        status?: enum Status {
            Active,
            Inactive,
        },
    }
);
// 生成：status: Option<Status>
```

### 枚举变体内的 Option

`?` 语法同样适用于枚举结构体变体内部：

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

## 默认值

使用 `=` 在类型后指定默认值：

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

### 行为规则

- **有** `= value` 的字段在生成的 `impl Default` 中使用该值。
- **没有** `= value` 的字段使用 `Default::default()`（如数字为 `0`，String 为 `""`，bool 为 `false`）。
- 如果**任何**字段有自定义默认值，宏会生成手动的 `impl Default` 块，而不是 `#[derive(Default)]`。

### 数组的默认值

```rust
derive_struct!(
    Root {
        // 默认为空
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Vec 默认为空
```

带有显式数组默认值：

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

// 新添加的元素使用 Item 级别的默认值
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### 枚举的默认值

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

枚举数组的默认值：

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
assert_eq!(root.members[0], Member::Arisu); // 来自 Vec 默认值
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // 来自枚举默认值
```

---

## 内联枚举

在结构体字段中内联定义枚举：

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

### 变体形式

枚举变体支持三种形式：

**单元变体** — 无关联数据：

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**结构体变体** — 命名字段，其中可以包含内联结构体和枚举：

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

**元组变体** — 位置数据，支持内联结构体、多字段和静态类型：

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### 嵌套枚举

枚举可以嵌套在枚举变体内：

```rust
// 结构体变体中的枚举
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

// 元组变体中的枚举
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

### 变体中的枚举数组

```rust
// 结构体变体中的 Vec<enum>
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

// 元组变体中的 Vec<enum>
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

## 引用类型

使用路径语法引用外部已定义的类型：

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

> **注意**：由于生成的类型位于模块（`__Root`）内部，通常需要使用 `super::` 来引用外部作用域的类型。具体路径取决于外部类型相对于宏调用位置的定义位置。

字段名命名灵活 — snake_case 和 PascalCase 名称均可使用：

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
