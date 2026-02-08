# Yuuka - 簡介

**Yuuka** 是一個 Rust 過程巨集庫，允許你使用簡潔的、類似 JSON 的 DSL 語法來定義複雜且深度巢狀的 struct 和 enum 層次結構。它基於 `serde` 構建，可以無縫進行序列化和反序列化。

## 安裝

在 `Cargo.toml` 中加入以下相依套件：

```toml
[dependencies]
yuuka = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> `serde` 和 `serde_json` 是選用的，但通常與 yuuka 一起使用以支援序列化。

## 核心巨集

Yuuka 匯出三個過程巨集：

| 巨集 | 用途 |
| --- | --- |
| [`derive_struct!`](./derive-struct.md) | 使用類 JSON DSL 定義巢狀的 struct 層次結構 |
| [`derive_enum!`](./derive-enum.md) | 定義帶有多種變體形式的 enum 類型 |
| [`auto!`](./auto-macro.md) | 使用簡化語法建構由上述巨集生成的類型實例 |

另請參閱：

- [屬性與可見性](./attributes.md) — 額外的 derive 巨集、屬性傳播、可見性控制和跨 crate 使用
- [範例](./examples.md) — 真實場景範例和生成程式碼結構說明

## 快速開始

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

此次 `derive_struct!` 呼叫會自動生成三個獨立的結構體 — `GameConfig`、`Window` 和 `Plugin` — 均帶有 `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`。`auto!` 巨集允許你使用 `{ }` 區塊建構匿名/內聯子結構體的實例，無需知道它們生成的名稱。

## 文件索引

| 文件 | 描述 |
| --- | --- |
| [derive_struct!](./derive-struct.md) | 結構體定義巨集 — 巢狀結構體、匿名結構體、Vec/Option 類型、預設值、內聯列舉、參考類型 |
| [derive_enum!](./derive-enum.md) | 列舉定義巨集 — 單元/結構體/元組變體、巢狀列舉、預設值 |
| [auto!](./auto-macro.md) | 實例建構巨集 — 匿名類型的簡化語法、列舉路徑、展開運算式 |
| [屬性與可見性](./attributes.md) | Derive 巨集、屬性傳播、`#[macros_recursive]`、欄位級屬性、可見性、`#[macro_export]`、跨 crate 使用 |
| [範例](./examples.md) | 真實場景範例、生成程式碼結構說明 |
