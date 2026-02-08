# `auto!` マクロ

`auto!` マクロは、`derive_struct!` と `derive_enum!` で生成された型のインスタンス構築を簡略化します。主な価値は匿名型名の自動解決にあり、人間が読みやすいパスを記述する一方で、マクロが正しい生成名に展開します。

## 基本コンセプト

匿名構造体や列挙型を使うと、yuuka は `_Root_0_anonymous` のような名前を生成します。これらを手動で構築するのは冗長で壊れやすいです：

```rust
derive_struct!(Root {
    data: {
        name: String,
        score: f64,
    },
});

// auto! なし — 生成名を知る必要がある
let val = Root {
    data: _Root_0_anonymous {
        name: "test".to_string(),
        score: 99.5,
    },
};

// auto! あり — { } を使うだけ
let val = auto!(Root {
    data: {
        name: "test".to_string(),
        score: 99.5,
    },
});
```

---

## 構造体の構築

### 基本的な構造体

```rust
derive_struct!(Root {
    a: String,
    b: i32,
});

let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
});
```

### ネストされた匿名構造体

```rust
derive_struct!(Root {
    a: String,
    b: i32,
    c: f64,
    d: {
        e: String = "world".to_string(),
        f: i32,
    },
});

let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: std::f64::consts::PI,
    d: {
        f: 24,
        ..Default::default()
    },
});
assert_eq!(obj.d.e, "world"); // デフォルト値
assert_eq!(obj.d.f, 24);      // 明示的に指定
```

### スプレッド式

標準の Rust 構造体更新構文と同様に、`..Default::default()` を使って残りのフィールドをデフォルト値で埋められます：

```rust
let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: 3.14,
    d: {
        f: 24,
        ..Default::default()  // eはデフォルト値 "world" を取得
    },
});
```

---

## 列挙型の構築

### ユニットバリアント

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        A,
        B(i32),
        C { a: String, b: i32 },
    }
);

assert_eq!(auto!(Root::A), Root::A);
```

### タプルバリアント

```rust
assert_eq!(auto!(Root::B(42)), Root::B(42));
```

### 構造体バリアント

```rust
assert_eq!(
    auto!(Root::C {
        a: "hello".to_string(),
        b: 42,
    }),
    Root::C {
        a: "hello".to_string(),
        b: 42,
    }
);
```

---

## 匿名列挙型のパス解決

ここが `auto!` の真価が発揮される場面です。タプルバリアント内にネストされた匿名列挙型に対して、`auto!` は複数レベルを通じてパスを解決します：

### 単一レベル

```rust
derive_enum!(
    #[derive(PartialEq)]
    enum Root {
        D(enum {
            E,
            F(i32),
            G { a: String, b: i32 },
        }),
    }
);

// auto! なし — 冗長
let _ = Root::D(__Root::_Root_0_anonymous::E);

// auto! あり — 簡潔
assert_eq!(auto!(Root::D::E), Root::D(__Root::_Root_0_anonymous::E));

assert_eq!(auto!(Root::D::F(42)), Root::D(__Root::_Root_0_anonymous::F(42)));

assert_eq!(
    auto!(Root::D::G {
        a: "hello".to_string(),
        b: 42,
    }),
    Root::D(__Root::_Root_0_anonymous::G {
        a: "hello".to_string(),
        b: 42,
    })
);
```

### 深くネストされたパス

`auto!` は任意の深さの匿名列挙型ネストを通じてパスを解決できます：

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

// 解決パス: A::B → _A_0_anonymous::C → _A_1_anonymous::D → _A_2_anonymous::E → _A_3_anonymous::F
assert_eq!(
    auto!(A::B::C::D::E::F),
    A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))))
);

assert_eq!(
    auto!(A::B::C::D::E::G("hello".to_string())),
    A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(
        _A_3_anonymous::G("hello".to_string())
    ))))
);
```

---

## 混合使用

`auto!` 呼び出しを他の `auto!` 呼び出しや通常の構造体構築の中にネストできます：

```rust
derive_struct!(
    #[derive(PartialEq)]
    Root {
        outer: {
            a: enum B {
                C {
                    c: i32,
                    d: f64,
                },
            },
        },
    }
);

let val = auto!(Root {
    outer: {
        a: auto!(B::C { c: 42, d: std::f64::consts::PI }),
    },
});

assert_eq!(val.outer.a, B::C { c: 42, d: std::f64::consts::PI });
```

---

## モジュール間の使用

型とヘルパーマクロがスコープ内にある限り、`auto!` はモジュール境界を越えて動作します：

```rust
#[macro_use]
mod definitions {
    use yuuka::derive_struct;

    derive_struct!(
        #[derive(PartialEq)]
        pub Root {
            a: String,
            b: i32,
        }
    );
}

mod usage {
    use yuuka::auto;
    use super::definitions::*;

    #[test]
    fn test() {
        assert_eq!(
            auto!(Root {
                a: "hello".to_string(),
                b: 42,
            }),
            Root {
                a: "hello".to_string(),
                b: 42,
            }
        );
    }
}
```

匿名型をモジュール間で使用する場合は、定義モジュールに `#[macro_use]` を付けてください：

```rust
#[macro_use]
mod definitions {
    use yuuka::derive_struct;

    derive_struct!(Root {
        data: {
            value: String,
        },
    });
}

mod usage {
    use yuuka::auto;
    use super::definitions::*;

    fn create() {
        let val = auto!(Root {
            data: {
                value: "hello".to_string(),
            },
        });
    }
}
```

クレート間の使用については、[属性と可視性 — クレート間の使用](./attributes.md#クレート間の使用) を参照してください。
