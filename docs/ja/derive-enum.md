# `derive_enum!` マクロ

`derive_enum!` マクロは、`derive_struct!` と同じ DSL 構文スタイルで独立した列挙型を定義します。3 つのバリアント形式、ネスト型、デフォルト値をすべてサポートします。

## 基本構文

```rust
use yuuka::derive_enum;

derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);
```

生成される列挙型には `#[derive(Debug, Clone)]` が自動適用されます。

---

## バリアントの形式

### ユニットバリアント

関連データを持たないシンプルなバリアント：

```rust
derive_enum!(
    enum Direction {
        North,
        South,
        East,
        West,
    }
);
```

### 構造体バリアント

名前付きフィールドを持つバリアント。フィールドにはインライン構造体定義を使用できます：

```rust
derive_enum!(
    enum Action {
        Move { x: f64, y: f64 },
        Attack {
            target: Target {
                id: u64,
                name: String,
            },
            damage: u32,
        },
    }
);
// Action 列挙型とともに独立した `Target` 構造体が生成されます。
```

### タプルバリアント

位置データを持つバリアント。インライン構造体、列挙型、静的型を含められます：

```rust
derive_enum!(
    enum Message {
        Text(String),
        Data(Payload { content: Vec<u8>, size: usize }),
        Multi(String, i32, bool),
    }
);
```

### 混合バリアント

3 つの形式すべてを 1 つの列挙型内で共存させることができます：

```rust
derive_enum!(
    #[derive(PartialEq, Serialize, Deserialize)]
    enum Router {
        Home,
        User { id: u64, name: String },
        Error(String),
    }
);
```

---

## ネスト列挙型

列挙型バリアントは他のインライン列挙型を含められます：

### タプルバリアント内のネスト

```rust
derive_enum!(
    enum Group {
        Millennium(enum Millennium {
            GameDevelopment(enum GameDevelopment {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            }),
            CAndC,
            Veritas,
        }),
    }
);

let _ = Group::Millennium(Millennium::GameDevelopment(GameDevelopment::Yuzu));
```

### 匿名ネスト列挙型

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        A,
        B(i32),
        C { a: String, b: i32 },
        D(enum {
            E,
            F(i32),
            G { a: String, b: i32 },
        }),
    }
);
```

バリアント内の匿名列挙型は `_Root_0_anonymous` のように命名されます。モジュール経由で参照できます：

```rust
let _ = Root::D(__Root::_Root_0_anonymous::E);
```

> **ヒント**: 生成された匿名名の扱いを避けるには [`auto!`](./auto-macro.md) を使ってください。`auto!(Root::D::E)` でパスが自動解決されます。

### 深くネストされた匿名列挙型

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum A {
        B(enum {
            C(enum {
                D(enum {
                    E(enum {
                        F,
                        G(String),
                    }),
                }),
            }),
        }),
    }
);

// 手動で構築する場合:
let _ = A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))));

// auto! を使う場合:
use yuuka::auto;
let _ = auto!(A::B::C::D::E::F);
let _ = auto!(A::B::C::D::E::G("hello".to_string()));
```

---

## デフォルト値

閉じ波括弧の後に `= VariantName` でデフォルトバリアントを指定します：

```rust
derive_enum!(
    enum Theme {
        Light,
        Dark,
        System,
    } = Dark
);

let theme = Theme::default();
// theme == Theme::Dark
```

### タプルバリアントのデフォルト

```rust
derive_enum!(
    enum Value {
        Int(i32),
        Text(String),
    } = Int(0)
);
```

### 匿名ネスト列挙型のデフォルト

```rust
derive_enum!(
    enum Group {
        Millennium(enum {
            GameDevelopment(enum GameDevelopment {
                Momoi,
                Midori,
                Yuzu,
                Arisu,
            } = Yuzu),
            CAndC,
            Veritas,
        } = GameDevelopment(Default::default())),
    } = Millennium(Default::default())
);

// Group::default() == Group::Millennium(GameDevelopment(Yuzu))
```

> **注意**: デフォルト値が指定されていない場合、生成される `impl Default` は `unimplemented!()` を使います。これは実行時に呼び出されるとパニックします。`Default::default()` を使う予定がある場合は、必ずデフォルトを指定してください。

---

## 追加 derive と属性マクロ

`derive_struct!` と同様に、`#[derive(...)]` や属性マクロを渡せます：

```rust
derive_enum!(
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Member {
        SaibaMomoi,
        SaibaMidori,
        HanaokaYuzu,
        TendouAris,
    } = SaibaMidori
);

let json = serde_json::to_string(&Member::default()).unwrap();
assert_eq!(json, r#""saiba_midori""#);
```

属性マクロ、再帰的伝播、バリアントレベル属性の全詳細は [属性と可視性](./attributes.md) を参照してください。

## `derive_struct!` との連携

`derive_struct!` のフィールド内にインライン列挙型を定義する方法については、[`derive_struct!` — インライン列挙型](./derive-struct.md#インライン列挙型) を参照してください。`derive_enum!` で定義した列挙型は、`derive_struct!` のフィールド型として参照型（パス構文）を使って参照することもできます。
