# yuuka — 项目状态与计划 (PLAN)

> 刷新于 2026-07-14。嵌套结构派生宏库。

## 1. 项目概述

- **名称**：`yuuka`
- **简介**：proc-macro 库，对"嵌套结构"（struct-of-struct、enum-of-enum、深度泛型）自动派生 `Debug` / `Clone` / `PartialEq` / `Serialize` / `Deserialize` 等 trait。
- **远程仓库**：https://github.com/celestia-island/yuuka.git
- **技术栈**：Rust / proc-macro2 / syn / quote
- **类别**：library（codegen）

## 2. 当前状态

- **当前分支**：`dev`
- **工作区**：干净
- **最近提交时间**：2026-07-12
- **最近提交**：`🔧 Pin script recipes to the resolved Git Bash to survive WSL shadowing.`
- **本地领先 `origin/dev`**：0

## 3. 未提交改动

无

## 4. 近期进展

- `🔧 Pin script recipes to the resolved Git Bash to survive WSL shadowing.`
- `🔧 Switch the justfile to Git Bash and fetch devtools recipes on demand.`
- `♻️ Standardize windows-shell to pwsh.exe across celestia repos.`
- `🐛 Replace shebang recipes with [script(...)] to fix the Windows cygpath error.`
- `📝 Add FUNDING.yml for GitHub Sponsors.`

## 5. 后续计划

1. **trait 集合扩展**：目前 5 个常用 trait；后续加 `Default`（按字段零值）与 `From<X>`。
2. **错误信息增强**：当前 attribute 误用提示较弱，引入 `darling` 或自实现。
3. **性能基线**：macro 展开后产物体积与手写对等的差距测量。

## 6. 跨仓依赖

- 被 entelecheia 与 arona 多个 crate 通过派生宏调用。
- 本仓无 `path = "../..."` 的硬编码 patch。

---

## 既有详细计划（存档）

API 与示例在 `docs/en/`；本文件只承载"当前态 → 后续计划"两部分。
