# 實戰範例

本頁展示 yuuka 在真實情境中的使用方式，包含完整的程式碼範例和生成結果說明。

---

## 語言包（i18n）

這個範例展示了如何使用 yuuka 定義多層結構化的國際化語言包。

```rust
use yuuka::derive_struct;

derive_struct!(
    #[macro_export]
    pub LanguagePack {
        display_name: String,
        general: GeneralLanguagePack {
            ok: String = "OK".to_string(),
            cancel: String = "Cancel".to_string(),
            confirm: String = "Confirm".to_string(),
        },
        error: ErrorLanguagePack {
            not_found: String = "Not Found".to_string(),
            unknown: String = "Unknown Error".to_string(),
        },
    }
);

fn main() {
    let pack = LanguagePack {
        display_name: "English".to_string(),
        ..Default::default()
    };
    
    println!("{}", pack.general.ok);       // "OK"
    println!("{}", pack.error.not_found);   // "Not Found"
}
```

### 語言包設計重點

- **預設值**：每個字串都有合理的英文預設值。透過 `..Default::default()` 語法即可使用。
- **`#[macro_export]`**：語言包可跨 crate 使用，其他模組可以透過 `auto!` 建構。
- **`pub` 可見性**：對外公開，讓依賴方 crate 可以直接參考類型。
- **巢狀結構**：按功能領域分組（general、error），清晰易維護。

---

## 伺服器路由設定

多層巢狀配置的範例，展示結構體巢狀搭配預設值：

```rust
use yuuka::derive_struct;

derive_struct!(
    #[macro_export]
    pub ServerRouter {
        ssl: SSL {
            cert_path: String,
            key_path: String,
        },
        router: Router {
            prefix: String = "/api".to_string(),
            timeout: u64 = 30,
            cors: Cors {
                enabled: bool = true,
                origin: String = "*".to_string(),
            },
        },
    }
);

fn main() {
    let router = ServerRouter {
        ssl: SSL {
            cert_path: "/path/to/cert.pem".to_string(),
            key_path: "/path/to/key.pem".to_string(),
        },
        ..Default::default()
    };

    println!("Prefix: {}", router.router.prefix);        // "/api"
    println!("CORS enabled: {}", router.router.cors.enabled); // true
}
```

### 路由設計重點

- **混合必填/選填**：`ssl` 的路徑無預設值（必須明確提供），`router` 有完整的合理預設。
- **三層巢狀**：`ServerRouter` → `Router` → `Cors`，yuuka 自動為每層生成獨立結構體。
- **設定繼承模式**：使用 `..Default::default()` 只需覆寫部分欄位。

---

## 帶列舉的應用設定

結合列舉和巢狀結構體的完整範例：

```rust
use yuuka::{derive_struct, auto};

derive_struct!(
    #[derive(PartialEq)]
    #[macro_export]
    pub AppConfig {
        name: String = "MyApp".to_string(),
        
        mode: enum RunMode {
            Development,
            Staging,
            Production,
        } = Development,
        
        database: Database {
            host: String = "localhost".to_string(),
            port: u16 = 5432,
            pool_size: u32 = 10,
        },
        
        features: [{
            name: String,
            enabled: bool = true,
        }],
    }
);

fn main() {
    let config = AppConfig {
        mode: auto!(AppConfig::RunMode::Production),
        database: Database {
            host: "db.example.com".to_string(),
            pool_size: 50,
            ..Default::default()
        },
        features: vec![
            auto!(AppConfig.features {
                name: "dark_mode".to_string(),
                enabled: true,
            }),
            auto!(AppConfig.features {
                name: "beta_ui".to_string(),
                enabled: false,
            }),
        ],
        ..Default::default()
    };

    assert_eq!(config.name, "MyApp");
    assert_eq!(config.mode, RunMode::Production);
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.features.len(), 2);
}
```

### 設定設計重點

- **列舉預設值**：`RunMode` 預設為 `Development`。
- **匿名陣列元素**：`features` 使用匿名結構體定義，透過 `auto!(AppConfig.features {...})` 建構。
- **混合建構方式**：具名類型直接建構，匿名類型用 `auto!`。
- **`#[derive(PartialEq)]`**：讓列舉可做相等比較。

---

## 生成的程式碼結構

理解 yuuka 生成的模組結構有助於除錯和進階使用。

### 命名約定

| 定義方式 | 生成的名稱 | 說明 |
| --- | --- | --- |
| `Root { ... }` | `Root` | 頂層結構體名稱直接使用 |
| `field: Info { ... }` | `Info` | 具名巢狀類型使用指定名稱 |
| `field: { ... }` | `_Root_0_anonymous` | 匿名類型，按出現順序編號 |
| `field: [{ ... }]` | `_Root_0_anonymous` | 匿名陣列元素類型 |
| `field: enum Status { ... }` | `Status` | 具名列舉使用指定名稱 |
| `field: enum { ... }` | `_Root_0_anonymous` | 匿名列舉 |

### 模組包裹

每次 `derive_struct!` 呼叫會建立一個隱藏模組 `__Root`（以根類型名稱为基礎），其中包含所有生成的類型，然後透過 `pub(crate) use __Root::*;` 匯出：

```rust
// derive_struct!(Root { info: Info { value: String } })
// 概念上生成：

mod __Root {
    #[derive(Debug, Clone, Default)]
    pub(crate) struct Root {
        pub info: Info,
    }
    
    #[derive(Debug, Clone, Default)]
    pub(crate) struct Info {
        pub value: String,
    }
}
pub(crate) use __Root::*;
```

### 對使用者的影響

- 所有生成的類型在同一作用域平攤可用。
- 參考外部類型時需使用 `super::` 或完整路徑。
- `auto!` 巨集隱藏了這些實作細節，推薦用它來建構匿名類型。
