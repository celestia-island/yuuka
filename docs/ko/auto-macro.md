# `auto!` 매크로

`auto!` 매크로는 `derive_struct!` 과 `derive_enum!` 으로 생성된 타입의 인스턴스를 간편하게 구축합니다. 가장 큰 가치는 익명 타입 이름을 자동으로 해석하는 것입니다 — 사람이 읽을 수 있는 경로를 작성하면 매크로가 올바른 생성된 이름으로 확장합니다.

## 핵심 개념

익명 구조체나 열거형을 사용하면, yuuka 는 `_Root_0_anonymous` 와 같은 이름을 생성합니다. 이를 수동으로 구축하는 것은 장황하고 깨지기 쉽습니다:

```rust
derive_struct!(Root {
    data: {
        name: String,
        score: f64,
    },
});

// auto! 없이 — 생성된 이름을 알아야 함
let val = Root {
    data: _Root_0_anonymous {
        name: "test".to_string(),
        score: 99.5,
    },
};

// auto! 사용 — { }만 사용하면 됨
let val = auto!(Root {
    data: {
        name: "test".to_string(),
        score: 99.5,
    },
});
```

---

## 구조체 구축

### 기본 구조체

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

### 중첩 익명 구조체

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
assert_eq!(obj.d.e, "world"); // 기본값으로부터
assert_eq!(obj.d.f, 24);      // 명시적으로 설정
```

### 전개 표현식

표준 Rust 구조체 업데이트 구문과 동일하게 `..Default::default()` 를 사용하여 나머지 필드를 기본값으로 채울 수 있습니다:

```rust
let obj = auto!(Root {
    a: "hello".to_string(),
    b: 42,
    c: 3.14,
    d: {
        f: 24,
        ..Default::default()  // e는 기본값 "world"를 받음
    },
});
```

---

## 열거형 구축

### 유닛 배리언트

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

### 튜플 배리언트

```rust
assert_eq!(auto!(Root::B(42)), Root::B(42));
```

### 구조체형 배리언트

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

## 익명 열거형 경로 해석

`auto!` 가 진정으로 빛을 발하는 부분입니다. 튜플 배리언트 내의 익명 열거형에 대해, `auto!` 는 여러 수준의 경로를 해석합니다:

### 단일 수준

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

// auto! 없이 — 장황함
let _ = Root::D(__Root::_Root_0_anonymous::E);

// auto! 사용 — 깔끔함
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

### 깊은 중첩 경로

`auto!` 는 임의 깊이의 익명 열거형 중첩을 통해 경로를 해석할 수 있습니다:

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

// 해석: A::B → _A_0_anonymous::C → _A_1_anonymous::D → _A_2_anonymous::E → _A_3_anonymous::F
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

## 혼합 사용

`auto!` 호출을 다른 `auto!` 호출이나 일반 구조체 구축 내에 중첩할 수 있습니다:

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

## 모듈 간 사용

타입과 헬퍼 매크로가 스코프 내에 있는 한, `auto!`는 모듈 경계를 넘어 작동합니다:

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

모듈 간에 익명 타입을 사용할 때는 정의 모듈에 `#[macro_use]` 를 표시해야 합니다:

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

크레이트 간 사용에 대해서는 [속성 및 가시성 — 크레이트 간 사용](./attributes.md#크레이트-간-사용)을 참조하세요.
