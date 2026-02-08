# `derive_struct!` マクロ

`derive_struct!` マクロは yuuka の中核です。簡潔な JSON 風 DSL を使って、複雑なネスト構造体の階層を定義できます。すべてのインライン型は自動的に独立したトップレベルの構造体/列挙型定義として抽出されます。

## 基本構文

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

これにより `#[derive(Debug, Clone, Default)]` が自動適用された構造体が生成されます：

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## ネスト構造体

型として `FieldName { ... }` を指定することで、親構造体の内部にインラインサブ構造体を直接定義できます：

```rust
derive_struct!(
    Root {
        info: Info {
            name: String,
            detail: Detail {
                level: u32,
                score: f64,
            },
        },
    }
);
```

これにより `Root`、`Info`、`Detail` の **3 つ** の独立した構造体が生成されます。それぞれが通常の Rust 構造体であり、生成モジュール内ではすべてのフィールドが public です。

任意の深さまでネストできます：

```rust
derive_struct!(
    Root {
        a: A {
            b: B {
                c: C {
                    d: D {
                        value: String,
                    },
                },
            },
        },
    }
);
```

---

## 匿名構造体

型名を省略すると、自動命名された構造体が作成されます：

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

匿名構造体は自動的に `_Root_0_anonymous` と命名されます。複数の匿名型がある場合は連番が付きます：

```rust
derive_struct!(
    Root {
        a: {
            b: String,
        },
        c: {
            d: f64,
        },
    }
);
// 生成される名前: _Root_0_anonymous (aに対応), _Root_1_anonymous (cに対応)
```

匿名構造体は深くネストできます：

```rust
derive_struct!(
    Root {
        a: {
            b: String,
            c: {
                d: f64 = std::f64::consts::PI,
                e: {
                    f: bool = false,
                },
            },
            g: {
                h: i32 = -114514,
            },
        },
    }
);

let root = Root::default();
assert_eq!(root.a.c.d, std::f64::consts::PI);
assert!(!root.a.c.e.f);
assert_eq!(root.a.g.h, -114514);
```

> **ヒント**: 生成された名前を知らなくても匿名構造体のインスタンスを構築するには、[`auto!`](./auto-macro.md) マクロを使ってください。

---

## 配列型 (Vec)

`[Type { ... }]` 構文を使って `Vec<Type>` フィールドを定義します：

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// 生成されるフィールド: items: Vec<Item>
```

### 匿名配列要素

```rust
derive_struct!(
    Root {
        items: [{
            name: String,
        }],
    }
);
// 生成される名前: items: Vec<_Root_0_anonymous>
```

### 列挙型配列

```rust
derive_struct!(
    Root {
        statuses: [enum Status {
            Active,
            Inactive,
        }],
    }
);
// 生成されるフィールド: statuses: Vec<Status>
```

### 匿名列挙型配列

```rust
derive_struct!(
    Root {
        values: [enum {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        }],
    }
);
// 生成される名前: values: Vec<_Root_0_anonymous>
```

---

## オプション型 (Option)

フィールド名の末尾に `?` を付けると、型が `Option<T>` でラップされます：

```rust
derive_struct!(
    Root {
        required: String,
        optional?: String,
    }
);
// 生成結果:
//   required: String,
//   optional: Option<String>,
```

### インライン構造体の Option

```rust
derive_struct!(
    Root {
        detail?: Detail {
            info: String,
        },
    }
);
// 生成結果: detail: Option<Detail>
```

### 匿名構造体の Option

```rust
derive_struct!(
    Root {
        data?: {
            value: String,
        },
    }
);
// 生成結果: data: Option<_Root_0_anonymous>
```

### インライン列挙型の Option

```rust
derive_struct!(
    Root {
        status?: enum Status {
            Active,
            Inactive,
        },
    }
);
// 生成結果: status: Option<Status>
```

### 列挙型バリアント内の Option

`?` 構文は列挙型の構造体バリアント内でも使えます：

```rust
derive_struct!(
    Root {
        action?: enum Action {
            Midori { detail?: String },
        },
    }
);
```

---

## デフォルト値

型の後に `=` でデフォルト値を割り当てます：

```rust
derive_struct!(
    Config {
        host: String = "localhost".to_string(),
        port: u16 = 8080,
        debug: bool = false,
    }
);

