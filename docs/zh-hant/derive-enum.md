# `derive_enum!` 巨集

`derive_enum!` 巨集用於獨立定義列舉類型。與在 `derive_struct!` 欄位內行定義列舉不同，`derive_enum!` 讓你以頂層方式建立列舉，支援完整的變體形式和巢狀能力。

## 基本語法

```rust
use yuuka::derive_enum;

derive_enum!(
    Status {
        Active,
        Inactive,
        Pending,
    }
);
```

生成的列舉自動帶有 `#[derive(Debug, Clone, Default)]`。第一個變體作為預設值（除非另行指定）。

---

## 變體形式

`derive_enum!` 支援所有三種 Rust 列舉變體形式。

### 單元變體

無關聯資料的簡單標籤：

```rust
derive_enum!(
    Color {
        Red,
        Green,
        Blue,
    }
);
```

### 結構體變體

帶有具名欄位的變體：

```rust
derive_enum!(
    Shape {
        Circle { radius: f64 },
        Rectangle { width: f64, height: f64 },
    }
);
```

結構體變體中的欄位可以包含內聯結構體定義：

```rust
derive_enum!(
    Event {
        Click {
            position: Position {
                x: f64,
                y: f64,
            },
        },
        KeyPress { key: String },
    }
);
```

### 元組變體

帶有位置參數的變體：

```rust
derive_enum!(
    Value {
        Integer(i64),
        Float(f64),
        Text(String),
    }
);
```

元組變體同樣可以包含內聯結構體：

```rust
derive_enum!(
    Wrapper {
        Data(Payload { content: String }),
        Empty,
    }
);
```

### 混合變體形式

可以在同一個列舉中自由混合不同形式：

```rust
derive_enum!(
    Action {
        None,
        Simple(String),
        Complex { name: String, detail: String },
    }
);
```

---

## 巢狀列舉

可以在列舉的結構體或元組變體中巢狀其他列舉：

### 結構體變體中巢狀

```rust
derive_enum!(
    Outer {
        Variant {
            inner: enum Inner {
                A,
                B,
                C,
            },
        },
    }
);

let val = Outer::Variant { inner: Inner::B };
```

### 元組變體中巢狀

```rust
derive_enum!(
    Outer {
        Wrapped(enum Inner {
            X,
            Y,
        }),
    }
);

let val = Outer::Wrapped(Inner::X);
```

### 深層巢狀

列舉可以多層巢狀，包含匿名列舉：

```rust
derive_enum!(
    Root {
        Branch {
            category: enum Category {
                TypeA {
                    detail: enum Detail {
                        High,
                        Low,
                    },
                },
                TypeB,
            },
        },
    }
);
```

---

## 預設值

使用 `= VariantName` 在列舉定義末尾指定預設值：

```rust
derive_enum!(
    #[derive(PartialEq)]
    Priority {
        Low,
        Medium,
        High,
    } = Medium
);

let p = Priority::default();
assert_eq!(p, Priority::Medium);
```

如果未指定預設變體，則使用 `#[derive(Default)]` 的標準行為（第一個變體作為預設值，需要型別支援）。

---

## 搭配額外 derive 和屬性

可以在列舉上追加額外屬性和 derive 巨集：

```rust
derive_enum!(
    #[derive(PartialEq, Eq, Hash)]
    #[repr(u8)]
    Status {
        Active,
        Inactive,
    }
);
```

這些屬性會追加到自動生成的 `#[derive(Debug, Clone, Default)]` 之後。

---

## 列舉陣列中巢狀列舉

在結構體或元組變體中可以使用 `[enum {...}]` 定義列舉元素的陣列：

```rust
derive_enum!(
    Container {
        Items {
            tags: [enum Tag {
                Important,
                Normal,
            }],
        },
    }
);
// tags: Vec<Tag>
```

---

## 與 `derive_struct!` 的互動

`derive_enum!` 生成的列舉可以在 `derive_struct!` 中透過參考路徑使用：

```rust
derive_enum!(
    Status {
        Active,
        Inactive,
    }
);

derive_struct!(
    Config {
        name: String,
        status: super::Status,
    }
);
```

> **注意**：由於 `derive_struct!` 內部會建立模組包裹生成的程式碼，若要參考外部類型需使用 `super::` 或適當的路徑。

另一個選擇是直接在 `derive_struct!` 欄位中使用行內 `enum` 語法，參閱 [derive_struct! - 內聯列舉](./derive-struct.md#內聯列舉)。
