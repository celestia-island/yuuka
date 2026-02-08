# `auto!` 巨集

`auto!` 巨集是 yuuka 提供的輔助巨集，讓你在不需要知道自動生成的類型名稱（如匿名結構體和匿名列舉的名稱）的情況下建構實例。它也可以用於具名類型的建構。

## 核心概念

當你用 `derive_struct!` 或 `derive_enum!` 建立匿名巢狀類型時，巨集會自動生成類似 `_Root_0_anonymous` 的名稱。`auto!` 巨集讓你透過路徑導航的方式建構這些類型，而不需要知道確切的名稱。

---

## 語法形式

`auto!` 共有 **6 種**呼叫形式：

### 形式 1：具名結構體建構

```rust
auto!(Root { field1: value1, field2: value2 })
```

等同於直接建構 `Root { field1: value1, field2: value2 }`。

### 形式 2：具名變體建構

```rust
auto!(Root::Variant)
auto!(Root::Variant { field: value })
auto!(Root::Variant(value))
```

等同於建構列舉的特定變體。

### 形式 3：匿名結構體建構（單層路徑）

```rust
auto!(Root.field_name { inner_field: value })
```

透過 `.field_name` 語法導航到 `Root` 中名為 `field_name` 的匿名欄位類型。巨集會自動解析出對應的 `_Root_N_anonymous` 名稱並用於建構。

### 形式 4：匿名列舉建構（單層路徑）

```rust
auto!(Root.field_name::Variant)
auto!(Root.field_name::Variant { field: value })
```

匿名列舉欄位的變體建構，透過 `Root.field_name` 找到匿名列舉類型，然後指定變體。

### 形式 5：深層匿名路徑建構（多層導航）

```rust
auto!(Root.field1.field2.field3 { inner: value })
```

透過多層 `.field` 導航，穿越多層巢狀匿名結構體。

### 形式 6：深層路徑列舉建構

```rust
auto!(Root.field1.field2::Variant)
```

多層匿名導航後建構列舉變體。

---

## 結構體建構

### 直接具名建構

```rust
derive_struct!(
    Root {
        name: String,
        count: i32,
    }
);

let r = auto!(Root {
    name: "test".to_string(),
    count: 42,
});
```

### 匿名結構體建構

```rust
derive_struct!(
    Root {
        data: {
            value: String,
            flag: bool,
        },
    }
);

// 不需要知道 _Root_0_anonymous 這個名稱
let d = auto!(Root.data {
    value: "hello".to_string(),
    flag: true,
});
```

### 深度巢狀匿名建構

```rust
derive_struct!(
    Root {
        a: {
            b: String,
            c: {
                d: f64 = std::f64::consts::PI,
            },
        },
    }
);

let c_instance = auto!(Root.a.c {
    d: 2.718,
});
```

---

## 列舉建構

### 具名列舉建構

```rust
derive_struct!(
    Root {
        member: enum Member {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        },
    }
);

let m = auto!(Root::Member::Momoi);
```

> 具名列舉也可以直接用 `Member::Momoi` 建構。

### 匿名列舉建構

```rust
derive_struct!(
    Root {
        status: enum {
            Active,
            Inactive,
        },
    }
);

// 透過欄位路徑存取匿名列舉
let s = auto!(Root.status::Active);
```

### 結構體變體建構

```rust
derive_struct!(
    Root {
        action: enum Action {
            Move { x: f64, y: f64 },
            Stop,
        },
    }
);

let a = auto!(Root::Action::Move { x: 1.0, y: 2.0 });
```

### 元組變體建構

```rust
derive_struct!(
    Root {
        wrapper: enum Wrapper {
            Value(i32),
            Empty,
        },
    }
);

let w = auto!(Root::Wrapper::Value(42));
```

---

## 混合使用

在同一段程式碼中混合使用不同形式：

```rust
derive_struct!(
    App {
        config: {
            debug: bool = false,
            log_level: String = "info".to_string(),
        },
        mode: enum Mode {
            Development,
            Production,
        },
        items: [{
            name: String,
        }],
    }
);

// 建構匿名設定
let cfg = auto!(App.config {
    debug: true,
    log_level: "debug".to_string(),
});

// 建構列舉
let mode = auto!(App::Mode::Development);

// 建構匿名陣列元素
let item = auto!(App.items {
    name: "item1".to_string(),
});
```

---

## 跨模組使用

`auto!` 可搭配 `#[macro_export]` 匯出的巨集跨模組使用：

```rust
// 在 lib.rs 或其他模組中
use yuuka::derive_struct;

derive_struct!(
    #[macro_export]
    Config {
        data: {
            value: String,
        },
    }
);

// 在另一個 crate 中
let d = auto!(Config.data {
    value: "cross-crate".to_string(),
});
```

詳見 [屬性與特性 - 跨 crate 使用](./attributes.md#跨-crate-使用)。