let config = Config::default();
assert_eq!(config.host, "localhost");
assert_eq!(config.port, 8080);
assert_eq!(config.debug, false);
```

### 動作の詳細

- 明示的に `= value` が指定されたフィールドは、生成される `impl Default` でその値を使います。
- `= value` が指定されていないフィールドは `Default::default()` を使います（数値なら `0`、String なら `""`、bool なら `false` など）。
- いずれかのフィールドにカスタムデフォルトがある場合、マクロは `#[derive(Default)]` の代わりに手動の `impl Default` ブロックを生成します。

### 配列のデフォルト値

```rust
derive_struct!(
    Root {
        // デフォルトでは空
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Vecはデフォルトで空
```

明示的な配列デフォルトを指定する場合：

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String = "unnamed".to_string(),
        }] = vec![Item { name: "first".to_string() }],
    }
);

let mut root = Root::default();
assert_eq!(root.items.len(), 1);
assert_eq!(root.items[0].name, "first");

// 新しい要素はItemレベルのデフォルト値を使用
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### 列挙型のデフォルト値

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        member: enum Member {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        } = Midori,
    }
);

let root = Root::default();
assert_eq!(root.member, Member::Midori);
```

列挙型配列のデフォルト値：

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        members: [enum Member {
            Momoi,
            Midori,
            Yuzu,
            Arisu,
        } = Midori] = vec![Member::Arisu],
    }
);

let mut root = Root::default();
assert_eq!(root.members[0], Member::Arisu); // Vecのデフォルト値
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // 列挙型のデフォルト値
```

---

## インライン列挙型

構造体フィールド内に列挙型をインラインで定義します：

```rust
derive_struct!(
    Root {
        status: enum Status {
            Active,
            Inactive,
        },
    }
);
```

### バリアントの形式

列挙型のバリアントは 3 つの形式をサポートします：

**ユニットバリアント** — 関連データなし：

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**構造体バリアント** — 名前付きフィールドを持ち、フィールド自体にインライン構造体や列挙型を含められます：

```rust
a: enum Member {
    Momoi {
        skill: Skill {
            name: String,
        },
    },
    Midori { skills: Vec<String>, level: usize },
    Yuzu {
        skill: SkillYuzu {
            name: String,
        },
        level: usize,
    },
    Arisu { level: usize },
}
```

**タプルバリアント** — 位置データを持ち、インライン構造体、複数フィールド、静的型をサポートします：

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### ネスト列挙型

列挙型バリアントの内部に他の列挙型をネストできます：

```rust
// 構造体バリアント内の列挙型
derive_struct!(
    Root {
        a: enum Member {
            Arisu {
                ty: enum ArisuType {
                    Arisu,
                    Key,
                },
            },
        },
    }
);
let _ = Root { a: Member::Arisu { ty: ArisuType::Key } };

// タプルバリアント内の列挙型
derive_struct!(
    Root {
        a: enum Member {
            Arisu(enum ArisuType {
                Arisu,
                Key,
            }),
        },
    }
);
let _ = Root { a: Member::Arisu(ArisuType::Key) };
```

### バリアント内の列挙型配列

```rust
// 構造体バリアント内のVec<enum>
derive_struct!(
    Root {
        a: enum Member {
            Arisu {
                ty: [enum ArisuType {
                    Arisu,
                    Key,
                }],
            },
        },
    }
);

// タプルバリアント内のVec<enum>
derive_struct!(
    Root {
        a: enum Member {
            Arisu([enum ArisuType {
                Arisu,
                Key,
            }]),
        },
    }
);
```

---

## 参照型

パス構文を使って外部で定義された型を参照します：

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct ExternalType {
    data: f64,
}

derive_struct!(
    Root {
        name: String,
        external: super::ExternalType,
    }
);
```

> **注意**: 生成された型はモジュール（`__Root`）の内部に配置されるため、通常は外部スコープの型を参照するのに `super::` が必要です。正確なパスは、外部型がマクロ呼び出しに対してどこに定義されているかによります。

フィールド名は柔軟で、snake_case と PascalCase の両方が使えます：

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
