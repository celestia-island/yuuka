# Yuuka - 简介

**Yuuka** 是一个 Rust 过程宏库，允许你使用简洁的、类似 JSON 的 DSL 语法来定义复杂且深度嵌套的 struct 和 enum 层次结构。它基于 `serde` 构建，可以无缝进行序列化和反序列化。

## 安装

在 `Cargo.toml` 中添加以下依赖：

```toml
[dependencies]
yuuka = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> `serde` 和 `serde_json` 是可选的，但通常与 yuuka 一起使用以支持序列化。

## 核心宏

Yuuka 导出三个过程宏：

| 宏 | 用途 |
| --- | --- |
| [`derive_struct!`](./derive-struct.md) | 使用类 JSON DSL 定义嵌套的 struct 层次结构 |
| [`derive_enum!`](./derive-enum.md) | 定义带有多种变体形式的 enum 类型 |
| [`auto!`](./auto-macro.md) | 使用简化语法构造由上述宏生成的类型实例 |

另请参阅：

- [属性与可见性](./attributes.md) — 额外的 derive 宏、属性传播、可见性控制和跨 crate 使用
- [示例](./examples.md) — 真实场景示例和生成代码结构说明

## 快速开始

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

此次 `derive_struct!` 调用会自动生成三个独立的结构体 — `GameConfig`、`Window` 和 `Plugin` — 均带有 `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`。`auto!` 宏允许你使用 `{ }` 块构造匿名/内联子结构体的实例，无需知道它们生成的名称。

## 文档索引

| 文档 | 描述 |
| --- | --- |
| [derive_struct!](./derive-struct.md) | 结构体定义宏 — 嵌套结构体、匿名结构体、Vec/Option 类型、默认值、内联枚举、引用类型 |
| [derive_enum!](./derive-enum.md) | 枚举定义宏 — 单元/结构体/元组变体、嵌套枚举、默认值 |
| [auto!](./auto-macro.md) | 实例构造宏 — 匿名类型的简化语法、枚举路径、展开表达式 |
| [属性与可见性](./attributes.md) | Derive 宏、属性传播、`#[macros_recursive]`、字段级属性、可见性、`#[macro_export]`、跨 crate 使用 |
| [示例](./examples.md) | 真实场景示例、生成代码结构说明 |
