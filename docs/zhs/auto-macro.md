# `auto!` 宏

`auto!` 宏简化了由 `derive_struct!` 和 `derive_enum!` 生成的类型的实例构造。它的核心价值在于自动解析匿名类型名称 — 你编写易读的路径，宏会将其展开为正确的生成名称。

## 为什么需要 `auto!`？

使用匿名结构体或枚举时，yuuka 会生成 `_Root_0_anonymous` 这样的名称。手动构造这些类型既冗长又脆弱：

```rust
derive_struct!(Root {
    data: {
        name: String,
        score: f64,
    },
});

// 不用 auto! — 必须知道生成的名称
let val = Root {
    data: _Root_0_anonymous {
        name: "test".to_string(),
        score: 99.5,
    },
};

// 用 auto! — 直接使用 { }
let val = auto!(Root {
    data: {
        name: "test".to_string(),
        score: 99.5,
    },
});
```

---

## 结构体构造

### 基本结构体

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

### 嵌套匿名结构体

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
assert_eq!(obj.d.e, "world"); // 来自默认值
assert_eq!(obj.d.f, 24);      // 显式设置
```

### 展开表达式

使用 `..Default::default()` 用默认值填充剩余字段，与标准 Rust 结构体更新语法相同：

```rust
let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: 3.14,
    d: {
        f: 24,
        ..Default::default()  // e 获得默认值 "world"
    },
});
```

---

## 枚举构造

### 单元变体

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

### 元组变体

```rust
assert_eq!(auto!(Root::B(42)), Root::B(42));
```

### 结构体变体

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

## 匿名枚举路径解析

这是 `auto!` 真正发挥作用的地方。对于嵌套在元组变体中的匿名枚举，`auto!` 可以解析多层路径：

### 单层

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

// 不用 auto! — 冗长
let _ = Root::D(__Root::_Root_0_anonymous::E);

// 用 auto! — 简洁
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

### 深度嵌套路径

`auto!` 可以解析任意深度的匿名枚举嵌套路径：

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

// 解析：A::B → _A_0_anonymous::C → _A_1_anonymous::D → _A_2_anonymous::E → _A_3_anonymous::F
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

可以在 `auto!` 调用内嵌套其他 `auto!` 调用或常规结构体构造：

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

## 跨模块使用

只要类型及其辅助宏在作用域内，`auto!` 就可以跨模块使用：

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

跨模块使用匿名类型时，确保定义模块标记了 `#[macro_use]`：

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

有关跨 crate 使用，请参阅[属性与可见性 — 跨 Crate 使用](./attributes.md#跨-crate-使用)。
