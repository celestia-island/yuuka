# `derive_struct!` 巨集

`derive_struct!` 巨集是 yuuka 的核心。它允許你使用簡潔的、類似 JSON 的 DSL 定義複雜的巢狀結構體層次結構。所有內聯類型會自動提取為獨立的頂層 struct/enum 定義。

## 基本語法

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

生成的結構體自動帶有 `#[derive(Debug, Clone, Default)]`：

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## 巢狀結構體

透過 `TypeName { ... }` 語法直接在父結構體內定義內聯子結構體：

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

這會生成**三個**獨立的結構體：`Root`、`Info` 和 `Detail`。每個都是標準的 Rust 結構體，所有欄位在生成的模組內公開。

可以巢狀到任意深度：

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

## 匿名結構體

省略類型名稱以建立自動命名的結構體：

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

匿名結構體會被自動命名為 `_Root_0_anonymous`。多個匿名類型按順序編號：

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
// 生成：_Root_0_anonymous（對應 a）、_Root_1_anonymous（對應 c）
```

匿名結構體可以深度巢狀：

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

> **提示**：使用 [`auto!`](./auto-macro.md) 巨集可以在不知道生成名稱的情況下建構匿名結構體實例。

---

## 陣列（Vec）類型

使用 `[Type { ... }]` 語法定義 `Vec<Type>` 欄位：

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// 生成欄位：items: Vec<Item>
```

### 匿名陣列元素

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

### 列舉陣列

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

### 匿名列舉陣列

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

## 可選（Option）類型

在欄位名後追加 `?` 以將類型包裹為 `Option<T>`：

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

### Option 與內聯結構體

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

### Option 與匿名結構體

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

### Option 與內聯列舉

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

### 列舉變體內的 Option

`?` 語法同樣適用於列舉結構體變體內部：

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

## 預設值

使用 `=` 在類型後指定預設值：

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

### 行為規則

- **有** `= value` 的欄位在生成的 `impl Default` 中使用該值。
- **沒有** `= value` 的欄位使用 `Default::default()`（如數字為 `0`，String 為 `""`，bool 為 `false`）。
- 如果**任何**欄位有自訂預設值，巨集會生成手動的 `impl Default` 區塊，而不是 `#[derive(Default)]`。

### 陣列的預設值

```rust
derive_struct!(
    Root {
        // 預設為空
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Vec 預設為空
```

帶有明確陣列預設值：

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

// 新增的元素使用 Item 級別的預設值
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### 列舉的預設值

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

列舉陣列的預設值：

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
assert_eq!(root.members[0], Member::Arisu); // 來自 Vec 預設值
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // 來自列舉預設值
```

---

## 內聯列舉

在結構體欄位中內聯定義列舉：

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

### 變體形式

列舉變體支援三種形式：

**單元變體** — 無關聯資料：

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**結構體變體** — 具名欄位，其中可以包含內聯結構體和列舉：

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

**元組變體** — 位置資料，支援內聯結構體、多欄位和靜態類型：

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### 巢狀列舉

列舉可以巢狀在列舉變體內：

```rust
// 結構體變體中的列舉
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

// 元組變體中的列舉
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

### 變體中的列舉陣列

```rust
// 結構體變體中的 Vec<enum>
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

// 元組變體中的 Vec<enum>
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

## 參考類型

使用路徑語法參考外部已定義的類型：

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

> **注意**：由於生成的類型位於模組（`__Root`）內部，通常需要使用 `super::` 來參考外部作用域的類型。具體路徑取決於外部類型相對於巨集呼叫位置的定義位置。

欄位名命名靈活 — snake_case 和 PascalCase 名稱均可使用：

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
