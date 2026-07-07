# 屬性與進階特性

本頁涵蓋 yuuka 巨集的進階特性：額外 derive、屬性巨集傳播、可見性控制、跨 crate 使用等。

## 額外 derive 巨集

在巨集呼叫最前方追加 `#[derive(...)]` 來為**所有**生成的類型新增 derive：

```rust
derive_struct!(
    #[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    Root {
        name: String,
        info: Info {
            detail: String,
        },
    }
);
```

`Root` 和 `Info` 都會帶有 `#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]`。

---

## 屬性巨集

也可以在頂層追加非 derive 屬性：

```rust
derive_struct!(
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    Root {
        user_name: String,
        item_count: u32,
    }
);
```

頂層屬性會套用到**所有**生成的類型上。

---

## `#[macros_recursive]` — 遞迴傳播

`#[macros_recursive(...)]` 使額外的 derive 和屬性遞迴套用到內部所有巢狀類型：

```rust
derive_struct!(
    #[macros_recursive(
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
    )]
    Root {
        user_name: String,
        nested: Nested {
            inner_value: String,
            deep: Deep {
                deep_field: i32,
            },
        },
    }
);
```

`Root`、`Nested` 和 `Deep` 都會帶有指定的 derive 和屬性。

### 與頂層 derive 結合

```rust
derive_struct!(
    #[derive(PartialEq)]
    #[macros_recursive(
        #[derive(serde::Serialize)]
    )]
    Root {
        data: Data {
            value: String,
        },
    }
);
```

- `Root`：`Debug, Clone, Default, PartialEq, Serialize`
- `Data`：`Debug, Clone, Default, Serialize`（`PartialEq` 不遞迴，`Serialize` 會遞迴）

---

## 欄位級屬性

在個別欄位上追加屬性：

```rust
derive_struct!(
    #[derive(serde::Serialize, serde::Deserialize)]
    Root {
        #[serde(rename = "userName")]
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        email?: String,
    }
);
```

---

## 內聯類型上的類型級屬性

### 具名內聯類型

對具名巢狀類型，使用 `#[derive]` 作為分隔符號：

```rust
derive_struct!(
    #[derive(serde::Serialize)]
    Root {
        info: Info {
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            #[derive]
            name: String,
            value: i32,
        },
    }
);
```

語法規則：

1. `#[derive]` 之前的屬性套用到 **類型本身**（`Info`）
2. `#[derive]` 之後的欄位是 **`Info` 的欄位**
3. 空的 `#[derive]` 作為分隔符號，本身不追加任何 derive

在上例中，`Info` 會額外帶有 `#[derive(serde::Deserialize)]` 和 `#[serde(deny_unknown_fields)]`。

### 匿名內聯類型

匿名類型同樣使用此語法：

```rust
derive_struct!(
    Root {
        data: {
            #[derive(PartialEq)]
            #[derive]
            value: String,
            count: i32,
        },
    }
);
```

生成的 `_Root_0_anonymous` 將額外帶有 `#[derive(PartialEq)]`。

---

## 變體級屬性

列舉變體上也可以追加屬性：

```rust
derive_struct!(
    #[derive(serde::Serialize)]
    Root {
        status: enum Status {
            #[serde(rename = "active")]
            Active,
            #[serde(rename = "inactive")]
            Inactive,
        },
    }
);
```

---

## 可見性

### 預設可見性

生成的類型預設為 `pub(crate)`，內部欄位為 `pub`：

```rust
derive_struct!(
    Root {
        name: String,
    }
);
// 生成：pub(crate) struct Root { pub name: String }
```

### pub 修飾

使用 `pub` 讓類型對外可見：

```rust
derive_struct!(
    pub Root {
        name: String,
    }
);
// 生成：pub struct Root { pub name: String }
```

### 巢狀類型的可見性

`pub` 只影響巨集呼叫直接指定的類型。巢狀內聯類型始終為 `pub(crate)`：

```rust
derive_struct!(
    pub Root {
        inner: Inner {
            value: String,
        },
    }
);
// Root: pub struct
// Inner: pub(crate) struct
```

如需巢狀類型也公開，需分別定義並使用參考。

---

## 跨 crate 使用

### `#[macro_export]`

使用 `#[macro_export]` 讓生成的類型和 `auto!` 巨集可跨 crate 使用：

```rust
// 在 library crate 中
use yuuka::derive_struct;

derive_struct!(
    #[macro_export]
    pub Config {
        host: String = "localhost".to_string(),
        port: u16 = 8080,
        data: {
            value: String,
        },
    }
);
```

這會：

1. 為每個生成的類型產生輔助巨集
2. 將這些巨集匯出到 crate 根（透過 `#[macro_export]`）
3. 允許外部 crate 的 `auto!` 巨集存取這些類型

### 在依賴端使用

```rust
// 在使用端 crate 中
use my_library::Config;

fn main() {
    let cfg = Config::default();
    
    // auto! 可用於匿名類型
    let data = auto!(Config.data {
        value: "hello".to_string(),
    });
}
```

### 完整範例

請參閱專案中的 `tests/across_crate_lib/` 和 `tests/across_crate_entry/` 目錄，其中包含了完整的跨 crate 使用範例。

---

## 完整屬性範例

綜合運用所有屬性功能：

```rust
derive_struct!(
    #[derive(PartialEq, serde::Serialize, serde::Deserialize)]
    #[macros_recursive(
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
    )]
    #[macro_export]
    pub AppConfig {
        #[serde(rename = "applicationName")]
        app_name: String = "MyApp".to_string(),
        
        server: ServerConfig {
            #[derive]
            host: String = "0.0.0.0".to_string(),
            port: u16 = 3000,
        },
        
        log_level: enum LogLevel {
            #[serde(rename = "debug")]
            Debug,
            #[serde(rename = "info")]
            Info,
            #[serde(rename = "error")]
            Error,
        } = Info,
        
        plugins?: [{
            name: String,
            enabled: bool = true,
        }],
    }
);
```

此範例展示了：

- 額外 derive（`PartialEq`、serde）
- 遞迴屬性傳播（`rename_all`）
- 跨 crate 匯出（`#[macro_export]`）
- 公開可見性（`pub`）
- 欄位屬性（`#[serde(rename)]`）
- 類型級屬性分隔（`#[derive]`）
- 預設值
- 內聯列舉帶預設變體
- 可選匿名陣列類型
