# 使用例

yuuka マクロの実用的な使用例と、生成されるコードの構造の解説です。

---

## 言語パック (i18n)

典型的なユースケース：JSON ファイルに直接マッピングされるネストされた言語パック構造を定義します。serde によるシリアライズ・デシリアライズをサポートします。

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

    // JSONからデシリアライズ
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

### 言語パックのポイント

- フィールド名に**非 ASCII 文字**（漢字など）が使えます — Rust 識別子と JSON キーの両方として機能します。
- `auto!` マクロが匿名サブ構造体（主页、设置、网络配置）の構築をシームレスに処理します。
- 生成された型は serde 完全互換で、JSON のラウンドトリップシリアライズが可能です。

---

## サーバールーター設定

より複雑な例：ネストされた配列とインライン列挙型を持つリバースプロキシ / サーバールーター設定のモデリング。

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

    // この構造はJSON との間で直接変換可能
    let json = serde_json::to_string_pretty(&config)?;
    let config_from_json: Config = serde_json::from_str(&json)?;
    assert_eq!(config, config_from_json);

    Ok(())
}
```

### ルーター設定のポイント

- **`[Service { ... }]`** は `services: Vec<Service>` を生成し、`Service` は独立した構造体になります。
- **ネストされた `[Rule { ... }]`** は Service 内部で別の `Vec<Rule>` をインライン Rule 構造体とともに生成します。
- **`enum Method { ... }`** は列挙型をインラインで定義し、異なるルーティング方法に対応する構造体バリアントを持ちます。
- 設定全体を JSON から読み込み / JSON に保存できます。

---

## 列挙型を使ったアプリ設定

列挙型バリアントを活用した設定例：

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, derive_enum, auto};

derive_enum!(
    #[derive(PartialEq, Serialize, Deserialize)]
    enum LogLevel {
        Debug,
        Info,
        Warn,
        Error,
    } = Info
);

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    AppConfig {
        name: String = "my-app".to_string(),
        log_level: super::LogLevel = super::LogLevel::Info,
        database: Database {
            host: String = "localhost".to_string(),
            port: u16 = 5432,
            pool_size: u32 = 10,
        },
        features: [Feature {
            name: String,
            enabled: bool = true,
        }],
    }
);

let config = AppConfig::default();
assert_eq!(config.name, "my-app");
assert_eq!(config.database.port, 5432);
assert_eq!(config.database.pool_size, 10);
```

### アプリ設定のポイント

- `derive_enum!` と `derive_struct!` を組み合わせて、型安全な設定構造を構築できます。
- 外部型を参照するには `super::` プレフィックスが必要です（生成された型はモジュール内に配置されるため）。
- デフォルト値を活用することで、最小限の設定で合理的な初期値を持つ設定オブジェクトを作成できます。

---

## 生成コードの構造

yuuka が何を生成するかを理解することで、ライブラリを使ったデバッグや作業がより効果的になります。

以下のように記述した場合：

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

マクロはおおよそ以下のコードを生成します：

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

    // auto! 用のヘルパーマクロ
    macro_rules! __auto_Root {
        (name $($tt:tt)*) => { $($tt)* };
        (child { $($tt:tt)* }) => { ::yuuka::auto!(Child { $($tt)* }) };
        (child $($tt:tt)*) => { $($tt)* };
        // ... 各フィールドに対応するルール
    }

    macro_rules! __auto_Child {
        (value $($tt:tt)*) => { $($tt)* };
        // ... 各フィールドに対応するルール
    }
}
pub use __Root::*;
```

### 主要なポイント

1. **モジュールラッピング**: すべての型は名前衝突を避けるために `__TypeName` というモジュールに配置されます。すべて `use __TypeName::*` で再エクスポートされます。

2. **自動 derive**: `Debug` と `Clone` は常に追加されます。カスタム `#[derive(...)]` マクロが後に追加されます。

3. **Default 実装**: カスタムデフォルトのあるフィールドがない場合 → `#[derive(Default)]`。いずれかのフィールドに `= value` がある場合 → 手動の `impl Default { ... }`。

4. **ヘルパーマクロ**: 各型に対して `__auto_TypeName!` マクロが生成されます。これらは `macro_rules!` マクロで、`auto!` 手続きマクロがフィールドの型（特に匿名構造体/列挙型の名前）を解決するために呼び出します。

5. **super インポート**: `use super::*` が外部スコープをモジュるに取り込むため、外部型を参照するときは `super::` プレフィックスが必要な理由です。

### モジュール命名規則

| 入力 | モジュール名 |
| --- | --- |
| `Root { ... }` | `__Root` |
| `Config { ... }` | `__Config` |
| Root 内の匿名フィールド | `_Root_0_anonymous`, `_Root_1_anonymous`, ... |
| 列挙型 A 内の匿名フィールド | `_A_0_anonymous`, `_A_1_anonymous`, ... |
