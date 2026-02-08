# `derive_struct!` 매크로

`derive_struct!` 매크로는 yuuka 의 핵심입니다. 간결한 JSON 스타일 DSL 을 사용하여 복잡한 중첩 구조체 계층 구조를 정의할 수 있습니다. 모든 인라인 타입은 자동으로 독립적인 최상위 구조체/열거형 정의로 추출됩니다.

## 기본 문법

```rust
use yuuka::derive_struct;

derive_struct!(
    Root {
        field1: String,
        field2: i32,
    }
);
```

이 코드는 `#[derive(Debug, Clone, Default)]` 가 자동으로 적용된 구조체를 생성합니다:

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct Root {
    pub field1: String,
    pub field2: i32,
}
```

---

## 중첩 구조체

부모 구조체 내부에서 타입으로 `FieldName { ... }` 을 지정하여 인라인 하위 구조체를 직접 정의할 수 있습니다:

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

이 코드는 `Root`, `Info`, `Detail` 이라는 **세 개의** 독립적인 구조체를 생성합니다. 각각은 생성된 모듈 내에서 모든 필드가 공개된 일반적인 Rust 구조체입니다.

임의의 깊이로 중첩할 수 있습니다:

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

## 익명 구조체

타입 이름을 생략하면 자동으로 이름이 지정된 구조체가 생성됩니다:

```rust
derive_struct!(
    Root {
        data: {
            value: String,
        },
    }
);
```

익명 구조체는 자동으로 `_Root_0_anonymous` 라는 이름이 부여됩니다. 여러 개의 익명 타입이 있는 경우, 순차적으로 번호가 매겨집니다:

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
// 생성: _Root_0_anonymous (a용), _Root_1_anonymous (c용)
```

익명 구조체는 깊게 중첩될 수 있습니다:

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

> **팁**: 생성된 이름을 몰라도 익명 구조체 인스턴스를 구축하려면 [`auto!`](./auto-macro.md) 매크로를 사용하세요.

---

## 배열 (Vec) 타입

`[Type { ... }]` 구문을 사용하여 `Vec<Type>` 필드를 정의합니다:

```rust
derive_struct!(
    Root {
        items: [Item {
            name: String,
            count: u32,
        }],
    }
);
// 생성되는 필드: items: Vec<Item>
```

### 익명 배열 요소

```rust
derive_struct!(
    Root {
        items: [{
            name: String,
        }],
    }
);
// 생성: items: Vec<_Root_0_anonymous>
```

### 열거형 배열

```rust
derive_struct!(
    Root {
        statuses: [enum Status {
            Active,
            Inactive,
        }],
    }
);
// 생성: statuses: Vec<Status>
```

### 익명 열거형 배열

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
// 생성: values: Vec<_Root_0_anonymous>
```

---

## 옵션 (Option) 타입

필드 이름 뒤에 `?` 를 추가하면 타입이 `Option<T>` 으로 래핑됩니다:

```rust
derive_struct!(
    Root {
        required: String,
        optional?: String,
    }
);
// 생성:
//   required: String,
//   optional: Option<String>,
```

### 인라인 구조체와 Option

```rust
derive_struct!(
    Root {
        detail?: Detail {
            info: String,
        },
    }
);
// 생성: detail: Option<Detail>
```

### 익명 구조체와 Option

```rust
derive_struct!(
    Root {
        data?: {
            value: String,
        },
    }
);
// 생성: data: Option<_Root_0_anonymous>
```

### 인라인 열거형과 Option

```rust
derive_struct!(
    Root {
        status?: enum Status {
            Active,
            Inactive,
        },
    }
);
// 생성: status: Option<Status>
```

### 열거형 배리언트 내 Option

`?` 구문은 열거형 구조체 배리언트 내에서도 작동합니다:

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

## 기본값

타입 뒤에 `=` 를 사용하여 기본값을 지정합니다:

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

### 동작 방식

- 명시적 `= value` 가 있는 필드는 생성된 `impl Default` 에서 해당 값을 사용합니다.
- `= value` 가 없는 필드는 `Default::default()` 를 사용합니다 (예: 숫자는 `0`, String 은 `""`, bool 은 `false`).
- 하나라도 커스텀 기본값이 있는 필드가 있으면, 매크로는 `#[derive(Default)]` 대신 수동 `impl Default` 블록을 생성합니다.

### 배열의 기본값

```rust
derive_struct!(
    Root {
        // 기본적으로 비어 있음
        items: [Item {
            name: String = "unnamed".to_string(),
        }],
    }
);

let root = Root::default();
assert_eq!(root.items.len(), 0); // Vec은 기본적으로 비어 있음
```

명시적 배열 기본값 사용:

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

// 새 항목은 Item 수준의 기본값을 받음
root.items.push(Default::default());
assert_eq!(root.items[1].name, "unnamed");
```

### 열거형의 기본값

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

기본값이 있는 열거형 배열:

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
assert_eq!(root.members[0], Member::Arisu); // Vec 기본값으로부터
root.members.push(Default::default());
assert_eq!(root.members[1], Member::Midori); // enum 기본값으로부터
```

---

## 인라인 열거형

구조체 필드 내에서 열거형을 인라인으로 정의합니다:

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

### 배리언트 형태

열거형 배리언트는 세 가지 형태를 지원합니다:

**유닛 배리언트** — 연관 데이터 없음:

```rust
a: enum Member {
    Momoi,
    Midori,
    Yuzu,
    Arisu,
}
```

**구조체형 배리언트** — 이름이 있는 필드로, 인라인 구조체와 열거형을 포함할 수 있음:

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

**튜플형 배리언트** — 위치 기반 데이터로, 인라인 구조체, 다중 필드, 정적 타입을 지원:

```rust
a: enum Member {
    Momoi(Skill { name: String }),
    Midori(Vec<String>, usize),
    Yuzu(SkillYuzu { name: String }, usize),
    Arisu(usize),
}
```

### 중첩 열거형

열거형은 열거형 배리언트 내에 중첩될 수 있습니다:

```rust
// 구조체형 배리언트 내 열거형
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

// 튜플형 배리언트 내 열거형
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

### 배리언트 내 열거형 배열

```rust
// 구조체형 배리언트 내 Vec<enum>
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

// 튜플형 배리언트 내 Vec<enum>
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

## 참조 타입

경로 구문을 사용하여 외부에서 정의된 타입을 참조합니다:

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

> **참고**: 생성된 타입은 모듈(`__Root`) 내부에 위치하므로, 외부 스코프의 타입을 참조하려면 일반적으로 `super::` 가 필요합니다. 정확한 경로는 매크로 호출 위치를 기준으로 외부 타입이 정의된 위치에 따라 달라집니다.

필드 이름은 유연하며 — snake_case 와 PascalCase 이름 모두 작동합니다:

```rust
derive_struct!(
    Root {
        a_b: String,
        B: i32,
        c: super::C,
    }
);
```
