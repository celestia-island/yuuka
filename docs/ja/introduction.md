# Yuuka - はじめに

**Yuuka** は、簡潔な JSON 風 DSL 構文を使って、複雑で深くネストされた構造体と列挙型の階層を定義できる Rust 手続きマクロライブラリです。シームレスなシリアライズ・デシリアライズのために `serde` をベースに構築されています。

## インストール

`Cargo.toml` に以下を追加してください：

```toml
[dependencies]
yuuka = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> `serde` と `serde_json` は任意ですが、シリアライズサポートのために yuuka と併用されることが多いです。

## コアマクロ

Yuuka は 3 つの手続きマクロをエクスポートします：

| マクロ | 用途 |
| --- | --- |
| [`derive_struct!`](./derive-struct.md) | JSON 風 DSL でネスト構造体の階層を定義する |
| [`derive_enum!`](./derive-enum.md) | さまざまなバリアント形式を持つ列挙型を定義する |
| [`auto!`](./auto-macro.md) | 上記マクロで生成された型のインスタンスを簡略構文で構築する |

関連ドキュメント：

- [属性と可視性](./attributes.md) — 追加 derive マクロ、属性の伝播、可視性制御、クレート間の使用
- [使用例](./examples.md) — 実用的な使用例と生成コードの構造

## クイックスタート

```rust
use serde::{Serialize, Deserialize};
use yuuka::{derive_struct, auto};

derive_struct!(
    #[derive(PartialEq, Serialize, Deserialize)]
    GameConfig {
        title: String,
        window: Window {
            width: u32,
            height: u32,
            fullscreen: bool,
        },
        plugins: [Plugin {
            name: String,
            enabled: bool,
        }],
    }
);

let config = auto!(GameConfig {
    title: "My Game".to_string(),
    window: {
        width: 1920,
        height: 1080,
        fullscreen: true,
    },
    plugins: vec![
        Plugin {
            name: "Audio".to_string(),
            enabled: true,
        },
    ],
});
```

この 1 回の `derive_struct!` 呼び出しで、`GameConfig`、`Window`、`Plugin` の 3 つの独立した構造体が自動生成されます。いずれも `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` が付与されます。`auto!` マクロを使えば、匿名型やインラインサブ構造体の生成名を知らなくても、`{ }` ブロックでインスタンスを構築できます。

## ドキュメント一覧

| ドキュメント | 説明 |
| --- | --- |
| [derive_struct!](./derive-struct.md) | 構造体定義マクロ — ネスト構造体、匿名構造体、Vec/Option 型、デフォルト値、インライン列挙型、参照型 |
| [derive_enum!](./derive-enum.md) | 列挙型定義マクロ — ユニット/構造体/タプルバリアント、ネスト列挙型、デフォルト値 |
| [auto!](./auto-macro.md) | インスタンス構築マクロ — 匿名型の簡略構文、列挙型パス、スプレッド式 |
| [属性と可視性](./attributes.md) | derive マクロ、属性の伝播、`#[macros_recursive]`、フィールドレベル属性、可視性、`#[macro_export]`、クレート間の使用 |
| [使用例](./examples.md) | 実用的な使用例、生成コードの構造の解説 |
