# 示例

真实场景的使用示例以及 yuuka 宏生成代码的结构说明。

---

## 语言包（国际化）

一个典型用例：定义嵌套的语言包结构，直接映射到 JSON 文件。通过 serde 支持序列化/反序列化。

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

    // 从 JSON 反序列化
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

要点：

- 字段名可以使用**非 ASCII 字符**（中文等）— 它们同时作为 Rust 标识符和 JSON 键。
- `auto!` 宏无缝处理匿名子结构体的构造（主页、设置、网络配置）。
- 生成的类型完全兼容 serde，支持 JSON 往返序列化。

---

## 服务器路由配置

一个更复杂的示例，建模反向代理/服务器路由配置，包含嵌套数组和内联枚举。

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

    // 此结构可以直接与 JSON 互转
    let json = serde_json::to_string_pretty(&config)?;
    let config_from_json: Config = serde_json::from_str(&json)?;
    assert_eq!(config, config_from_json);

    Ok(())
}
```

要点：

- **`[Service { ... }]`** 生成 `services: Vec<Service>`，`Service` 作为独立结构体。
- **嵌套的 `[Rule { ... }]`** 在 Service 内生成 `Vec<Rule>`，Rule 为内联结构体。
- **`enum Method { ... }`** 内联定义枚举，使用结构体变体表示不同的路由方法。
- 整个配置可以从 JSON 加载/保存到 JSON。

---

## 生成代码结构

理解 yuuka 生成的代码有助于调试和有效使用本库。

当你编写：

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

宏大致生成如下代码：

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

    // auto! 的辅助宏
    macro_rules! __auto_Root {
        (name $($tt:tt)*) => { $($tt)* };
        (child { $($tt:tt)* }) => { ::yuuka::auto!(Child { $($tt)* }) };
        (child $($tt:tt)*) => { $($tt)* };
        // ... 每个字段的更多规则
    }

    macro_rules! __auto_Child {
        (value $($tt:tt)*) => { $($tt)* };
        // ... 每个字段的更多规则
    }
}
pub use __Root::*;
```

### 关键要素

1. **模块包裹**：所有类型放入名为 `__TypeName` 的模块中，避免命名冲突。通过 `use __TypeName::*` 全部重新导出。

2. **自动派生**：`Debug` 和 `Clone` 始终添加。你自定义的 `#[derive(...)]` 宏会追加在后面。

3. **Default 实现**：如果没有字段有自定义默认值 → `#[derive(Default)]`。如果任何字段有 `= value` → 手动 `impl Default { ... }`。

4. **辅助宏**：为每个类型生成 `__auto_TypeName!` 宏。这些是 `macro_rules!` 宏，由 `auto!` 过程宏调用来解析字段类型 — 特别是匿名 struct/enum 名称。

5. **Super 导入**：`use super::*` 将外部作用域引入模块，这就是为什么引用外部类型时需要 `super::` 前缀。

### 模块命名规则

| 输入 | 模块名 |
| --- | --- |
| `Root { ... }` | `__Root` |
| `Config { ... }` | `__Config` |
| Root 中的匿名字段 | `_Root_0_anonymous`、`_Root_1_anonymous`、... |
| 枚举 A 中的匿名字段 | `_A_0_anonymous`、`_A_1_anonymous`、... |
