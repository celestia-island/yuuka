# `derive_enum!` 매크로

`derive_enum!` 매크로는 `derive_struct!` 과 동일한 DSL 구문 스타일로 독립적인 열거형 타입을 정의합니다. 세 가지 배리언트 형태, 중첩 타입, 기본값을 모두 지원합니다.

## 기본 문법

```rust
use yuuka::derive_enum;

derive_enum!(
    enum Status {
        Active,
        Inactive,
    }
);
```

이 코드는 `#[derive(Debug, Clone)]` 이 자동으로 적용된 열거형을 생성합니다.

---

## 배리언트 형태

### 유닛 배리언트

연관 데이터가 없는 단순 배리언트:

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

### 구조체형 배리언트

이름이 있는 필드를 가진 배리언트입니다. 필드에 인라인 구조체 정의를 사용할 수 있습니다:

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
// Action 열거형과 함께 독립적인 `Target` 구조체가 생성됩니다.
```

### 튜플형 배리언트

위치 기반 데이터를 가진 배리언트입니다. 인라인 구조체, 열거형, 정적 타입을 포함할 수 있습니다:

```rust
derive_enum!(
    enum Message {
        Text(String),
        Data(Payload { content: Vec<u8>, size: usize }),
        Multi(String, i32, bool),
    }
);
```

### 혼합 배리언트

세 가지 형태 모두 하나의 열거형에 공존할 수 있습니다:

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

## 중첩 열거형

열거형 배리언트는 다른 인라인 열거형을 포함할 수 있습니다:

### 튜플 배리언트 내

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

### 익명 중첩 열거형

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

배리언트 내 익명 열거형은 `_Root_0_anonymous` 와 같은 이름이 부여됩니다. 모듈을 통해 참조할 수 있습니다:

```rust
let _ = Root::D(__Root::_Root_0_anonymous::E);
```

> **팁**: 생성된 익명 이름을 다루지 않으려면 [`auto!`](./auto-macro.md)를 사용하세요. `auto!(Root::D::E)` 가 자동으로 경로를 해석합니다.

### 깊게 중첩된 익명 열거형

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

// 수동 구축:
let _ = A::B(_A_0_anonymous::C(_A_1_anonymous::D(_A_2_anonymous::E(_A_3_anonymous::F))));

// auto! 사용:
use yuuka::auto;
let _ = auto!(A::B::C::D::E::F);
let _ = auto!(A::B::C::D::E::G("hello".to_string()));
```

---

## 기본값

닫는 중괄호 뒤에 `= VariantName` 을 사용하여 기본 배리언트를 지정합니다:

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

### 튜플 배리언트의 기본값

```rust
derive_enum!(
    enum Value {
        Int(i32),
        Text(String),
    } = Int(0)
);
```

### 중첩 익명 열거형의 기본값

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

> **참고**: 기본값이 지정되지 않은 경우, 생성된 `impl Default` 는 `unimplemented!()` 를 사용하며, 호출 시 런타임에서 패닉이 발생합니다. `Default::default()` 를 사용할 계획이라면 반드시 기본값을 지정하세요.

---

## 추가 derive 및 속성 매크로

`derive_struct!` 과 마찬가지로, `#[derive(...)]` 및 속성 매크로를 전달할 수 있습니다:

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

속성 매크로, 재귀적 전파, 배리언트 수준 속성에 대한 자세한 내용은 [속성 및 가시성](./attributes.md)을 참조하세요.

---

## `derive_struct!` 와의 연동

`derive_enum!` 로 정의된 열거형은 `derive_struct!` 의 필드 타입으로 사용할 수 있으며, 그 반대도 가능합니다. `derive_struct!` 내에서 인라인으로 열거형을 정의하는 방법은 [derive_struct! — 인라인 열거형](./derive-struct.md#인라인-열거형) 섹션을 참조하세요.

`auto!` 매크로는 두 매크로로 생성된 타입 모두에서 작동하며, 익명 열거형 경로를 자동으로 해석하여 수동 구축의 번거로움을 없애줍니다.
