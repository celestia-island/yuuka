# 属性と可視性

このドキュメントでは、`derive_struct!` と `derive_enum!` で生成される型に対する derive マクロ、属性マクロ、可視性、クレート間エクスポートの制御方法を説明します。

---

## 追加 derive マクロ

型名の前に `#[derive(...)]` を配置して、生成されるルート型に derive マクロを追加します：

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

> **注意**: `Debug` と `Clone` は常に自動的に derive されます。明示的に指定する必要はありません。

`derive_enum!` でも同様に使えます：

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

## 属性マクロ

属性マクロは `#[derive(...)]` の後に配置します：

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

## `#[macros_recursive]` による再帰的属性伝播

`#[macros_recursive(...)]` を使って、**すべて** のネストされたインライン型に属性を伝播させます：

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
// すべてのネストレベルでcamelCaseが適用: "nickName", "simplifiedChinese", "firstName" など
```

`#[macros_recursive(...)]` は、階層内で生成されるすべての構造体と列挙型に指定された属性を適用します — ルート型だけではありません。

---

## フィールドレベル属性

フィールド名の直前に属性を配置します：

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

// "live_in" は "liveIn" ではなく "location" としてシリアライズされる
```

### バリアントレベル属性

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

## 型レベル属性（インライン型への適用）

フィールドで定義されたインライン構造体/列挙型に `#[derive(...)]` と属性を適用できます。**フィールド名の前** に配置し、`#[derive(...)]` をフィールド属性と型属性の区切りとして使います：

### 名前付きインライン型

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

// Root には #[serde(deny_unknown_fields)] が適用
// Location には #[derive(PartialEq)] と #[serde(rename_all = "UPPERCASE")] が適用
// "location" フィールドは "position" にリネーム
```

### 匿名インライン型への `#[derive]` セパレータ

匿名型の場合は、空の `#[derive]` をセパレータとして使います：

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

// 空の #[derive] がフィールドレベル属性（上）と型レベル属性（下）を分離
```

### 列挙型バリアントへの適用

同じパターンが列挙型のタプルバリアントでも使えます：

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

匿名列挙型バリアントの場合も同様です：

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

## 可視性

### `pub` 修飾子

`pub` を使って生成される型とモジュールをパブリックにします：

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

これにより `pub mod __Root` と `pub use __Root::*` が生成され、すべての型が現在のモジュール外からアクセス可能になります。

### デフォルトの可視性

`pub` なしの場合、型は `pub(crate)` になります：

```rust
derive_struct!(
    Root {
        name: String,
    }
);
// 生成結果: pub(crate) mod __Root { ... }
// 生成結果: pub(crate) use __Root::*;
```

> **注意**: `pub` 宣言は通常、モジュールやクレートレベル（関数の外）で使用されます。テスト関数内では可視性は関係ありません。

---

## クレート間の使用

生成された型と `auto!` ヘルパーマクロを他のクレートで使用するためにエクスポートするには、`#[macro_export]` を使います：

### ライブラリクレート

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

> **注意**: `#[macro_export]` は `#[derive(...)]` の前後どちらにも配置できます — どちらの位置でも動作します。

### 利用側クレート

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

### 仕組み

`#[macro_export]` は、生成された `macro_rules!` ヘルパーマクロ（`__auto_TestStruct!` など）をクレートルートレベルで使えるようにします。この属性がなければ、ヘルパーマクロは定義クレート内でのみ見え、外部クレートからは `auto!` が機能しません。

### Cargo.toml の設定

ライブラリクレートが適切にリンクできるようにします：

```toml
[lib]
crate-type = ["rlib", "dylib"]
```

### 完全な例

以下は、ライブラリクレートでの型定義からそのエクスポート、利用側クレートでの使用までの完全なフローです：

```rust
// ライブラリクレート (my_lib)
use yuuka::{derive_struct, derive_enum};

derive_struct!(
    #[derive(PartialEq)]
    #[macro_export]
    pub Config {
        name: String,
        settings: {
            debug: bool = false,
            level: u32 = 1,
        },
    }
);

// 利用側クレート
use yuuka::auto;
use my_lib::*;

let cfg = auto!(Config {
    name: "app".to_string(),
    settings: {
        debug: true,
        ..Default::default()
    },
});
assert_eq!(cfg.settings.level, 1); // デフォルト値
```

`#[macro_export]`、`pub` 可視性、`auto!` を組み合わせることで、クレート間でネスト型の定義と構築がシームレスに行えます。
